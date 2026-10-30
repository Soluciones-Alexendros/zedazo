use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;
use zedazo_core::application::process::CancellationToken;
use zedazo_core::infrastructure::config::load_config;

use crate::app::AppState;
use crate::dto::{InputMeta, JobManifest, JobRecord, JobStatus, RulesMeta};
use crate::jobs::{now_rfc3339, save_manifest};

#[derive(Deserialize)]
pub struct AuditBody {
    pub upload_id: String,
    pub config_toml: Option<String>,
}

pub async fn get_audit(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = state.storage.job_dir(&job_id).join("audit.tsv");
    if !path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }
    let text = std::fs::read_to_string(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut items = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            continue;
        }
        let cols: Vec<_> = line.split('\t').collect();
        items.push(serde_json::json!({ "cols": cols }));
    }
    Ok(Json(serde_json::json!({ "items": items })))
}

pub async fn create_audit(
    State(state): State<AppState>,
    Json(body): Json<AuditBody>,
) -> Result<(StatusCode, Json<JobManifest>), (StatusCode, String)> {
    let upload_path = state.storage.upload_path(&body.upload_id);
    if !upload_path.exists() {
        return Err((StatusCode::NOT_FOUND, "upload desconocido".into()));
    }
    let job_id = Uuid::now_v7().to_string();
    let meta = std::fs::metadata(&upload_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let config_toml = body.config_toml.clone();
    let manifest = JobManifest {
        job_id: job_id.clone(),
        status: JobStatus::Queued,
        display_name: Some("audit".into()),
        created_at: now_rfc3339(),
        started_at: None,
        completed_at: None,
        core_version: state.core_version.clone(),
        input: InputMeta {
            original_name: "audit.vcf".into(),
            sha256: String::new(),
            bytes: meta.len(),
            source_detected: None,
            vcard_version: None,
            upload_id: body.upload_id.clone(),
        },
        rules: RulesMeta {
            mode: if config_toml.is_some() {
                "toml_inline".into()
            } else {
                "builtin".into()
            },
            sha256: None,
        },
        summary: None,
        artifacts: vec![],
        error: None,
        retention_hours: state.retention_hours,
    };
    state
        .storage
        .ensure_job_dir(&job_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let job_dir = state.storage.job_dir(&job_id);
    let input = job_dir.join("input.vcf");
    zedazo_core::infrastructure::secure_fs::copy_file(
        &upload_path,
        &input,
        zedazo_core::infrastructure::secure_fs::WriteOpts::new(false),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let audit_out = job_dir.join("audit.tsv");
    let config_path = if let Some(ref toml) = config_toml {
        let p = job_dir.join("config.toml");
        zedazo_core::infrastructure::secure_fs::write_bytes(
            &p,
            toml.as_bytes(),
            zedazo_core::infrastructure::secure_fs::WriteOpts::new(true),
        )
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        let _ = load_config(Some(&p)).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        Some(p)
    } else {
        None
    };

    let input_c = input.clone();
    let audit_c = audit_out.clone();
    let cfg = config_path.clone();
    tokio::task::spawn_blocking(move || {
        zedazo_core::application::audit::execute(&input_c, Some(&audit_c), cfg.as_deref(), "auto")
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut manifest = manifest;
    manifest.status = JobStatus::Completed;
    manifest.completed_at = Some(now_rfc3339());
    manifest.artifacts = vec!["audit_tsv".into()];
    save_manifest(&state, &manifest);

    let record = JobRecord {
        manifest: manifest.clone(),
        contact_views: vec![],
        duplicate_groups: vec![],
        warnings: vec![],
        cancel: CancellationToken::new(),
        requested_artifacts: vec![zedazo_core::application::process::ArtifactKind::AuditTsv],
        config_toml,
    };
    state.jobs.write().await.insert(job_id, record);
    Ok((StatusCode::CREATED, Json(manifest)))
}
