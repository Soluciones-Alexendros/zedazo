use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::app::AppState;

/// Borra todos los datos locales (jobs, uploads, tmp). Single-user local.
pub async fn wipe_all(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    // Subir época antes de cancelar para que workers abandonen escrituras.
    let epoch = state
        .storage
        .bump_wipe_epoch()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state
        .wipe_epoch
        .store(epoch, std::sync::atomic::Ordering::SeqCst);

    {
        let mut jobs = state.jobs.write().await;
        for rec in jobs.values_mut() {
            rec.cancel.cancel();
        }
        jobs.clear();
    }
    {
        let mut events = state.events.write().await;
        events.clear();
    }
    {
        let mut deleted = state.deleted_ids.write().await;
        deleted.clear();
    }
    state
        .storage
        .wipe_all()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "wiped": true,
            "jobs": true,
            "uploads": true,
            "tmp": true,
            "wipe_epoch": epoch
        })),
    ))
}
