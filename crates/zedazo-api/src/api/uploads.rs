//! Uploads multipart (S-04): chunks, tope, BEGIN:VCARD, semáforo.

use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::Json;
use axum::Router;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::app::AppState;
use crate::dto::UploadResponse;
use crate::storage::Storage;

/// Router de uploads con `DefaultBodyLimit` = max + 64 KiB (fuera del layer estrecho).
pub fn uploads_router(max_upload_bytes: usize) -> Router<AppState> {
    Router::new()
        .route("/api/v1/uploads", post(create_upload))
        .layer(DefaultBodyLimit::max(
            max_upload_bytes.saturating_add(64 * 1024),
        ))
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
    let mut size: u64 = 0;
    let mut sha = String::new();
    let upload_id = Uuid::now_v7().to_string();

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

        let path = state.storage.upload_path(&upload_id);
        state
            .storage
            .assert_under_root(&path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if path.exists() {
            return Err((StatusCode::CONFLICT, "upload_id ya existe".into()));
        }

        let mut file = tokio::fs::File::create(&path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let mut hasher = Sha256::new();
        let mut total: usize = 0;
        let mut first = true;
        let mut begin_ok = false;

        loop {
            let chunk = field
                .chunk()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            let Some(chunk) = chunk else {
                break;
            };
            if first {
                first = false;
                let sample = String::from_utf8_lossy(&chunk[..chunk.len().min(512)]);
                begin_ok = sample.contains("BEGIN:VCARD") || sample.contains("begin:vcard");
            }
            total = total.saturating_add(chunk.len());
            if total > state.max_upload_bytes {
                drop(file);
                let _ = tokio::fs::remove_file(&path).await;
                return Err((
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "upload demasiado grande".into(),
                ));
            }
            hasher.update(&chunk);
            file.write_all(&chunk)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }

        if !begin_ok {
            drop(file);
            let _ = tokio::fs::remove_file(&path).await;
            return Err((StatusCode::BAD_REQUEST, "contenido no parece VCF".into()));
        }

        file.flush()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }

        size = total as u64;
        sha = hex::encode(hasher.finalize());
        let meta = serde_json::json!({
            "created_at": chrono::Utc::now().to_rfc3339(),
            "bytes": size,
            "sha256": sha,
        });
        let _ = crate::storage::Storage::atomic_write(
            &state.storage.upload_meta_path(&upload_id),
            meta.to_string().as_bytes(),
        );
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
