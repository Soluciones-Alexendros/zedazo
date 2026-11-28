use serde::{Deserialize, Serialize};
use zedazo_core::application::process::{
    ArtifactKind, ContactView, DuplicateGroupView, ProcessSummary, ProcessWarning,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Created,
    Uploading,
    Uploaded,
    Queued,
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
    CancelRequested,
    Cancelled,
    Expired,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobManifest {
    pub job_id: String,
    pub status: JobStatus,
    pub display_name: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub core_version: String,
    pub input: InputMeta,
    pub rules: RulesMeta,
    pub summary: Option<ProcessSummary>,
    pub artifacts: Vec<String>,
    pub error: Option<String>,
    pub retention_hours: u64,
    /// Config TOML inline usada para este job (para reconstrucción en reconcile).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_toml: Option<String>,
    /// Timestamp de confirmación de persistencia (cuando se escribió manifiesto + vistas en disco).
    /// Solo presente si `persisted = true` y status = Completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persisted_at: Option<String>,
    /// Indica si el resultado se confirmó en disco (manifiesto + vistas).
    /// `Completed` implica `persisted = true`; `Failed` con `persisted = false` = fallo de persistencia.
    #[serde(default)]
    pub persisted: bool,
    /// Si el fallo es reintentable (ej. disco lleno, permisos). Solo relevante si `persisted = false`.
    #[serde(default)]
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMeta {
    pub original_name: String,
    pub sha256: String,
    pub bytes: u64,
    pub source_detected: Option<String>,
    pub vcard_version: Option<String>,
    pub upload_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesMeta {
    pub mode: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEvent {
    pub id: u64,
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct JobRecord {
    pub manifest: JobManifest,
    pub contact_views: Vec<ContactView>,
    pub duplicate_groups: Vec<DuplicateGroupView>,
    pub warnings: Vec<ProcessWarning>,
    pub cancel: zedazo_core::application::process::CancellationToken,
    pub requested_artifacts: Vec<ArtifactKind>,
    pub config_toml: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub upload_id: String,
    pub display_name: Option<String>,
    pub rules: Option<RulesRequest>,
    pub artifacts: Option<Vec<String>>,
    pub retention_hours: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct RulesRequest {
    pub mode: Option<String>,
    pub toml: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub api_version: String,
    pub core_version: String,
    pub storage_mode: String,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: String,
    pub git_sha: String,
    pub build_date: String,
    pub schema_version: String,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub upload_id: String,
    pub original_name: String,
    pub bytes: u64,
    pub sha256: String,
}

pub fn parse_artifact_kind(s: &str) -> Option<ArtifactKind> {
    match s {
        "vcf" => Some(ArtifactKind::Vcf),
        "audit_tsv" => Some(ArtifactKind::AuditTsv),
        "stats_json" => Some(ArtifactKind::StatsJson),
        "stats_markdown" => Some(ArtifactKind::StatsMarkdown),
        "csv" => Some(ArtifactKind::Csv),
        "json" => Some(ArtifactKind::Json),
        _ => None,
    }
}

pub fn artifact_kind_str(k: ArtifactKind) -> &'static str {
    match k {
        ArtifactKind::Vcf => "vcf",
        ArtifactKind::AuditTsv => "audit_tsv",
        ArtifactKind::StatsJson => "stats_json",
        ArtifactKind::StatsMarkdown => "stats_markdown",
        ArtifactKind::Csv => "csv",
        ArtifactKind::Json => "json",
    }
}
