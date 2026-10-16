//! Reconciliación al arrancar y barrido periódico de retención.
//!
//! El TTL no es exacto al segundo: el barrido corre cada hora (y una vez al arrancar).

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};
use zedazo_core::application::process::CancellationToken;

use crate::app::AppState;
use crate::dto::{JobManifest, JobRecord, JobStatus};
use crate::jobs::{now_rfc3339, JobEvent};
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
        if should_expire(&manifest, retention_hours) {
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

        recovered.insert(
            id.clone(),
            JobRecord {
                manifest,
                contact_views: vec![],
                duplicate_groups: vec![],
                warnings: vec![],
                cancel: CancellationToken::new(),
                requested_artifacts: vec![],
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

fn should_expire(manifest: &JobManifest, default_retention: u64) -> bool {
    let Some(completed) = manifest.completed_at.as_ref() else {
        return false;
    };
    let Ok(ts) = chrono::DateTime::parse_from_rfc3339(completed) else {
        return false;
    };
    let age = chrono::Utc::now().signed_duration_since(ts.with_timezone(&chrono::Utc));
    let hours = if manifest.retention_hours > 0 {
        manifest.retention_hours
    } else {
        default_retention
    };
    age.num_hours() >= hours as i64
}

fn sweep_uploads(
    state: &AppState,
    upload_ttl_hours: u64,
    referenced: &std::collections::HashSet<String>,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now();
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
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                })
                .map(|dt| dt.with_timezone(&chrono::Utc))
        } else {
            // Sin meta: usar mtime del fichero.
            state
                .storage
                .upload_path(&upload_id)
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(chrono::DateTime::<chrono::Utc>::from)
        };
        let Some(created) = created else {
            continue;
        };
        let age = now.signed_duration_since(created);
        if age.num_hours() >= upload_ttl_hours as i64 {
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
            .map(|t| {
                let dt = chrono::DateTime::<chrono::Utc>::from(t);
                chrono::Utc::now().signed_duration_since(dt).num_hours() >= 1
            })
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
            if should_expire(&rec.manifest, retention_hours) {
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

/// Garantiza un canal de eventos para jobs recuperados (si se pide SSE).
#[allow(dead_code)]
pub async fn ensure_event_channel(
    events: &Arc<RwLock<HashMap<String, broadcast::Sender<JobEvent>>>>,
    job_id: &str,
) -> broadcast::Sender<JobEvent> {
    let mut ev = events.write().await;
    if let Some(tx) = ev.get(job_id) {
        return tx.clone();
    }
    let (tx, _) = broadcast::channel(256);
    ev.insert(job_id.to_string(), tx.clone());
    tx
}
