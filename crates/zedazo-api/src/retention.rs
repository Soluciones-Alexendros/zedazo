//! Reconciliación al arrancar y barrido periódico de retención.
//!
//! El TTL no es exacto al segundo: el barrido corre cada hora (y una vez al arrancar).

use std::collections::HashMap;
use std::sync::Arc;

use zedazo_core::application::process::{
    ArtifactKind, CancellationToken, NullProgress, ProcessRequest, RulesConfigSource,
};

use crate::app::AppState;
use crate::dto::{JobManifest, JobRecord, JobStatus};
use crate::jobs::now_rfc3339;
use crate::storage::{PendingKind, Storage, DEFAULT_UPLOAD_TTL_HOURS};

fn is_terminal(status: &JobStatus) -> bool {
    matches!(
        status,
        JobStatus::Completed
            | JobStatus::Failed
            | JobStatus::Cancelled
            | JobStatus::Expired
            | JobStatus::Deleted
    )
}

fn is_in_flight(status: &JobStatus) -> bool {
    !is_terminal(status) && !matches!(status, JobStatus::Created | JobStatus::Uploaded)
}

/// Reconstruye el registro en memoria desde manifiestos en disco.
pub async fn reconcile_on_startup(state: &AppState) -> anyhow::Result<()> {
    let retention_hours = state.retention_hours;
    let upload_ttl = std::env::var("ZEDAZO_UPLOAD_TTL_HOURS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_UPLOAD_TTL_HOURS);

    let mut recovered: HashMap<String, JobRecord> = HashMap::new();
    let mut referenced_uploads = std::collections::HashSet::new();

    for dir in state.storage.list_job_dirs()? {
        let id = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if Storage::validate_id(&id).is_err() {
            tracing::warn!(path = %dir.display(), "directorio de job con id inválido; en cola de limpieza");
            let _ = state
                .storage
                .enqueue_pending_delete(PendingKind::JobDir, &id);
            continue;
        }
        let manifest_path = dir.join("manifest.json");
        if !manifest_path.exists() {
            tracing::warn!(job_id = %id, "job sin manifiesto; en cola de limpieza");
            let _ = state.storage.enqueue_pending_delete(PendingKind::Job, &id);
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&manifest_path) else {
            tracing::warn!(job_id = %id, "manifiesto ilegible; en cola de limpieza");
            let _ = state.storage.enqueue_pending_delete(PendingKind::Job, &id);
            continue;
        };
        let Ok(mut manifest) = serde_json::from_str::<JobManifest>(&raw) else {
            tracing::warn!(job_id = %id, "manifiesto inválido; en cola de limpieza");
            let _ = state.storage.enqueue_pending_delete(PendingKind::Job, &id);
            continue;
        };

        referenced_uploads.insert(manifest.input.upload_id.clone());

        if is_in_flight(&manifest.status)
            || matches!(
                manifest.status,
                JobStatus::Queued
                    | JobStatus::CancelRequested
                    | JobStatus::Created
                    | JobStatus::Uploading
                    | JobStatus::Uploaded
            )
        {
            // Sin reanudación segura: estado terminal fallido.
            manifest.status = JobStatus::Failed;
            manifest.error = Some("proceso_interrumpido".into());
            if manifest.completed_at.is_none() {
                manifest.completed_at = Some(now_rfc3339());
            }
            let _ = crate::jobs::save_manifest_result(state, &manifest);
            tracing::info!(job_id = %id, "job interrumpido marcado como failed/proceso_interrumpido");
        }

        // Expirados: borrar ya.
        if should_expire(&manifest, retention_hours, jiff::Timestamp::now()) {
            match state.storage.delete_job_dir(&id) {
                Ok(()) => {
                    tracing::info!(job_id = %id, "job vencido eliminado en reconciliación");
                    continue;
                }
                Err(e) => {
                    tracing::error!(job_id = %id, error = %e, "fallo al borrar job vencido");
                    let _ = state.storage.enqueue_pending_delete(PendingKind::Job, &id);
                    continue;
                }
            }
        }

        // Para jobs Completed, cargar vistas persistidas o reconstruir (F-01, Q1-D híbrido)
        let (contact_views, duplicate_groups, warnings) = if manifest.status == JobStatus::Completed
        {
            let job_dir = state.storage.job_dir(&id);
            match load_views_or_rebuild(&job_dir, &manifest).await {
                Ok((cv, dg, w)) => (cv, dg, w),
                Err(e) => {
                    tracing::error!(job_id = %id, error = %e, "fallo al cargar/reconstruir vistas; job marcado como RecoveredPartial");
                    manifest.status = JobStatus::Failed;
                    manifest.error = Some(format!("recovery_failed: {}", e));
                    let _ = crate::jobs::save_manifest_result(state, &manifest);
                    (vec![], vec![], vec![])
                }
            }
        } else {
            (vec![], vec![], vec![])
        };

        // Reconstruir requested_artifacts desde el manifiesto
        let requested_artifacts: Vec<zedazo_core::application::process::ArtifactKind> = manifest
            .artifacts
            .iter()
            .filter_map(|s| match s.as_str() {
                "vcf" => Some(zedazo_core::application::process::ArtifactKind::Vcf),
                "audit_tsv" => Some(zedazo_core::application::process::ArtifactKind::AuditTsv),
                "stats_json" => Some(zedazo_core::application::process::ArtifactKind::StatsJson),
                "stats_markdown" => {
                    Some(zedazo_core::application::process::ArtifactKind::StatsMarkdown)
                }
                "csv" => Some(zedazo_core::application::process::ArtifactKind::Csv),
                "json" => Some(zedazo_core::application::process::ArtifactKind::Json),
                _ => None,
            })
            .collect();

        recovered.insert(
            id.clone(),
            JobRecord {
                manifest,
                contact_views,
                duplicate_groups,
                warnings,
                cancel: CancellationToken::new(),
                requested_artifacts,
                config_toml: None,
            },
        );
    }

    {
        let mut jobs = state.jobs.write().await;
        *jobs = recovered;
    }

    // Canales SSE vacíos (no se rehidratan).
    {
        let mut events = state.events.write().await;
        events.clear();
    }

    sweep_uploads(state, upload_ttl, &referenced_uploads)?;
    sweep_tmp(state)?;
    retry_pending_deletes(state).await?;

    Ok(())
}

/// Intenta cargar vistas persistidas (contacts_view.json, duplicates_view.json, warnings.json).
/// Si faltan o están corruptas, reconstruye re-procesando el VCF original desde artifacts/.
async fn load_views_or_rebuild(
    job_dir: &std::path::Path,
    manifest: &JobManifest,
) -> anyhow::Result<(
    Vec<zedazo_core::application::process::ContactView>,
    Vec<zedazo_core::application::process::DuplicateGroupView>,
    Vec<zedazo_core::application::process::ProcessWarning>,
)> {
    // Rutas de las vistas persistidas
    let contacts_path = job_dir.join("contacts_view.json");
    let duplicates_path = job_dir.join("duplicates_view.json");
    let warnings_path = job_dir.join("warnings.json");

    // Intentar cargar vistas existentes
    if contacts_path.exists() && duplicates_path.exists() && warnings_path.exists() {
        let contacts_raw = std::fs::read_to_string(&contacts_path)?;
        let duplicates_raw = std::fs::read_to_string(&duplicates_path)?;
        let warnings_raw = std::fs::read_to_string(&warnings_path)?;

        let contacts: Vec<zedazo_core::application::process::ContactView> =
            serde_json::from_str(&contacts_raw)?;
        let duplicates: Vec<zedazo_core::application::process::DuplicateGroupView> =
            serde_json::from_str(&duplicates_raw)?;
        let warnings: Vec<zedazo_core::application::process::ProcessWarning> =
            serde_json::from_str(&warnings_raw)?;

        tracing::info!(job_id = %manifest.job_id, "vistas cargadas desde JSON persistido");
        return Ok((contacts, duplicates, warnings));
    }

    // Fallback: reconstruir re-procesando el VCF original
    tracing::info!(job_id = %manifest.job_id, "vistas no encontradas o incompletas; reconstruyendo desde artefactos");

    // Buscar el VCF de entrada original en artifacts/ o en el directorio del job
    let input_vcf = job_dir.join("artifacts").join("input.vcf");
    let fallback_vcf = job_dir.join("input.vcf");
    let vcf_path = if input_vcf.exists() {
        input_vcf
    } else if fallback_vcf.exists() {
        fallback_vcf
    } else {
        // Sin VCF original: mantener vistas vacías (compatibilidad legacy)
        tracing::warn!(job_id = %manifest.job_id, "job Completed sin vistas persistidas ni VCF original; manteniendo vistas vacías");
        return Ok((vec![], vec![], vec![]));
    };

    // Configuración: usar BuiltIn o TomlInline del manifiesto si está disponible
    let config = if let Some(toml) = &manifest.config_toml {
        RulesConfigSource::TomlInline(toml.clone())
    } else {
        RulesConfigSource::BuiltIn
    };

    // Artefactos solicitados (los que estaban en el manifiesto original)
    let mut arts: std::collections::BTreeSet<ArtifactKind> = manifest
        .artifacts
        .iter()
        .filter_map(|s| match s.as_str() {
            "vcf" => Some(ArtifactKind::Vcf),
            "audit_tsv" => Some(ArtifactKind::AuditTsv),
            "stats_json" => Some(ArtifactKind::StatsJson),
            "stats_markdown" => Some(ArtifactKind::StatsMarkdown),
            "csv" => Some(ArtifactKind::Csv),
            "json" => Some(ArtifactKind::Json),
            _ => None,
        })
        .collect();
    if arts.is_empty() {
        arts.insert(ArtifactKind::Vcf);
        arts.insert(ArtifactKind::AuditTsv);
        arts.insert(ArtifactKind::StatsJson);
        arts.insert(ArtifactKind::Csv);
        arts.insert(ArtifactKind::Json);
    }

    let req = ProcessRequest {
        job_id: zedazo_core::application::process::JobId(manifest.job_id.clone()),
        input_path: vcf_path.clone(),
        working_directory: job_dir.to_path_buf(),
        config,
        requested_artifacts: arts,
        source_override: manifest
            .input
            .source_detected
            .clone()
            .unwrap_or_else(|| "auto".into()),
        dry_run: false,
        strict: false,
        cancellation: CancellationToken::new(),
        progress: Arc::new(NullProgress),
    };

    let result = zedazo_core::application::process::run(&req)
        .map_err(|e| anyhow::anyhow!("fallback re-proceso falló: {}", e))?;

    tracing::info!(job_id = %manifest.job_id, contacts = result.contact_views.len(), duplicates = result.duplicate_groups.len(), "vistas reconstruidas");
    Ok((
        result.contact_views,
        result.duplicate_groups,
        result.warnings,
    ))
}

/// ¿Ha vencido el TTL de retención? Reloj inyectable para tests (S-12).
pub fn should_expire(manifest: &JobManifest, default_retention: u64, now: jiff::Timestamp) -> bool {
    let Some(completed) = manifest.completed_at.as_ref() else {
        return false;
    };
    let Ok(ts) = completed.parse::<jiff::Timestamp>() else {
        return false;
    };
    let age_secs = now.duration_since(ts).as_secs();
    let hours = if manifest.retention_hours > 0 {
        manifest.retention_hours
    } else {
        default_retention
    };
    age_secs / 3600 >= hours as i64
}

fn sweep_uploads(
    state: &AppState,
    upload_ttl_hours: u64,
    referenced: &std::collections::HashSet<String>,
) -> anyhow::Result<()> {
    let now = jiff::Timestamp::now();
    for upload_id in state.storage.list_upload_ids()? {
        if referenced.contains(&upload_id) {
            continue;
        }
        if Storage::validate_id(&upload_id).is_err() {
            let _ = state
                .storage
                .enqueue_pending_delete(PendingKind::Upload, &upload_id);
            continue;
        }
        let meta_path = state.storage.upload_meta_path(&upload_id);
        let created = if meta_path.exists() {
            std::fs::read_to_string(&meta_path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| {
                    v.get("created_at")
                        .and_then(|c| c.as_str())
                        .and_then(|s| s.parse::<jiff::Timestamp>().ok())
                })
        } else {
            // Sin meta: usar mtime del fichero.
            state
                .storage
                .upload_path(&upload_id)
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| jiff::Timestamp::try_from(t).ok())
        };
        let Some(created) = created else {
            continue;
        };
        let age = now.duration_since(created);
        if age.as_secs() / 3600 >= upload_ttl_hours as i64 {
            match state.storage.delete_upload(&upload_id) {
                Ok(()) => tracing::info!(%upload_id, "upload huérfano eliminado"),
                Err(e) => {
                    tracing::error!(%upload_id, error = %e, "fallo al borrar upload");
                    let _ = state
                        .storage
                        .enqueue_pending_delete(PendingKind::Upload, &upload_id);
                }
            }
        }
    }
    Ok(())
}

