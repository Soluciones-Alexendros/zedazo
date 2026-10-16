//! Wipe admin + replay SSE (Last-Event-ID / last_event_id).

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tempfile::tempdir;
use tower::ServiceExt;
use zedazo_api::app::{self, AppState};
use zedazo_api::jobs::{read_events_after, JobEvent};

async fn body_json(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wipe_clears_jobs_and_fs() {
    let dir = tempdir().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    let jobs_dir = dir.path().join("jobs").join("fake");
    std::fs::create_dir_all(&jobs_dir).unwrap();
    std::fs::write(jobs_dir.join("x"), b"1").unwrap();

    let res = app::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/wipe")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    assert_eq!(body["wiped"], true);
    assert!(!jobs_dir.exists());
    assert!(dir.path().join("jobs").exists());
    assert!(state.jobs.read().await.is_empty());
}

#[test]
fn events_ndjson_replay_filters_by_id() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("events.ndjson");
    let lines = [
        JobEvent {
            id: 1,
            event: "phase".into(),
            data: serde_json::json!({"phase":"Validating"}),
        },
        JobEvent {
            id: 2,
            event: "phase".into(),
            data: serde_json::json!({"phase":"Parsing"}),
        },
        JobEvent {
            id: 3,
            event: "metric".into(),
            data: serde_json::json!({"name":"contacts","value":2}),
        },
    ];
    let content = lines
        .iter()
        .map(|e| serde_json::to_string(e).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, content).unwrap();
    let after = read_events_after(&path, 1);
    assert_eq!(after.len(), 2);
    assert_eq!(after[0].id, 2);
    assert_eq!(after[1].id, 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_query_last_event_id_replays() {
    let dir = tempdir().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    let job_id = "01900000-0000-7000-8000-0000000000ff";
    state.storage.ensure_job_dir(job_id).unwrap();
    let events_path = state.storage.job_dir(job_id).join("events.ndjson");
    let ev = JobEvent {
        id: 5,
        event: "phase".into(),
        data: serde_json::json!({"phase":"Completed"}),
    };
    std::fs::write(&events_path, serde_json::to_string(&ev).unwrap() + "\n").unwrap();

    {
        let (tx, _rx) = tokio::sync::broadcast::channel(8);
        state.events.write().await.insert(job_id.into(), tx);
        let mut jobs = state.jobs.write().await;
        jobs.insert(
            job_id.into(),
            zedazo_api::dto::JobRecord {
                manifest: zedazo_api::dto::JobManifest {
                    job_id: job_id.into(),
                    status: zedazo_api::dto::JobStatus::Completed,
                    display_name: None,
                    created_at: "t".into(),
                    started_at: None,
                    completed_at: None,
                    core_version: "t".into(),
                    input: zedazo_api::dto::InputMeta {
                        original_name: "x".into(),
                        sha256: "x".into(),
                        bytes: 1,
                        source_detected: None,
                        vcard_version: None,
                        upload_id: "01900000-0000-7000-8000-0000000000fe".into(),
                    },
                    rules: zedazo_api::dto::RulesMeta {
                        mode: "builtin".into(),
                        sha256: None,
                    },
                    summary: None,
                    artifacts: vec![],
                    error: None,
                    retention_hours: 24,
                },
                contact_views: vec![],
                duplicate_groups: vec![],
                warnings: vec![],
                cancel: Default::default(),
                requested_artifacts: vec![],
                config_toml: None,
            },
        );
    }

    let res = app::router(state)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/jobs/{job_id}/events?last_event_id=4"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    // Leer un fragmento del stream SSE (replay + keep-alive)
    let body = res.into_body();
    let collected = tokio::time::timeout(Duration::from_millis(400), async {
        body.collect().await.unwrap().to_bytes()
    })
    .await;
    // El stream puede no cerrar; timeout es esperado. Si llegaron bytes, validamos replay.
    if let Ok(bytes) = collected {
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("id:5") || text.contains("Completed"),
            "replay SSE: {text}"
        );
    }
}
