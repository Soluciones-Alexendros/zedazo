//! Contrato de procesamiento compartido CLI/API (ADR-0015).

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::application::cribar;
use crate::application::stats::Stats;
use crate::domain::audit::AuditEntry;
use crate::domain::contact::Contact;
use crate::domain::screening::ScreeningDecision;
use crate::error::CribaError;
use crate::infrastructure::csv_writer::export_csv;
use crate::infrastructure::json_writer::export_json;
use crate::infrastructure::secure_fs::{self, WriteOpts};

/// Identificador opaco de trabajo (ULID/UUID string).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Vcf,
    AuditTsv,
    StatsJson,
    StatsMarkdown,
    Csv,
    Json,
}

#[derive(Debug, Clone)]
pub enum RulesConfigSource {
    BuiltIn,
    TomlPath(PathBuf),
    TomlInline(String),
}

/// Señal cooperativa de cancelación (sin Tokio en el core).
#[derive(Debug, Default, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Flag interno compartible con [`crate::domain::identity::DedupCtrl`].
    pub fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancelled)
    }

    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "procesamiento cancelado")
    }
}

impl std::error::Error for Cancelled {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessPhase {
    Validating,
    Parsing,
    Screening,
    Normalizing,
    Classifying,
    Deduplicating,
    Verifying,
    WritingArtifacts,
    Completed,
    Failed,
    Cancelled,
}

pub trait ProgressReporter: Send + Sync {
    fn phase(&self, phase: ProcessPhase);
    fn metric(&self, name: &str, value: u64);
    fn message(&self, msg: &str);
}

#[derive(Debug, Default)]
pub struct NullProgress;

impl ProgressReporter for NullProgress {
    fn phase(&self, _phase: ProcessPhase) {}
    fn metric(&self, _name: &str, _value: u64) {}
    fn message(&self, _msg: &str) {}
}

#[derive(Debug, Default)]
pub struct TracingProgress;

impl ProgressReporter for TracingProgress {
    fn phase(&self, phase: ProcessPhase) {
        tracing::info!(?phase, "fase");
    }
    fn metric(&self, name: &str, value: u64) {
        tracing::info!(%name, value, "métrica");
    }
    fn message(&self, msg: &str) {
        tracing::info!("{msg}");
    }
}

pub struct ProcessRequest {
    pub job_id: JobId,
    pub input_path: PathBuf,
    pub working_directory: PathBuf,
    pub config: RulesConfigSource,
    pub requested_artifacts: BTreeSet<ArtifactKind>,
    pub source_override: String,
    pub dry_run: bool,
    pub strict: bool,
    pub cancellation: CancellationToken,
    pub progress: Arc<dyn ProgressReporter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSummary {
    pub input_contacts: usize,
    pub retained: usize,
    pub needs_review: usize,
    pub eliminated: usize,
    pub quarantine: usize,
    pub duplicate_groups: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDescriptor {
    pub kind: ArtifactKind,
    pub path: PathBuf,
    pub bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactView {
    pub uid: String,
    pub fn_value: String,
    pub result: String,
    pub screening_rule: Option<String>,
    pub categories: Vec<String>,
    pub emails: Vec<String>,
    pub tels: Vec<String>,
    pub merged_uids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroupView {
    pub canonical_uid: String,
    pub member_uids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventView {
    pub uid: String,
    pub action: String,
    pub rule: String,
    pub evidence: String,
}

#[derive(Debug)]
pub struct ProcessResult {
    pub summary: ProcessSummary,
    pub stats: Stats,
    pub contacts: Vec<Contact>,
    pub contact_views: Vec<ContactView>,
    pub duplicate_groups: Vec<DuplicateGroupView>,
    pub audit_events: Vec<AuditEventView>,
    pub audit_entries: Vec<AuditEntry>,
    pub artifacts: Vec<ArtifactDescriptor>,
    pub warnings: Vec<ProcessWarning>,
}

impl ContactView {
    pub fn from_contact(c: &Contact) -> Self {
        let result = match &c.decision {
            crate::domain::screening::ScreeningDecision::Conserved => "conserved",
            crate::domain::screening::ScreeningDecision::Eliminated(_) => "eliminated",
            crate::domain::screening::ScreeningDecision::NeedsReview(_) => "needs_review",
            crate::domain::screening::ScreeningDecision::Quarantine(_) => "quarantine",
        }
        .to_string();
        let mut categories: Vec<String> = c.categories.n1.iter().cloned().collect();
        categories.extend(c.categories.n2.iter().cloned());
        categories.extend(c.categories.n3.iter().cloned());
        categories.sort();
        Self {
            uid: c.uid.clone(),
            fn_value: c.fn_value.clone(),
            result,
            screening_rule: if c.screening_rule.is_empty() {
                None
            } else {
                Some(c.screening_rule.clone())
            },
            categories,
            emails: c.emails.iter().map(|e| e.value.clone()).collect(),
            tels: c.tels.iter().map(|t| t.value.clone()).collect(),
            merged_uids: c.merged_uids.clone(),
        }
    }
}

impl AuditEventView {
    pub fn from_entry(e: &AuditEntry) -> Self {
        let action = match e.action {
            crate::domain::audit::AuditAction::Conserved => "conserved",
            crate::domain::audit::AuditAction::Eliminated => "eliminated",
            crate::domain::audit::AuditAction::Quarantine => "quarantine",
            crate::domain::audit::AuditAction::NeedsReview => "needs_review",
            crate::domain::audit::AuditAction::Merged => "merged",
        };
        Self {
            uid: e.uid.clone(),
            action: action.to_string(),
            rule: e.rule.clone(),
            evidence: e.reason.clone(),
        }
    }
}

fn cancelled_err() -> CribaError {
    CribaError::Io(std::io::Error::new(
        std::io::ErrorKind::Interrupted,
        "cancelado",
    ))
}

/// Ejecuta el pipeline vía cribar y materializa artefactos solicitados en `working_directory`.
pub fn run(req: &ProcessRequest) -> Result<ProcessResult, CribaError> {
    req.cancellation.check().map_err(|_| cancelled_err())?;
    req.progress.phase(ProcessPhase::Validating);
    secure_fs::ensure_dir_0700(&req.working_directory)?;

    let config_path = match &req.config {
        RulesConfigSource::BuiltIn => None,
        RulesConfigSource::TomlPath(p) => Some(p.clone()),
        RulesConfigSource::TomlInline(s) => {
            let p = req.working_directory.join("config.toml");
            secure_fs::write_bytes(&p, s.as_bytes(), WriteOpts::new(true))?;
            Some(p)
        }
    };

    req.progress.phase(ProcessPhase::Parsing);
    req.cancellation.check().map_err(|_| cancelled_err())?;

    let out_vcf = req.working_directory.join("output.vcf");
    let audit_path = req.working_directory.join("audit.tsv");
    let want_vcf = req.requested_artifacts.contains(&ArtifactKind::Vcf);
    let want_audit = req.requested_artifacts.contains(&ArtifactKind::AuditTsv);

    let timeout_secs = std::env::var("ZEDAZO_JOB_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|&s| s > 0);
    let dedup_ctrl = crate::domain::identity::DedupCtrl {
        cancelled: Some(req.cancellation.flag()),
        deadline: timeout_secs
            .map(|s| std::time::Instant::now() + std::time::Duration::from_secs(s)),
    };

    let (stats, contacts) = cribar::execute_with_dedup(
        &req.input_path,
        if want_vcf && !req.dry_run {
            Some(out_vcf.as_path())
        } else {
            None
        },
        if want_audit && !req.dry_run {
            Some(audit_path.as_path())
        } else {
            None
        },
        config_path.as_deref(),
        &req.source_override,
        req.dry_run,
        req.strict,
        false,
        &dedup_ctrl,
    )?;

    req.progress.phase(ProcessPhase::WritingArtifacts);
    req.progress.metric("contacts", contacts.len() as u64);

    let mut warnings = Vec::new();
    if let Some(ref p) = config_path {
        if let Ok(cfg) = crate::infrastructure::config::load_config(Some(p)) {
            for w in cfg.warnings {
                warnings.push(ProcessWarning {
                    code: w.code,
                    message: w.message,
                });
            }
        }
    }
    let mut artifacts = Vec::new();
    let audit_entries: Vec<AuditEntry> = Vec::new();
    if audit_path.exists() {
        artifacts.push(ArtifactDescriptor {
            kind: ArtifactKind::AuditTsv,
            path: audit_path.clone(),
            bytes: fs::metadata(&audit_path).ok().map(|m| m.len()),
        });
    }
    if out_vcf.exists() {
        artifacts.push(ArtifactDescriptor {
            kind: ArtifactKind::Vcf,
            path: out_vcf.clone(),
            bytes: fs::metadata(&out_vcf).ok().map(|m| m.len()),
        });
    }

    let artifact_opts = WriteOpts::new(false).protect(&req.input_path);
    if !req.dry_run && req.requested_artifacts.contains(&ArtifactKind::Csv) {
        let p = req.working_directory.join("contacts.csv");
        export_csv(&contacts, &p, artifact_opts)?;
        artifacts.push(ArtifactDescriptor {
            kind: ArtifactKind::Csv,
            path: p,
            bytes: None,
        });
    }
    if !req.dry_run && req.requested_artifacts.contains(&ArtifactKind::Json) {
        let p = req.working_directory.join("contacts.json");
        export_json(&contacts, &p, artifact_opts)?;
        artifacts.push(ArtifactDescriptor {
            kind: ArtifactKind::Json,
            path: p,
            bytes: None,
        });
    }
    if !req.dry_run && req.requested_artifacts.contains(&ArtifactKind::StatsJson) {
        let p = req.working_directory.join("stats.json");
        secure_fs::write_bytes(&p, stats.to_json()?.as_bytes(), artifact_opts)?;
        artifacts.push(ArtifactDescriptor {
            kind: ArtifactKind::StatsJson,
            path: p,
            bytes: None,
        });
    }
    if !req.dry_run
        && req
            .requested_artifacts
            .contains(&ArtifactKind::StatsMarkdown)
    {
        let p = req.working_directory.join("stats.md");
        secure_fs::write_bytes(&p, stats.to_markdown().as_bytes(), artifact_opts)?;
        artifacts.push(ArtifactDescriptor {
            kind: ArtifactKind::StatsMarkdown,
            path: p,
            bytes: None,
        });
    }

    let retained = contacts
        .iter()
        .filter(|c| matches!(c.decision, ScreeningDecision::Conserved))
        .count();
    let needs_review = contacts
        .iter()
        .filter(|c| matches!(c.decision, ScreeningDecision::NeedsReview(_)))
        .count();
    let eliminated = contacts
        .iter()
        .filter(|c| matches!(c.decision, ScreeningDecision::Eliminated(_)))
        .count();
    let quarantine = contacts
        .iter()
        .filter(|c| matches!(c.decision, ScreeningDecision::Quarantine(_)))
        .count();
    let duplicate_groups = contacts
        .iter()
        .filter(|c| !c.merged_uids.is_empty())
        .count();

    let contact_views: Vec<ContactView> = contacts.iter().map(ContactView::from_contact).collect();
    let duplicate_groups_view: Vec<DuplicateGroupView> = contacts
        .iter()
        .filter(|c| !c.merged_uids.is_empty())
        .map(|c| {
            let mut members = c.merged_uids.clone();
            members.push(c.uid.clone());
            DuplicateGroupView {
                canonical_uid: c.uid.clone(),
                member_uids: members,
            }
        })
        .collect();

    req.progress.phase(ProcessPhase::Completed);

    Ok(ProcessResult {
        summary: ProcessSummary {
            input_contacts: stats.total_entrada,
            retained,
            needs_review,
            eliminated,
            quarantine,
            duplicate_groups,
        },
        stats,
        contacts,
        contact_views,
        duplicate_groups: duplicate_groups_view,
        audit_events: audit_entries
            .iter()
            .map(AuditEventView::from_entry)
            .collect(),
        audit_entries,
        artifacts,
        warnings,
    })
}
