//! Equivalencia semántica CLI (cribar) ↔ API HTTP sobre fixtures.
//!
//! Criterio: mismos conteos y artefactos (VCF/audit/stats/csv/json), no solo HTTP 200.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tempfile::tempdir;
use tower::ServiceExt;
use zedazo_api::app::{self, AppState};
use zedazo_core::application::cribar;
use zedazo_core::application::stats::Stats;
use zedazo_core::infrastructure::csv_writer::export_csv;
use zedazo_core::infrastructure::json_writer::export_json;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("zedazo-core")
        .join("tests")
        .join("fixtures")
}

fn fixture(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

const FIXTURES: &[&str] = &[
    "sample-contacts.vcf",
    "google_contactos.vcf",
    "google_otroscontactos.vcf",
    "proton_sample.vcf",
    "iso_sample.vcf",
    "duplicates.vcf",
    "edge_cases.vcf",
];

fn normalize_audit_tsv(raw: &str) -> String {
    raw.lines()
        .enumerate()
        .map(|(i, line)| {
            let mut cols: Vec<String> = line.split('\t').map(str::to_string).collect();
            if i > 0 && !cols.is_empty() {
                cols[0] = "<TS>".into();
            }
            // CATEGORIAS (col 7): conjunto sin orden estable entre corridas.
            if i > 0 && cols.len() > 7 {
                let mut cats: Vec<&str> = cols[7].split(',').filter(|s| !s.is_empty()).collect();
                cats.sort_unstable();
                cols[7] = cats.join(",");
            }
            cols.join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Diff semántico de VCF: ignora timestamps de job (`X-ZEDAZO-DATE`) y orden de CATEGORIES.
fn normalize_vcf(raw: &str) -> String {
    raw.lines()
        .map(|line| {
            let line = line.trim_end_matches('\r');
            if let Some(rest) = line.strip_prefix("X-ZEDAZO-DATE:") {
                let _ = rest;
                return "X-ZEDAZO-DATE:<TS>".to_string();
            }
            if let Some(cats) = line.strip_prefix("CATEGORIES:") {
                let mut parts: Vec<&str> = cats.split(',').filter(|s| !s.is_empty()).collect();
                parts.sort_unstable();
                return format!("CATEGORIES:{}", parts.join(","));
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_stats_json(raw: &str) -> Value {
    let mut v: Value = serde_json::from_str(raw).expect("stats.json válido");
    if let Some(obj) = v.as_object_mut() {
        if let Some(Value::Object(m)) = obj.get("por_categoria").cloned() {
            let ordered: BTreeMap<_, _> = m.into_iter().collect();
            obj.insert(
                "por_categoria".into(),
                Value::Object(ordered.into_iter().collect()),
            );
        }
        if let Some(Value::Object(m)) = obj.get("por_origen").cloned() {
            let ordered: BTreeMap<_, _> = m.into_iter().collect();
            obj.insert(
                "por_origen".into(),
                Value::Object(ordered.into_iter().collect()),
            );
        }
    }
    v
}

fn normalize_categories_field(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('"');
    let mut cats: Vec<&str> = trimmed
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    cats.sort_unstable();
    cats.join(", ")
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut cols = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                cols.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    cols.push(cur);
    cols
}

fn normalize_csv(raw: &str) -> String {
    let mut lines = raw.lines();
    let Some(header) = lines.next() else {
        return String::new();
    };
    let headers: Vec<&str> = header.split(',').collect();
    let cat_idx = headers.iter().position(|h| *h == "CATEGORIES");
    let mut out = vec![header.to_string()];
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let mut cols = parse_csv_line(line);
        if let Some(i) = cat_idx {
            if let Some(cell) = cols.get_mut(i) {
                *cell = normalize_categories_field(cell);
            }
        }
        out.push(cols.join(","));
    }
    out.join("\n")
}

fn normalize_contacts_json(raw: &str) -> Value {
    let mut v: Value = serde_json::from_str(raw).expect("contacts.json");
    fn sort_cats_in_obj(obj: &mut serde_json::Map<String, Value>) {
        if let Some(Value::Array(cats)) = obj.get_mut("categories") {
            let mut sorted: Vec<String> = cats
                .iter()
                .filter_map(|c| c.as_str().map(str::to_string))
                .collect();
            sorted.sort();
            *cats = sorted.into_iter().map(Value::String).collect();
        }
        if let Some(Value::String(s)) = obj.get_mut("categories") {
            *s = normalize_categories_field(s);
        }
    }
    match &mut v {
        Value::Array(arr) => {
            for item in arr {
                if let Some(obj) = item.as_object_mut() {
                    sort_cats_in_obj(obj);
                }
            }
        }
        Value::Object(root) => {
            if let Some(Value::Array(arr)) = root.get_mut("contacts") {
                for item in arr {
                    if let Some(obj) = item.as_object_mut() {
                        sort_cats_in_obj(obj);
                    }
                }
            }
        }
        _ => {}
    }
    v
}

async fn body_bytes(res: axum::response::Response) -> Vec<u8> {
    res.into_body()
        .collect()
        .await
        .expect("leer body")
        .to_bytes()
        .to_vec()
}

async fn body_json(res: axum::response::Response) -> Value {
    let bytes = body_bytes(res).await;
    serde_json::from_slice(&bytes).expect("JSON")
}

fn multipart_vcf(filename: &str, content: &[u8]) -> (String, Vec<u8>) {
    let boundary = "----zedazoParityBoundary7MA4YWxk";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: text/vcard\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

/// Ejecuta el flujo HTTP completo reutilizando el mismo `AppState`.
async fn api_artifacts(fixture_path: &Path) -> (Value, String, String, String, String, String) {
    let dir = tempdir().expect("tempdir");
    let state = AppState::new(dir.path().to_path_buf(), 52_428_800, 24).expect("state");

    let vcf = std::fs::read(fixture_path).expect("leer fixture");
    let name = fixture_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("input.vcf");
    let (ct, body) = multipart_vcf(name, &vcf);

    let upload_res = app::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/uploads")
                .header("content-type", ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_res.status(), StatusCode::CREATED);
    let upload: Value = body_json(upload_res).await;
    let upload_id = upload["upload_id"].as_str().unwrap().to_string();

    let create_body = serde_json::json!({
        "upload_id": upload_id,
        "artifacts": ["vcf", "audit_tsv", "stats_json", "csv", "json"]
    });
    let create_res = app::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/jobs")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);
    let job: Value = body_json(create_res).await;
    let job_id = job["job_id"].as_str().expect("job_id").to_string();

    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    let summary = loop {
        assert!(
            tokio::time::Instant::now() < deadline,
            "timeout esperando job {job_id}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
        let get_res = app::router(state.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/jobs/{job_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_res.status(), StatusCode::OK);
        let payload: Value = body_json(get_res).await;
        let status = payload["job"]["status"].as_str().unwrap_or("");
        match status {
            "completed" => break payload["job"]["summary"].clone(),
            "failed" => panic!("job falló: {:?}", payload["job"]["error"]),
            "cancelled" | "expired" => panic!("job en estado inesperado: {status}"),
            _ => continue,
        }
    };

    async fn artifact(state: &AppState, job_id: &str, kind: &str) -> String {
        let res = app::router(state.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/jobs/{job_id}/artifacts/{kind}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "artefacto {kind} debe descargarse"
        );
        String::from_utf8(body_bytes(res).await).expect("utf-8 artefacto")
    }

    let vcf_out = artifact(&state, &job_id, "vcf").await;
    let audit = artifact(&state, &job_id, "audit_tsv").await;
    let stats = artifact(&state, &job_id, "stats_json").await;
    let csv = artifact(&state, &job_id, "csv").await;
    let json = artifact(&state, &job_id, "json").await;

    (summary, vcf_out, audit, stats, csv, json)
}

fn cli_artifacts(fixture_path: &Path) -> (Stats, String, String, String, String, String) {
    let dir = tempdir().expect("cli temp");
    let out_vcf = dir.path().join("output.vcf");
    let audit = dir.path().join("audit.tsv");
    let (stats, contacts) = cribar::execute(
        fixture_path,
        Some(&out_vcf),
        Some(&audit),
        None,
        "auto",
        false,
        false,
        true,
    )
    .expect("cribar CLI");

    let stats_json = stats.to_json().expect("stats json");
    let csv_path = dir.path().join("contacts.csv");
    let json_path = dir.path().join("contacts.json");
    export_csv(
        &contacts,
        &csv_path,
        zedazo_core::infrastructure::secure_fs::WriteOpts::new(true),
    )
    .expect("csv");
    export_json(
        &contacts,
        &json_path,
        zedazo_core::infrastructure::secure_fs::WriteOpts::new(true),
    )
    .expect("json");

    (
        stats,
        std::fs::read_to_string(&out_vcf).expect("vcf"),
        std::fs::read_to_string(&audit).expect("audit"),
        stats_json,
        std::fs::read_to_string(&csv_path).expect("csv"),
        std::fs::read_to_string(&json_path).expect("json"),
    )
}

fn assert_semantic_parity(name: &str, summary: &Value, cli: &Stats) {
    assert_eq!(
        summary["input_contacts"].as_u64().unwrap() as usize,
        cli.total_entrada,
        "{name}: input_contacts"
    );
    assert_eq!(
        summary["retained"].as_u64().unwrap() as usize,
        cli.conservados,
        "{name}: retained"
    );
    assert_eq!(
        summary["needs_review"].as_u64().unwrap() as usize,
        cli.needs_review,
        "{name}: needs_review"
    );
    assert_eq!(
        summary["eliminated"].as_u64().unwrap() as usize,
        cli.eliminados,
        "{name}: eliminated"
    );
    assert_eq!(
        summary["quarantine"].as_u64().unwrap() as usize,
        cli.cuarentena,
        "{name}: quarantine"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn equivalence_cli_http_all_fixtures() {
    for name in FIXTURES {
        let path = fixture(name);
        assert!(path.exists(), "falta fixture {name} en {}", path.display());

        let (cli_stats, cli_vcf, cli_audit, cli_stats_json, cli_csv, cli_json) =
            cli_artifacts(&path);
        let (summary, api_vcf, api_audit, api_stats, api_csv, api_json) =
            api_artifacts(&path).await;

        assert!(
            !summary.is_null(),
            "{name}: summary no debe ser null (paridad semántica, no solo completed)"
        );
        assert_semantic_parity(name, &summary, &cli_stats);

        assert_eq!(
            normalize_vcf(&cli_vcf),
            normalize_vcf(&api_vcf),
            "{name}: VCF semántico distinto"
        );
        assert_eq!(
            normalize_audit_tsv(&cli_audit),
            normalize_audit_tsv(&api_audit),
            "{name}: audit TSV distinto"
        );
        assert_eq!(
            normalize_stats_json(&cli_stats_json),
            normalize_stats_json(&api_stats),
            "{name}: stats.json distinto"
        );
        assert_eq!(
            normalize_csv(&cli_csv),
            normalize_csv(&api_csv),
            "{name}: CSV distinto"
        );
        assert_eq!(
            normalize_contacts_json(&cli_json),
            normalize_contacts_json(&api_json),
            "{name}: JSON distinto"
        );
    }
}

/// Guardrail: un job "completed" sin summary/artefactos no cuenta como paridad.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn equivalence_rejects_empty_completed_semantics() {
    let path = fixture("sample-contacts.vcf");
    let (summary, vcf, audit, stats, csv, json) = api_artifacts(&path).await;
    assert!(summary.get("input_contacts").is_some());
    assert!(summary["input_contacts"].as_u64().unwrap() > 0);
    assert!(!vcf.is_empty());
    assert!(!audit.is_empty());
    assert!(!stats.is_empty());
    assert!(!csv.is_empty());
    assert!(!json.is_empty());
}

/// Entrada vacía: el upload se rechaza (400); no hay job ni artefacto.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn adverse_empty_vcf_upload_rejected() {
    let dir = tempdir().expect("tempdir");
    let state = AppState::new(dir.path().to_path_buf(), 52_428_800, 24).expect("state");
    let (ct, body) = multipart_vcf("empty.vcf", b"");
    let upload_res = app::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/uploads")
                .header("content-type", ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_res.status(), StatusCode::BAD_REQUEST);
}

/// TOML de reglas inválido: validate responde ok=false.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn adverse_rules_validate_rejects_invalid_toml() {
    let dir = tempdir().expect("tempdir");
    let state = AppState::new(dir.path().to_path_buf(), 52_428_800, 24).expect("state");
    let res = app::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/rules/validate")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "toml": "[zedazo\nesto = no es toml"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = body_json(res).await;
    assert_eq!(body["ok"], false);
    assert!(body["diagnostics"]
        .as_array()
        .map(|a| !a.is_empty())
        .unwrap_or(false));
}

/// Cancelar → borrar → segundo DELETE idempotente (204).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_then_delete_is_idempotent() {
    let dir = tempdir().expect("tempdir");
    let state = AppState::new(dir.path().to_path_buf(), 52_428_800, 24).expect("state");
    let path = fixture("sample-contacts.vcf");
    let vcf = std::fs::read(&path).expect("fixture");
    let (ct, body) = multipart_vcf("sample.vcf", &vcf);

    let upload_res = app::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/uploads")
                .header("content-type", ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let upload: Value = body_json(upload_res).await;
    let upload_id = upload["upload_id"].as_str().unwrap();

    let create_res = app::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/jobs")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "upload_id": upload_id,
                        "artifacts": ["vcf", "audit_tsv"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let job: Value = body_json(create_res).await;
    let job_id = job["job_id"].as_str().unwrap().to_string();

    let cancel = app::router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/jobs/{job_id}/cancel"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel.status(), StatusCode::ACCEPTED);

    // Esperar estado terminal (cancelled o completed si ganó la carrera).
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        assert!(tokio::time::Instant::now() < deadline, "timeout cancel");
        tokio::time::sleep(Duration::from_millis(40)).await;
        let get_res = app::router(state.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/jobs/{job_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let payload: Value = body_json(get_res).await;
        let status = payload["job"]["status"].as_str().unwrap_or("");
        if matches!(status, "cancelled" | "completed" | "failed" | "expired") {
            break;
        }
    }

    let del1 = app::router(state.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del1.status(), StatusCode::NO_CONTENT);

    let del2 = app::router(state.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del2.status(), StatusCode::NO_CONTENT);
}

/// Descargar artefactos tras job completed (ruta GUI → API).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn download_artifacts_after_completed() {
    let path = fixture("duplicates.vcf");
    let (_summary, vcf, audit, _stats, csv, json) = api_artifacts(&path).await;
    assert!(vcf.contains("BEGIN:VCARD"));
    assert!(audit.contains('\t'));
    assert!(csv.contains(',') || csv.lines().count() > 0);
    assert!(json.trim_start().starts_with('[') || json.trim_start().starts_with('{'));
}
