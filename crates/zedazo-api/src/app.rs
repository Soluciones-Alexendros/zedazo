use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tokio::sync::{broadcast, RwLock, Semaphore};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::api;
use crate::auth::{self, AuthConfig};
use crate::host_guard::{self, HostPolicy};
use crate::jobs::{JobEvent, JobRecord};
use crate::storage::Storage;

/// Errores de configuración con mensajes accionables.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("retention_max_hours debe ser >= 1 y <= 8760 (1 año); valor: {0}")]
    RetentionMaxHours(u64),
    #[error("retention_hours ({0}) debe ser >= 1 y <= retention_max_hours ({1})")]
    RetentionHours(u64, u64),
    #[error("max_upload_bytes debe ser > 0; valor: {0}")]
    MaxUploadBytes(usize),
    #[error("max_concurrent_jobs debe ser > 0; valor: {0}")]
    MaxConcurrentJobs(usize),
    #[error("data_dir no escribible: {0}")]
    DataDirNotWritable(PathBuf),
}

/// Tope de retención de job (horas); defecto del plan S-13.
pub const DEFAULT_RETENTION_MAX_HOURS: u64 = 168;

/// Límite superior absoluto para retention_max_hours (1 año).
pub const MAX_RETENTION_MAX_HOURS: u64 = 8760;

/// Cupo interno de uploads concurrentes (sin env nueva).
const UPLOAD_SEM_PERMITS: usize = 2;

#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
    pub max_upload_bytes: usize,
    /// Máximo de jobs no terminales concurrentes (0 = sin límite explícito).
    pub max_concurrent_jobs: usize,
    pub retention_hours: u64,
    pub retention_max_hours: u64,
    pub jobs: Arc<RwLock<HashMap<String, JobRecord>>>,
    pub events: Arc<RwLock<HashMap<String, broadcast::Sender<JobEvent>>>>,
    pub core_version: String,
    pub api_version: String,
    pub auth: AuthConfig,
    /// Época de wipe: el worker abandona escrituras si no coincide.
    pub wipe_epoch: Arc<AtomicU64>,
    /// Tombstones en memoria para DELETE idempotente en la misma vida del proceso.
    pub deleted_ids: Arc<RwLock<HashSet<String>>>,
    pub host_policy: HostPolicy,
    pub upload_sem: Arc<Semaphore>,
}

/// Valida la configuración antes de construir el estado.
/// Devuelve error con mensaje accionable si hay valores inválidos.
pub fn validate_config(
    data_dir: &PathBuf,
    max_upload_bytes: usize,
    max_concurrent_jobs: usize,
    retention_hours: u64,
    retention_max_hours: u64,
) -> Result<(), ConfigError> {
    if retention_max_hours == 0 || retention_max_hours > MAX_RETENTION_MAX_HOURS {
        return Err(ConfigError::RetentionMaxHours(retention_max_hours));
    }
    if retention_hours == 0 || retention_hours > retention_max_hours {
        return Err(ConfigError::RetentionHours(
            retention_hours,
            retention_max_hours,
        ));
    }
    if max_upload_bytes == 0 {
        return Err(ConfigError::MaxUploadBytes(max_upload_bytes));
    }
    if max_concurrent_jobs == 0 {
        return Err(ConfigError::MaxConcurrentJobs(max_concurrent_jobs));
    }
    // Verificar que data_dir es escribible (crear si no existe)
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir)
            .map_err(|_| ConfigError::DataDirNotWritable(data_dir.clone()))?;
    } else {
        // Test de escritura
        let test_file = data_dir.join(".zedazo_write_test");
        std::fs::write(&test_file, b"test")
            .map_err(|_| ConfigError::DataDirNotWritable(data_dir.clone()))?;
        let _ = std::fs::remove_file(&test_file);
    }
    Ok(())
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
        let retention_max_hours: u64 = std::env::var("ZEDAZO_JOB_RETENTION_MAX_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_RETENTION_MAX_HOURS);
        let retention_hours: u64 = std::env::var("ZEDAZO_JOB_RETENTION_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        // Validar antes de clamp/build
        validate_config(
            &data_dir,
            max_upload_bytes,
            max_concurrent_jobs,
            retention_hours,
            retention_max_hours,
        )?;

        let retention_hours = retention_hours.clamp(1, retention_max_hours);
        let auth = AuthConfig::from_env()?;
        let bind = std::env::var("ZEDAZO_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
        let host_policy = HostPolicy::from_bind(&bind);
        Self::build(
            data_dir,
            max_upload_bytes,
            max_concurrent_jobs,
            retention_hours,
            retention_max_hours,
            auth,
            host_policy,
        )
    }

    /// Estado para tests in-process (TempDir como data root; auth disabled).
    pub fn new(
        data_dir: PathBuf,
        max_upload_bytes: usize,
        retention_hours: u64,
    ) -> anyhow::Result<Self> {
        Self::build(
            data_dir,
            max_upload_bytes,
            32,
            retention_hours,
            DEFAULT_RETENTION_MAX_HOURS,
            AuthConfig::disabled(),
            HostPolicy::for_port(8080),
        )
    }

    pub fn new_with_auth(
        data_dir: PathBuf,
        max_upload_bytes: usize,
        max_concurrent_jobs: usize,
        retention_hours: u64,
        auth: AuthConfig,
    ) -> anyhow::Result<Self> {
        Self::build(
            data_dir,
            max_upload_bytes,
            max_concurrent_jobs,
            retention_hours,
            DEFAULT_RETENTION_MAX_HOURS,
            auth,
            HostPolicy::for_port(8080),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        data_dir: PathBuf,
        max_upload_bytes: usize,
        max_concurrent_jobs: usize,
        retention_hours: u64,
        retention_max_hours: u64,
        auth: AuthConfig,
        host_policy: HostPolicy,
    ) -> anyhow::Result<Self> {
        let storage = Storage::new(data_dir)?;
        let wipe_epoch = storage.read_wipe_epoch();
        Ok(Self {
            storage,
            max_upload_bytes,
            max_concurrent_jobs,
            retention_hours,
            retention_max_hours,
            jobs: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(HashMap::new())),
            core_version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: "v1".to_string(),
            auth,
            wipe_epoch: Arc::new(AtomicU64::new(wipe_epoch)),
            deleted_ids: Arc::new(RwLock::new(HashSet::new())),
            host_policy,
            upload_sem: Arc::new(Semaphore::new(UPLOAD_SEM_PERMITS)),
        })
    }

    pub fn with_host_policy(mut self, policy: HostPolicy) -> Self {
        self.host_policy = policy;
        self
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

    let uploads = api::uploads::uploads_router(limit).route_layer(middleware::from_fn_with_state(
        state.clone(),
        auth::require_auth,
    ));

    let protected = Router::new()
        .route("/api/v1/version", get(api::health::version))
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
        .layer(RequestBodyLimitLayer::new(limit.saturating_add(1024)))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    // host_guard queda como capa exterior (tras cors) para que OPTIONS no
    // se ataje con 200 antes de validar Origin (S-01).
    public
        .merge(uploads)
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            host_guard::host_origin_middleware,
        ))
        .with_state(state)
}
