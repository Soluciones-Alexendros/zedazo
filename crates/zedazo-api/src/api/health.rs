use axum::extract::State;
use axum::Json;

use crate::app::AppState;
use crate::dto::{HealthResponse, VersionResponse};

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        api_version: state.api_version.clone(),
        core_version: state.core_version.clone(),
        storage_mode: std::env::var("ZEDAZO_STORAGE_MODE").unwrap_or_else(|_| "ephemeral".into()),
    })
}

pub async fn version(State(state): State<AppState>) -> Json<VersionResponse> {
    // Identidad del binario: embebida en compilación (build.rs). No leer env en runtime
    // — eso permitiría que smoke/run-env.sh acrediten un SHA ajeno al artefacto.
    Json(VersionResponse {
        version: state.core_version.clone(),
        git_sha: option_env!("ZEDAZO_GIT_SHA")
            .unwrap_or("unknown")
            .to_string(),
        build_date: option_env!("ZEDAZO_BUILD_DATE")
            .unwrap_or("unknown")
            .to_string(),
        schema_version: "1".into(),
    })
}
