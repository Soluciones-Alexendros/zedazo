use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tokio::sync::{broadcast, RwLock};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::api;
use crate::auth::{self, AuthConfig};
use crate::jobs::{JobEvent, JobRecord};
use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
    pub max_upload_bytes: usize,
    /// Máximo de jobs no terminales concurrentes (0 = sin límite explícito).
    pub max_concurrent_jobs: usize,
    pub retention_hours: u64,
    pub jobs: Arc<RwLock<HashMap<String, JobRecord>>>,
    pub events: Arc<RwLock<HashMap<String, broadcast::Sender<JobEvent>>>>,
    pub core_version: String,
    pub api_version: String,
    pub auth: AuthConfig,
    /// Época de wipe: el worker abandona escrituras si no coincide.
    pub wipe_epoch: Arc<AtomicU64>,
    /// Tombstones en memoria para DELETE idempotente en la misma vida del proceso.
    pub deleted_ids: Arc<RwLock<HashSet<String>>>,
}

impl AppState {
    pub fn from_env() -> anyhow::Result<Self> {
        let data_dir =
            PathBuf::from(std::env::var("ZEDAZO_DATA_DIR").unwrap_or_else(|_| "./data".into()));
        let max_upload_bytes: usize = std::env::var("ZEDAZO_MAX_UPLOAD_BYTES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(52_428_800);
        let max_concurrent_jobs: usize = std::env::var("ZEDAZO_MAX_CONCURRENT_JOBS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4);
        let retention_hours: u64 = std::env::var("ZEDAZO_JOB_RETENTION_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);
        let auth = AuthConfig::from_env()?;
        Self::new_with_auth(
            data_dir,
            max_upload_bytes,
            max_concurrent_jobs,
            retention_hours,
            auth,
        )
    }

    /// Estado para tests in-process (TempDir como data root; auth disabled).
    pub fn new(
        data_dir: PathBuf,
        max_upload_bytes: usize,
        retention_hours: u64,
    ) -> anyhow::Result<Self> {
        Self::new_with_auth(
            data_dir,
            max_upload_bytes,
            32,
            retention_hours,
            AuthConfig::disabled(),
        )
    }

    pub fn new_with_auth(
        data_dir: PathBuf,
        max_upload_bytes: usize,
        max_concurrent_jobs: usize,
        retention_hours: u64,
        auth: AuthConfig,
    ) -> anyhow::Result<Self> {
        let storage = Storage::new(data_dir)?;
        let wipe_epoch = storage.read_wipe_epoch();
        Ok(Self {
            storage,
            max_upload_bytes,
            max_concurrent_jobs,
            retention_hours,
            jobs: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(HashMap::new())),
            core_version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: "v1".to_string(),
            auth,
            wipe_epoch: Arc::new(AtomicU64::new(wipe_epoch)),
            deleted_ids: Arc::new(RwLock::new(HashSet::new())),
        })
    }

    pub fn current_wipe_epoch(&self) -> u64 {
        self.wipe_epoch.load(std::sync::atomic::Ordering::SeqCst)
    }
}

pub fn router(state: AppState) -> Router {
    let limit = state.max_upload_bytes;
    let cors = auth::cors_layer_for(&state.auth);

    let public = Router::new()
        .route("/api/v1/health", get(api::health::health))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/logout", post(auth::logout));

    let protected = Router::new()
        .route("/api/v1/version", get(api::health::version))
        .route("/api/v1/uploads", post(api::uploads::create_upload))
        .route(
            "/api/v1/jobs",
            get(api::jobs::list_jobs).post(api::jobs::create_job),
        )
        .route(
            "/api/v1/jobs/{job_id}",
            get(api::jobs::get_job).delete(api::jobs::delete_job),
        )
        .route("/api/v1/jobs/{job_id}/events", get(api::jobs::job_events))
        .route("/api/v1/jobs/{job_id}/cancel", post(api::jobs::cancel_job))
        .route(
            "/api/v1/jobs/{job_id}/contacts",
            get(api::contacts::list_contacts),
        )
        .route(
            "/api/v1/jobs/{job_id}/contacts/{contact_id}",
            get(api::contacts::get_contact),
        )
        .route(
            "/api/v1/jobs/{job_id}/duplicates",
            get(api::contacts::list_duplicates),
        )
        .route("/api/v1/jobs/{job_id}/audit", get(api::audit::get_audit))
        .route("/api/v1/jobs/{job_id}/stats", get(api::stats::get_stats))
        .route(
            "/api/v1/jobs/{job_id}/artifacts/{kind}",
            get(api::artifacts::download),
        )
        .route("/api/v1/audits", post(api::audit::create_audit))
        .route("/api/v1/rules/validate", post(api::rules::validate))
        .route("/api/v1/rules/preview", post(api::rules::preview))
        .route("/api/v1/admin/wipe", post(api::admin::wipe_all))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    public
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(limit + 1024))
        .layer(cors)
        .with_state(state)
}
