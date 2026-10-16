use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::broadcast;
use zedazo_core::application::process::{
    self, ArtifactKind, JobId, ProcessPhase, ProcessRequest, ProgressReporter, RulesConfigSource,
};

use crate::app::AppState;
use crate::dto::artifact_kind_str;
pub use crate::dto::{JobEvent, JobManifest, JobRecord, JobStatus};
use crate::storage::Storage;

pub struct ApiProgress {
    tx: broadcast::Sender<JobEvent>,
    seq: std::sync::atomic::AtomicU64,
    events_path: PathBuf,
}

impl ApiProgress {
    pub fn new(tx: broadcast::Sender<JobEvent>, events_path: PathBuf) -> Self {
        Self {
            tx,
            seq: std::sync::atomic::AtomicU64::new(1),
            events_path,
        }
    }

    fn emit(&self, event: &str, data: serde_json::Value) {
        let id = self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let ev = JobEvent {
            id,
            event: event.to_string(),
            data: data.clone(),
        };
        if let Ok(line) = serde_json::to_string(&ev) {
            if let Ok(mut f) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.events_path)
            {
                let _ = writeln!(f, "{line}");
            }
        }
        let _ = self.tx.send(ev);
    }
}

impl ProgressReporter for ApiProgress {
    fn phase(&self, phase: ProcessPhase) {
        self.emit(
            "phase",
            serde_json::json!({ "phase": format!("{:?}", phase) }),
        );
    }
    fn metric(&self, name: &str, value: u64) {
        self.emit(
            "metric",
            serde_json::json!({ "name": name, "value": value }),
        );
    }
    fn message(&self, msg: &str) {
        self.emit("message", serde_json::json!({ "message": msg }));
    }
}

/// Lee eventos persistidos con id > after_id.
pub fn read_events_after(path: &std::path::Path, after_id: u64) -> Vec<JobEvent> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<JobEvent>(line).ok())
        .filter(|ev| ev.id > after_id)
        .collect()
}

pub fn spawn_job(state: AppState, job_id: String) {
    let epoch_at_spawn = state.current_wipe_epoch();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = run_job_sync(&state, &job_id, epoch_at_spawn) {
            tracing::error!(%job_id, error = %e, "job falló");
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                if state.current_wipe_epoch() != epoch_at_spawn {
                    return;
                }
                let mut jobs = state.jobs.write().await;
                if let Some(rec) = jobs.get_mut(&job_id) {
                    let msg = e.to_string();
                    let disk_full = msg.contains("No space left")
                        || msg.contains("espacio")
                        || msg.contains("ENOSPC");
                    rec.manifest.status = JobStatus::Failed;
                    rec.manifest.error = Some(if disk_full { "disco_lleno".into() } else { msg });
                    rec.manifest.completed_at = Some(now_rfc3339());
                    let _ = save_manifest_result(&state, &rec.manifest);
                    if disk_full {
                        // Intentar limpiar parciales; si falla, cola de reintento.
                        if state.storage.delete_job_dir(&job_id).is_err() {
                            let _ = state
                                .storage
                                .enqueue_pending_delete(crate::storage::PendingKind::Job, &job_id);
                        }
                    }
                }
            });
        }
    });
}

