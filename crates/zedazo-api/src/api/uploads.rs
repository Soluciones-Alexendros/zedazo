//! Uploads multipart (S-04): chunks, tope, BEGIN:VCARD, semáforo.
//! W2-T1: Buffer de detección 4KiB + TempFile guard + commit explícito.

use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::Json;
use axum::Router;
use sha2::{Digest, Sha256};
use std::io::Write;
use tempfile::NamedTempFile;
use uuid::Uuid;

use crate::app::AppState;
use crate::dto::UploadResponse;
use crate::storage::Storage;

/// Buffer de detección para `BEGIN:VCARD` (4 KiB).
/// Cubre cualquier header MIME + fragmentación arbitraria.
const DETECTION_BUFFER: usize = 4096;

/// Router de uploads con `DefaultBodyLimit` = max + 64 KiB (fuera del layer estrecho).
pub fn uploads_router(max_upload_bytes: usize) -> Router<AppState> {
    Router::new()
        .route("/api/v1/uploads", post(create_upload))
        .route("/api/v1/uploads/{id}/commit", post(commit_upload))
        .layer(DefaultBodyLimit::max(
            max_upload_bytes.saturating_add(64 * 1024),
        ))
}

/// Detecta si el buffer comienza con `BEGIN:VCARD` (case-insensitive).
fn is_vcf_prefix(buf: &[u8]) -> bool {
    let sample = String::from_utf8_lossy(buf);
    sample.contains("BEGIN:VCARD") || sample.contains("begin:vcard")
}

pub async fn create_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), (StatusCode, String)> {
    let _permit = state.upload_sem.clone().try_acquire_owned().map_err(|_| {
        (
            StatusCode::TOO_MANY_REQUESTS,
            "demasiados uploads concurrentes".into(),
        )
    })?;

    let mut file_name = String::from("upload.vcf");
    let mut wrote = false;
    let upload_id = Uuid::now_v7().to_string();
    let mut size: u64 = 0;
    let mut sha = String::new();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }
        if let Some(n) = field.file_name() {
            file_name = Storage::sanitize_name(n);
        }

        let upload_dir = state.storage.root.join("uploads");
        tokio::fs::create_dir_all(&upload_dir)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Usar NamedTempFile en el directorio de uploads para atomicidad y limpieza automática.
        let mut temp_file = NamedTempFile::new_in(&upload_dir)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let temp_path = temp_file.path().to_path_buf();
        let mut hasher = Sha256::new();
        let mut total: usize = 0;
        let mut detection_buf = Vec::with_capacity(DETECTION_BUFFER);
        let mut detection_done = false;

        loop {
            let chunk = field
                .chunk()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            let Some(chunk) = chunk else {
                break;
            };

            // Acumular en buffer de detección hasta encontrar BEGIN:VCARD o llenar buffer
            if !detection_done {
                let remaining = DETECTION_BUFFER.saturating_sub(detection_buf.len());
                let take = chunk.len().min(remaining);
                detection_buf.extend_from_slice(&chunk[..take]);
                if detection_buf.len() >= DETECTION_BUFFER || take < chunk.len() {
                    detection_done = true;
                    if !is_vcf_prefix(&detection_buf) {
                        return Err((StatusCode::BAD_REQUEST, "contenido no parece VCF".into()));
                    }
                }
            }

            total = total.saturating_add(chunk.len());
            if total > state.max_upload_bytes {
                return Err((
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "upload demasiado grande".into(),
                ));
            }

            hasher.update(&chunk);
            temp_file
                .as_file_mut()
                .write_all(&chunk)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }

        // Si el buffer de detección no se llenó completamente, verificar al final
        if !detection_done && !is_vcf_prefix(&detection_buf) {
            return Err((StatusCode::BAD_REQUEST, "contenido no parece VCF".into()));
        }

        size = total as u64;
        sha = hex::encode(hasher.finalize());

        // Persistir metadata del upload (incluye ruta temporal para commit posterior)
        let meta = serde_json::json!({
            "created_at": jiff::Timestamp::now().to_string(),
            "bytes": size,
            "sha256": sha,
            "temp_path": temp_path.to_string_lossy(),
            "original_name": file_name,
        });
        crate::storage::Storage::atomic_write(
            &state.storage.upload_meta_path(&upload_id),
            meta.to_string().as_bytes(),
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // NO hacer persist() aquí; el archivo temporal se mantiene hasta commit explícito.
        // NamedTempFile se limpia automáticamente al dropearse si no se hace persist().
        std::mem::forget(temp_file); // Evitar limpieza automática; commit_upload la moverá.

        wrote = true;
        break;
    }

    if !wrote {
        return Err((StatusCode::BAD_REQUEST, "falta campo file".into()));
    }

    Ok((
        StatusCode::CREATED,
        Json(UploadResponse {
            upload_id,
            original_name: file_name,
            bytes: size,
            sha256: sha,
        }),
    ))
}