fn sweep_tmp(state: &AppState) -> anyhow::Result<()> {
    let tmp = state.storage.root.join("tmp");
    if !tmp.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(&tmp)? {
        let entry = entry?;
        let path = entry.path();
        // Borrar tmp abandonado (>1 h por mtime).
        let old = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| jiff::Timestamp::try_from(t).ok())
            .map(|dt| jiff::Timestamp::now().duration_since(dt).as_secs() / 3600 >= 1)
            .unwrap_or(false);
        if old {
            if path.is_dir() {
                let _ = std::fs::remove_dir_all(&path);
            } else {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    Ok(())
}

async fn retry_pending_deletes(state: &AppState) -> anyhow::Result<()> {
    let items = state.storage.load_pending_deletes();
    if items.is_empty() {
        return Ok(());
    }
    let mut remaining = Vec::new();
    for item in items {
        let result = match item.kind {
            PendingKind::Job | PendingKind::JobDir => state.storage.delete_job_dir(&item.id),
            PendingKind::Upload => state.storage.delete_upload(&item.id),
        };
        match result {
            Ok(()) => {
                tracing::info!(id = %item.id, kind = ?item.kind, "borrado pendiente reintentado OK");
                let mut events = state.events.write().await;
                events.remove(&item.id);
                let mut jobs = state.jobs.write().await;
                jobs.remove(&item.id);
            }
            Err(e) => {
                tracing::error!(id = %item.id, kind = ?item.kind, error = %e, "borrado pendiente sigue fallando");
                remaining.push(item);
            }
        }
    }
    state.storage.save_pending_deletes(&remaining)?;
    Ok(())
}

/// Barrido periódico: jobs vencidos, uploads, tmp y pendientes.
pub async fn retention_sweep(state: &AppState) {
    let retention_hours = state.retention_hours;
    let upload_ttl = std::env::var("ZEDAZO_UPLOAD_TTL_HOURS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_UPLOAD_TTL_HOURS);

    let mut to_expire = Vec::new();
    {
        let jobs = state.jobs.read().await;
        for (id, rec) in jobs.iter() {
            if should_expire(&rec.manifest, retention_hours, jiff::Timestamp::now()) {
                to_expire.push(id.clone());
            }
        }
    }

    for id in to_expire {
        expire_job(state, &id).await;
    }

    // Uploads referenciados por jobs vivos.
    let referenced: std::collections::HashSet<String> = {
        let jobs = state.jobs.read().await;
        jobs.values()
            .map(|j| j.manifest.input.upload_id.clone())
            .collect()
    };
    let _ = sweep_uploads(state, upload_ttl, &referenced);
    let _ = sweep_tmp(state);
    let _ = retry_pending_deletes(state).await;
}

async fn expire_job(state: &AppState, id: &str) {
    {
        let mut jobs = state.jobs.write().await;
        if let Some(rec) = jobs.get_mut(id) {
            rec.manifest.status = JobStatus::Expired;
            let _ = crate::jobs::save_manifest_result(state, &rec.manifest);
        }
    }
    match state.storage.delete_job_dir(id) {
        Ok(()) => {
            {
                let mut jobs = state.jobs.write().await;
                jobs.remove(id);
            }
            {
                let mut events = state.events.write().await;
                events.remove(id);
            }
            tracing::info!(job_id = %id, "job expirado por retención");
        }
        Err(e) => {
            tracing::error!(job_id = %id, error = %e, "fallo al borrar job expirado; se reintentará");
            let _ = state.storage.enqueue_pending_delete(PendingKind::Job, id);
            // Mantener entrada en registro para reintento visible.
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeleteOutcome {
    Deleted,
    /// Ya no existía (segundo DELETE o wipe previo).
    Idempotent,
    NotFound,
    /// Borrado de FS falló; queda en pending_deletes.
    PendingRetry,
}

/// Elimina un job del seguimiento y del disco; si el borrado falla, queda pendiente.
pub async fn delete_job_tracked(state: &AppState, job_id: &str) -> DeleteOutcome {
    if Storage::validate_id(job_id).is_err() {
        return DeleteOutcome::NotFound;
    }
    let in_memory = {
        let mut jobs = state.jobs.write().await;
        if let Some(rec) = jobs.get_mut(job_id) {
            rec.cancel.cancel();
            rec.manifest.status = JobStatus::Deleted;
            true
        } else {
            false
        }
    };
    let on_disk = state.storage.job_dir(job_id).exists();
    let in_pending = state
        .storage
        .load_pending_deletes()
        .iter()
        .any(|p| p.id == job_id && matches!(p.kind, PendingKind::Job | PendingKind::JobDir));

    {
        let deleted = state.deleted_ids.read().await;
        if !in_memory && !on_disk && !in_pending && deleted.contains(job_id) {
            return DeleteOutcome::Idempotent;
        }
    }
    if !in_memory && !on_disk && !in_pending {
        return DeleteOutcome::NotFound;
    }
    if !in_memory && !on_disk && in_pending {
        return DeleteOutcome::Idempotent;
    }

    match state.storage.delete_job_dir(job_id) {
        Ok(()) => {
            {
                let mut jobs = state.jobs.write().await;
                jobs.remove(job_id);
            }
            {
                let mut events = state.events.write().await;
                events.remove(job_id);
            }
            {
                let mut deleted = state.deleted_ids.write().await;
                deleted.insert(job_id.to_string());
            }
            DeleteOutcome::Deleted
        }
        Err(e) => {
            tracing::error!(job_id = %job_id, error = %e, "delete_job: borrado falló");
            let _ = state
                .storage
                .enqueue_pending_delete(PendingKind::Job, job_id);
            {
                let mut jobs = state.jobs.write().await;
                jobs.remove(job_id);
            }
            {
                let mut deleted = state.deleted_ids.write().await;
                deleted.insert(job_id.to_string());
            }
            DeleteOutcome::PendingRetry
        }
    }
}

#[cfg(test)]
mod should_expire_tests {
    use super::*;
    use crate::dto::{InputMeta, RulesMeta};

    fn completed_manifest(completed_at: &str, retention_hours: u64) -> JobManifest {
        JobManifest {
            job_id: "01900000-0000-7000-8000-0000000000ee".into(),
            status: JobStatus::Completed,
            display_name: None,
            created_at: completed_at.into(),
            started_at: Some(completed_at.into()),
            completed_at: Some(completed_at.into()),
            core_version: "0.5.1".into(),
            input: InputMeta {
                original_name: "x.vcf".into(),
                sha256: "abc".into(),
                bytes: 1,
                source_detected: None,
                vcard_version: None,
                upload_id: "00000000-0000-7000-8000-000000000001".into(),
            },
            rules: RulesMeta {
                mode: "builtin".into(),
                sha256: None,
            },
            summary: None,
            artifacts: vec![],
            error: None,
            retention_hours,
            config_toml: None,
            persisted_at: None,
            persisted: false,
            retryable: false,
        }
    }

    #[test]
    fn expires_when_age_reaches_retention() {
        let completed = "2026-01-01T00:00:00Z";
        let manifest = completed_manifest(completed, 24);
        let now = "2026-01-02T00:00:00Z".parse::<jiff::Timestamp>().unwrap();
        assert!(should_expire(&manifest, 24, now));
    }

    #[test]
    fn not_expired_before_retention() {
        let completed = "2026-01-01T00:00:00Z";
        let manifest = completed_manifest(completed, 24);
        let now = "2026-01-01T12:00:00Z".parse::<jiff::Timestamp>().unwrap();
        assert!(!should_expire(&manifest, 24, now));
    }

    #[test]
    fn uses_default_when_manifest_retention_zero() {
        let completed = "2026-01-01T00:00:00Z";
        let manifest = completed_manifest(completed, 0);
        let now = "2026-01-03T00:00:00Z".parse::<jiff::Timestamp>().unwrap();
        assert!(should_expire(&manifest, 48, now));
        assert!(!should_expire(
            &manifest,
            72,
            "2026-01-02T00:00:00Z".parse::<jiff::Timestamp>().unwrap()
        ));
    }
}
