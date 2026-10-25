//! Retención, reconciliación tras reinicio y borrado reintentable.

use std::fs;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tempfile::TempDir;
use tower::ServiceExt;
use zedazo_api::app::{self, AppState};
use zedazo_api::dto::{InputMeta, JobManifest, JobStatus, RulesMeta};
use zedazo_api::jobs::{now_rfc3339, save_manifest_result};
use zedazo_api::retention;
use zedazo_api::storage::{PendingKind, Storage};

fn write_interrupted_job(storage: &Storage, job_id: &str) {
    storage.ensure_job_dir(job_id).unwrap();
    let manifest = JobManifest {
        job_id: job_id.to_string(),
        status: JobStatus::Screening,
        display_name: None,
        created_at: now_rfc3339(),
        started_at: Some(now_rfc3339()),
        completed_at: None,
        core_version: "0.5.1".into(),
        input: InputMeta {
            original_name: "x.vcf".into(),
            sha256: "abc".into(),
            bytes: 1,
            source_detected: None,
            vcard_version: None,
            upload_id: "00000000-0000-7000-8000-000000000001".into(),
        },
        rules: RulesMeta {
            mode: "builtin".into(),
            sha256: None,
        },
        summary: None,
        artifacts: vec![],
        error: None,
        retention_hours: 24,
    };
    let path = storage.job_dir(job_id).join("manifest.json");
    Storage::atomic_write(
        &path,
        serde_json::to_string_pretty(&manifest).unwrap().as_bytes(),
    )
    .unwrap();
    fs::write(
        storage.job_dir(job_id).join("input.vcf"),
        b"BEGIN:VCARD\nEND:VCARD\n",
    )
    .unwrap();
}

#[tokio::test]
async fn restart_marks_in_flight_as_proceso_interrumpido() {
    let dir = TempDir::new().unwrap();
    let job_id = "01900000-0000-7000-8000-0000000000aa";
    {
        let storage = Storage::new(dir.path().to_path_buf()).unwrap();
        write_interrupted_job(&storage, job_id);
    }

    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    retention::reconcile_on_startup(&state).await.unwrap();

    let jobs = state.jobs.read().await;
    let rec = jobs.get(job_id).expect("job rehidratado");
    assert_eq!(rec.manifest.status, JobStatus::Failed);
    assert_eq!(rec.manifest.error.as_deref(), Some("proceso_interrumpido"));
    assert!(rec.manifest.completed_at.is_some());
}

#[tokio::test]
async fn orphan_upload_swept_by_ttl() {
    let dir = TempDir::new().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    let upload_id = "01900000-0000-7000-8000-0000000000bb";
    let path = state.storage.upload_path(upload_id);
    fs::write(&path, b"orphan").unwrap();
    // Meta antigua (48 h).
    let old = (chrono::Utc::now() - chrono::Duration::hours(48)).to_rfc3339();
    Storage::atomic_write(
        &state.storage.upload_meta_path(upload_id),
        serde_json::json!({ "created_at": old, "bytes": 6, "sha256": "x" })
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    std::env::set_var("ZEDAZO_UPLOAD_TTL_HOURS", "1");
    retention::reconcile_on_startup(&state).await.unwrap();
    std::env::remove_var("ZEDAZO_UPLOAD_TTL_HOURS");

    assert!(!path.exists(), "upload huérfano debe borrarse");
}

#[tokio::test]
async fn failed_delete_stays_in_pending() {
    let dir = TempDir::new().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    let job_id = "01900000-0000-7000-8000-0000000000cc";
    write_interrupted_job(&state.storage, job_id);
    retention::reconcile_on_startup(&state).await.unwrap();

    // Simular fallo encolando a mano (misma vía que delete cuando FS falla).
    state
        .storage
        .enqueue_pending_delete(PendingKind::Job, job_id)
        .unwrap();
    let pending = state.storage.load_pending_deletes();
    assert!(pending.iter().any(|p| p.id == job_id));

    // Reintento exitoso tras borrado real.
    state.storage.delete_job_dir(job_id).unwrap();
    retention::retention_sweep(&state).await;
    let pending = state.storage.load_pending_deletes();
    assert!(!pending.iter().any(|p| p.id == job_id));
}

#[tokio::test]
async fn second_delete_is_idempotent() {
    let dir = TempDir::new().unwrap();
    let state = Arc::new(AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap());
    let job_id = "01900000-0000-7000-8000-0000000000dd";
    write_interrupted_job(&state.storage, job_id);
    retention::reconcile_on_startup(&state).await.unwrap();

    let app = app::router((*state).clone());
    let del1 = app
        .clone()
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .method("DELETE")
                .uri(format!("/api/v1/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del1.status(), StatusCode::NO_CONTENT);

    let del2 = app
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .method("DELETE")
                .uri(format!("/api/v1/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del2.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn wipe_bumps_epoch() {
    let dir = TempDir::new().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    let before = state.current_wipe_epoch();
    let app = app::router(state.clone());
    let res = app
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .method("POST")
                .uri("/api/v1/admin/wipe")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["wiped"], true);
    assert!(state.current_wipe_epoch() > before);
}

#[tokio::test]
async fn completed_job_survives_restart() {
    let dir = TempDir::new().unwrap();
    let job_id = "01900000-0000-7000-8000-0000000000ee";
    {
        let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
        state.storage.ensure_job_dir(job_id).unwrap();
        let mut manifest = JobManifest {
            job_id: job_id.to_string(),
            status: JobStatus::Completed,
            display_name: None,
            created_at: now_rfc3339(),
            started_at: Some(now_rfc3339()),
            completed_at: Some(now_rfc3339()),
            core_version: "0.5.1".into(),
            input: InputMeta {
                original_name: "x.vcf".into(),
                sha256: "abc".into(),
                bytes: 1,
                source_detected: None,
                vcard_version: None,
                upload_id: "00000000-0000-7000-8000-000000000002".into(),
            },
            rules: RulesMeta {
                mode: "builtin".into(),
                sha256: None,
            },
            summary: None,
            artifacts: vec!["vcf".into()],
            error: None,
            retention_hours: 24,
        };
        // Asegurar no vencido.
        manifest.completed_at = Some(now_rfc3339());
        save_manifest_result(&state, &manifest).unwrap();
    }

    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    retention::reconcile_on_startup(&state).await.unwrap();
    let jobs = state.jobs.read().await;
    let rec = jobs.get(job_id).expect("completed rehidratado");
    assert_eq!(rec.manifest.status, JobStatus::Completed);
}