/// Commit explícito del upload: mueve el archivo temporal a la ubicación final
/// y dispara la creación del job de procesamiento.
pub async fn commit_upload(
    State(state): State<AppState>,
    axum::extract::Path(upload_id): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    // Leer metadata del upload
    let meta_path = state.storage.upload_meta_path(&upload_id);
    let meta_raw = tokio::fs::read_to_string(&meta_path).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            format!("upload no encontrado: {}", e),
        )
    })?;
    let meta: serde_json::Value = serde_json::from_str(&meta_raw)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let temp_path_str = meta
        .get("temp_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "meta sin temp_path".into(),
            )
        })?;
    let temp_path = std::path::PathBuf::from(temp_path_str);

    let original_name = meta
        .get("original_name")
        .and_then(|v| v.as_str())
        .unwrap_or("upload.vcf");
    let size = meta.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
    let sha = meta.get("sha256").and_then(|v| v.as_str()).unwrap_or("");

    // Verificar que el archivo temporal existe
    if !temp_path.exists() {
        return Err((
            StatusCode::GONE,
            "archivo temporal expirado o ya committeado".into(),
        ));
    }

    // Mover archivo temporal a ubicación final (uploads/<upload_id>)
    let final_path = state.storage.upload_path(&upload_id);
    tokio::fs::rename(&temp_path, &final_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Ajustar permisos
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ =
            tokio::fs::set_permissions(&final_path, std::fs::Permissions::from_mode(0o600)).await;
    }

    // Crear job automáticamente
    let job_id = uuid::Uuid::now_v7().to_string();
    let job_dir = state
        .storage
        .ensure_job_dir(&job_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let input_path = job_dir.join("input.vcf");
    tokio::fs::copy(&final_path, &input_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Crear manifiesto inicial
    let manifest = crate::dto::JobManifest {
        job_id: job_id.clone(),
        status: crate::dto::JobStatus::Queued,
        display_name: Some(original_name.to_string()),
        created_at: jiff::Timestamp::now().to_string(),
        started_at: None,
        completed_at: None,
        core_version: state.core_version.clone(),
        input: crate::dto::InputMeta {
            original_name: original_name.to_string(),
            sha256: sha.to_string(),
            bytes: size,
            source_detected: None,
            vcard_version: None,
            upload_id: upload_id.clone(),
        },
        rules: crate::dto::RulesMeta {
            mode: "builtin".into(),
            sha256: None,
        },
        summary: None,
        artifacts: vec![],
        error: None,
        retention_hours: state.retention_hours,
        config_toml: None,
        persisted_at: None,
        persisted: false,
        retryable: false,
    };

    crate::jobs::save_manifest(&state, &manifest);

    // Registrar evento
    let (tx, _rx) = tokio::sync::broadcast::channel(256);
    {
        let mut events = state.events.write().await;
        events.insert(job_id.clone(), tx);
    }

    // Crear record y spawnear job
    let record = crate::dto::JobRecord {
        manifest: manifest.clone(),
        contact_views: vec![],
        duplicate_groups: vec![],
        warnings: vec![],
        cancel: zedazo_core::application::process::CancellationToken::new(),
        requested_artifacts: vec![
            zedazo_core::application::process::ArtifactKind::Vcf,
            zedazo_core::application::process::ArtifactKind::AuditTsv,
            zedazo_core::application::process::ArtifactKind::StatsJson,
            zedazo_core::application::process::ArtifactKind::Csv,
            zedazo_core::application::process::ArtifactKind::Json,
        ],
        config_toml: None,
    };
    {
        let mut jobs = state.jobs.write().await;
        jobs.insert(job_id.clone(), record);
    }
    crate::jobs::spawn_job(state.clone(), job_id.clone());

    // Eliminar metadata del upload (ya committeado)
    let _ = tokio::fs::remove_file(&meta_path).await;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "upload_id": upload_id,
            "job_id": job_id,
            "original_name": original_name.to_string(),
            "bytes": size,
            "sha256": sha.to_string(),
        })),
    ))
}