fn run_job_sync(state: &AppState, job_id: &str, epoch_at_spawn: u64) -> anyhow::Result<()> {
    let handle = tokio::runtime::Handle::current();
    if state.current_wipe_epoch() != epoch_at_spawn {
        anyhow::bail!("wipe tras spawn; job abandonado");
    }
    let (rec_snapshot, tx) = handle.block_on(async {
        let jobs = state.jobs.read().await;
        let rec = jobs
            .get(job_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("job no encontrado"))?;
        let events = state.events.read().await;
        let tx = events
            .get(job_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("sin canal de eventos"))?;
        Ok::<_, anyhow::Error>((rec, tx))
    })?;

    {
        let mut jobs = handle.block_on(state.jobs.write());
        if let Some(r) = jobs.get_mut(job_id) {
            r.manifest.status = JobStatus::Validating;
            r.manifest.started_at = Some(now_rfc3339());
            save_manifest(state, &r.manifest);
        }
    }

    if state.current_wipe_epoch() != epoch_at_spawn || rec_snapshot.cancel.is_cancelled() {
        anyhow::bail!("job cancelado o wipe antes de escribir");
    }

    let job_dir = state.storage.ensure_job_dir(job_id)?;
    let events_path = job_dir.join("events.ndjson");
    let input = job_dir.join("input.vcf");
    let upload_path = state
        .storage
        .upload_path(&rec_snapshot.manifest.input.upload_id);
    if !input.exists() {
        std::fs::copy(&upload_path, &input)?;
    }

    let config = if let Some(toml) = &rec_snapshot.config_toml {
        RulesConfigSource::TomlInline(toml.clone())
    } else {
        RulesConfigSource::BuiltIn
    };

    let mut arts: BTreeSet<ArtifactKind> =
        rec_snapshot.requested_artifacts.iter().copied().collect();
    if arts.is_empty() {
        arts.insert(ArtifactKind::Vcf);
        arts.insert(ArtifactKind::AuditTsv);
        arts.insert(ArtifactKind::StatsJson);
        arts.insert(ArtifactKind::Csv);
        arts.insert(ArtifactKind::Json);
    }

    let progress: Arc<dyn ProgressReporter> = Arc::new(ApiProgress::new(tx.clone(), events_path));
    let req = ProcessRequest {
        job_id: JobId(job_id.to_string()),
        input_path: input,
        working_directory: job_dir.clone(),
        config,
        requested_artifacts: arts,
        source_override: "auto".into(),
        dry_run: false,
        strict: false,
        cancellation: rec_snapshot.cancel.clone(),
        progress,
    };

    if rec_snapshot.cancel.is_cancelled() || state.current_wipe_epoch() != epoch_at_spawn {
        handle.block_on(async {
            let mut jobs = state.jobs.write().await;
            if let Some(r) = jobs.get_mut(job_id) {
                r.manifest.status = JobStatus::Cancelled;
                r.manifest.completed_at = Some(now_rfc3339());
                save_manifest(state, &r.manifest);
            }
        });
        return Ok(());
    }

    let result = process::run(&req)?;

    if state.current_wipe_epoch() != epoch_at_spawn {
        anyhow::bail!("wipe durante el procesamiento; no se persisten resultados");
    }

    handle.block_on(async {
        let mut jobs = state.jobs.write().await;
        if let Some(r) = jobs.get_mut(job_id) {
            if r.cancel.is_cancelled() {
                r.manifest.status = JobStatus::Cancelled;
            } else {
                r.manifest.status = JobStatus::Completed;
                r.manifest.summary = Some(result.summary.clone());
                r.contact_views = result.contact_views.clone();
                r.duplicate_groups = result.duplicate_groups.clone();
                r.warnings = result.warnings.clone();
                r.manifest.artifacts = result
                    .artifacts
                    .iter()
                    .map(|a| artifact_kind_str(a.kind).to_string())
                    .collect();
                let contacts_path = job_dir.join("contacts_view.json");
                let _ = Storage::atomic_write(
                    &contacts_path,
                    serde_json::to_string_pretty(&result.contact_views)
                        .unwrap_or_default()
                        .as_bytes(),
                );
                let dup_path = job_dir.join("duplicates_view.json");
                let _ = Storage::atomic_write(
                    &dup_path,
                    serde_json::to_string_pretty(&result.duplicate_groups)
                        .unwrap_or_default()
                        .as_bytes(),
                );
            }
            r.manifest.completed_at = Some(now_rfc3339());
            save_manifest(state, &r.manifest);
        }
    });

    Ok(())
}

pub fn save_manifest(state: &AppState, manifest: &JobManifest) {
    let _ = save_manifest_result(state, manifest);
}

pub fn save_manifest_result(state: &AppState, manifest: &JobManifest) -> anyhow::Result<()> {
    crate::storage::Storage::validate_id(&manifest.job_id)?;
    let path = state
        .storage
        .job_dir(&manifest.job_id)
        .join("manifest.json");
    state.storage.assert_under_root(&path)?;
    let s = serde_json::to_string_pretty(manifest)?;
    Storage::atomic_write(&path, s.as_bytes())
}

pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}
