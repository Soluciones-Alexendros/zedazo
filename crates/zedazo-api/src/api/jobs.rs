use std::collections::BTreeSet;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::stream::Stream;
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use uuid::Uuid;
use zedazo_core::application::process::{ArtifactKind, CancellationToken};

use crate::app::AppState;
use crate::dto::{
    parse_artifact_kind, CreateJobRequest, InputMeta, JobManifest, JobRecord, JobStatus, RulesMeta,
};
use crate::jobs::{now_rfc3339, save_manifest, spawn_job};

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

pub async fn list_jobs(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Json<serde_json::Value> {
    let jobs = state.jobs.read().await;
    let mut items: Vec<_> = jobs
        .values()
        .filter(|j| {
            q.status.as_ref().map_or(true, |s| {
                let st = serde_json::to_value(&j.manifest.status)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string));
                st.as_deref() == Some(s.as_str())
            })
        })
        .map(|j| &j.manifest)
        .collect();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Json(serde_json::json!({ "items": items, "next_cursor": null }))
}

pub async fn create_job(
    State(state): State<AppState>,
    Json(body): Json<CreateJobRequest>,
) -> Result<(StatusCode, Json<JobManifest>), (StatusCode, String)> {
    crate::storage::Storage::validate_id(&body.upload_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "upload_id inválido".into()))?;
    if state.max_concurrent_jobs > 0 {
        let jobs = state.jobs.read().await;
        let active = jobs
            .values()
            .filter(|j| {
                !matches!(
                    j.manifest.status,
                    JobStatus::Completed
                        | JobStatus::Failed
                        | JobStatus::Cancelled
                        | JobStatus::Expired
                        | JobStatus::Deleted
                )
            })
            .count();
        if active >= state.max_concurrent_jobs {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                "límite de trabajos concurrentes".into(),
            ));
        }
    }
    let upload_path = state.storage.upload_path(&body.upload_id);
    if !upload_path.exists() {
        return Err((StatusCode::NOT_FOUND, "upload_id desconocido".into()));
    }
    let meta = std::fs::metadata(&upload_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let sha = {
        use sha2::{Digest, Sha256};
        let data = std::fs::read(&upload_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        hex::encode(Sha256::digest(&data))
    };

    let job_id = Uuid::now_v7().to_string();
    let retention = body.retention_hours.unwrap_or(state.retention_hours);
    if !(1..=state.retention_max_hours).contains(&retention) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "retention_hours fuera de rango 1..={}",
                state.retention_max_hours
            ),
        ));
    }
    let rules_mode = body
        .rules
        .as_ref()
        .and_then(|r| r.mode.clone())
        .unwrap_or_else(|| "builtin".into());
    let config_toml = body.rules.as_ref().and_then(|r| r.toml.clone());

    let arts: Vec<ArtifactKind> = body
        .artifacts
        .unwrap_or_default()
        .iter()
        .filter_map(|s| parse_artifact_kind(s))
        .collect();
    let arts = if arts.is_empty() {
        vec![
            ArtifactKind::Vcf,
            ArtifactKind::AuditTsv,
            ArtifactKind::StatsJson,
            ArtifactKind::Csv,
            ArtifactKind::Json,
        ]
    } else {
        arts
    };

    let _ = BTreeSet::<ArtifactKind>::from_iter(arts.iter().copied());

    let manifest = JobManifest {
        job_id: job_id.clone(),
        status: JobStatus::Queued,
        display_name: body.display_name,
        created_at: now_rfc3339(),
        started_at: None,
        completed_at: None,
        core_version: state.core_version.clone(),
        input: InputMeta {
            original_name: body.upload_id.clone(),
            sha256: sha,
            bytes: meta.len(),
            source_detected: None,
            vcard_version: None,
            upload_id: body.upload_id.clone(),
        },
        rules: RulesMeta {
            mode: rules_mode,
            sha256: config_toml.as_ref().map(|t| {
                use sha2::{Digest, Sha256};
                hex::encode(Sha256::digest(t.as_bytes()))
            }),
        },
        summary: None,
        artifacts: vec![],
        error: None,
        retention_hours: retention,
    };

    state
        .storage
        .ensure_job_dir(&job_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    save_manifest(&state, &manifest);

    let (tx, _rx) = tokio::sync::broadcast::channel(256);
    {
        let mut events = state.events.write().await;
        events.insert(job_id.clone(), tx);
    }

    let record = JobRecord {
        manifest: manifest.clone(),
        contact_views: vec![],
        duplicate_groups: vec![],
        warnings: vec![],
        cancel: CancellationToken::new(),
        requested_artifacts: arts,
        config_toml,
    };
    {
        let mut jobs = state.jobs.write().await;
        jobs.insert(job_id.clone(), record);
    }

    spawn_job(state.clone(), job_id);
    Ok((StatusCode::CREATED, Json(manifest)))
}

pub async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let jobs = state.jobs.read().await;
    let rec = jobs.get(&job_id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::json!({
        "job": rec.manifest,
        "warnings": rec.warnings,
    })))
}

pub async fn delete_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    use crate::retention::DeleteOutcome;
    match crate::retention::delete_job_tracked(&state, &job_id).await {
        DeleteOutcome::Deleted | DeleteOutcome::Idempotent | DeleteOutcome::PendingRetry => {
            Ok(StatusCode::NO_CONTENT)
        }
        DeleteOutcome::NotFound => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn cancel_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut jobs = state.jobs.write().await;
    let rec = jobs.get_mut(&job_id).ok_or(StatusCode::NOT_FOUND)?;
    rec.cancel.cancel();
    rec.manifest.status = JobStatus::CancelRequested;
    save_manifest(&state, &rec.manifest);
    Ok(StatusCode::ACCEPTED)
}

pub async fn job_events(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    headers: axum::http::HeaderMap,
    Query(q): Query<EventsQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, StatusCode> {
    {
        let jobs = state.jobs.read().await;
        if !jobs.contains_key(&job_id) {
            return Err(StatusCode::NOT_FOUND);
        }
    }

    let after_id = q
        .last_event_id
        .as_deref()
        .or_else(|| headers.get("last-event-id").and_then(|v| v.to_str().ok()))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let events_path = state.storage.job_dir(&job_id).join("events.ndjson");
    let replay = crate::jobs::read_events_after(&events_path, after_id);

    let rx = {
        let events = state.events.read().await;
        let tx = events.get(&job_id).ok_or(StatusCode::NOT_FOUND)?;
        tx.subscribe()
    };

    let replay_stream = futures_util::stream::iter(replay.into_iter().map(|ev| {
        Ok(Event::default()
            .id(ev.id.to_string())
            .event(ev.event)
            .data(ev.data.to_string()))
    }));

    let live = BroadcastStream::new(rx).filter_map(move |msg| match msg {
        Ok(ev) => {
            if ev.id <= after_id {
                None
            } else {
                Some(Ok(Event::default()
                    .id(ev.id.to_string())
                    .event(ev.event)
                    .data(ev.data.to_string())))
            }
        }
        Err(_) => None,
    });

    let stream = replay_stream.chain(live);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[derive(Deserialize)]
pub struct EventsQuery {
    pub last_event_id: Option<String>,
}
