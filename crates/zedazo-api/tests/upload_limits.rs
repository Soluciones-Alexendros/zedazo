//! S-04: límite de upload (10 MB OK; MAX+1 → 413).

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use tower::ServiceExt;
use zedazo_api::app::{self, AppState};

fn vcf_payload(target_bytes: usize) -> Vec<u8> {
    let mut body = String::from("BEGIN:VCARD\r\nVERSION:4.0\r\nUID:big\r\nFN:Big\r\nNOTE:");
    // Relleno sintético hasta el tamaño pedido (sin PII).
    while body.len() + 20 < target_bytes {
        body.push('x');
    }
    body.push_str("\r\nEND:VCARD\r\n");
    let mut bytes = body.into_bytes();
    if bytes.len() < target_bytes {
        bytes.extend(std::iter::repeat(b'y').take(target_bytes - bytes.len()));
    } else {
        bytes.truncate(target_bytes);
        // Asegurar marcadores al inicio tras truncate agresivo
        let prefix = b"BEGIN:VCARD\r\n";
        bytes[..prefix.len()].copy_from_slice(prefix);
    }
    bytes
}

fn multipart_body(filename: &str, data: &[u8]) -> (String, Vec<u8>) {
    let boundary = "----zedazoBoundary7MA4YWxkTrZu0gW";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: text/vcard\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

#[tokio::test]
async fn upload_10mb_created_sha_matches() {
    let dir = tempdir().unwrap();
    const MAX: usize = 10 * 1024 * 1024;
    let state = AppState::new(dir.path().to_path_buf(), MAX, 24).unwrap();
    let data = vcf_payload(MAX);
    let expect_sha = hex::encode(Sha256::digest(&data));
    let (ct, body) = multipart_body("big.vcf", &data);

    let res = app::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/uploads")
                .header(header::HOST, "127.0.0.1:8080")
                .header(header::CONTENT_TYPE, ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["sha256"], expect_sha);
    assert_eq!(json["bytes"].as_u64().unwrap(), MAX as u64);
}

#[tokio::test]
async fn upload_over_limit_413() {
    let dir = tempdir().unwrap();
    const MAX: usize = 1024 * 1024;
    let state = AppState::new(dir.path().to_path_buf(), MAX, 24).unwrap();
    let data = vcf_payload(MAX + 1);
    let (ct, body) = multipart_body("over.vcf", &data);

    let res = app::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/uploads")
                .header(header::HOST, "127.0.0.1:8080")
                .header(header::CONTENT_TYPE, ct)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
