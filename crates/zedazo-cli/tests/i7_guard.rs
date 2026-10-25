//! Guardas I7 / S-02 / S-05 / S-06: same-file, symlink, modos 0600, Parse sin valor.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_zedazo"))
}

fn sha256_hex(path: &Path) -> String {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .expect("sha256sum");
    assert!(out.status.success(), "sha256sum falló");
    let s = String::from_utf8_lossy(&out.stdout);
    s.split_whitespace().next().unwrap().to_string()
}

fn minimal_vcf() -> &'static str {
    "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:test-i7\r\nFN:Example Org\r\nN:Org;Example;;;\r\nEMAIL:contact@example.invalid\r\nTEL;TYPE=cell:+34600000000\r\nEND:VCARD\r\n"
}

fn write_input(dir: &Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, minimal_vcf()).unwrap();
    p
}

fn run_cribar(args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("ejecutar zedazo")
}

#[test]
fn o_igual_a_entrada_no_destruye() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path(), "x.vcf");
    let before = sha256_hex(&input);
    let out = run_cribar(&[
        "cribar",
        input.to_str().unwrap(),
        "-o",
        input.to_str().unwrap(),
    ]);
    assert_ne!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(sha256_hex(&input), before);
}

#[test]
fn a_igual_a_entrada_no_destruye() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path(), "in.vcf");
    let before = sha256_hex(&input);
    let out_vcf = dir.path().join("out.vcf");
    let out = run_cribar(&[
        "cribar",
        input.to_str().unwrap(),
        "-o",
        out_vcf.to_str().unwrap(),
        "-a",
        input.to_str().unwrap(),
    ]);
    assert_ne!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(sha256_hex(&input), before);
}

#[test]
fn symlink_salida_rechazada() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path(), "in.vcf");
    let target = dir.path().join("target.vcf");
    fs::write(&target, b"other").unwrap();
    let link = dir.path().join("link.vcf");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let before = sha256_hex(&input);
    let out = run_cribar(&[
        "cribar",
        input.to_str().unwrap(),
        "-o",
        link.to_str().unwrap(),
    ]);
    assert_ne!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(sha256_hex(&input), before);
}

#[test]
fn ruta_relativa_equivale_a_entrada() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path(), "x.vcf");
    let before = sha256_hex(&input);
    let nested = dir.path().join("sub");
    fs::create_dir_all(&nested).unwrap();
    let rel = nested.join("..").join("x.vcf");
    let out = run_cribar(&[
        "cribar",
        input.to_str().unwrap(),
        "-o",
        rel.to_str().unwrap(),
    ]);
    assert_ne!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(sha256_hex(&input), before);
}

#[test]
fn salidas_modo_0600() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path(), "in.vcf");
    let out_vcf = dir.path().join("out.vcf");
    let audit = dir.path().join("audit.tsv");
    let out = run_cribar(&[
        "cribar",
        input.to_str().unwrap(),
        "-o",
        out_vcf.to_str().unwrap(),
        "-a",
        audit.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let mode_o = fs::metadata(&out_vcf).unwrap().permissions().mode();
    let mode_a = fs::metadata(&audit).unwrap().permissions().mode();
    assert_eq!(mode_o & 0o077, 0, "out mode={mode_o:#o}");
    assert_eq!(mode_a & 0o077, 0, "audit mode={mode_a:#o}");
}

#[test]
fn parse_tel_no_filtra_digitos_en_error() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("bad.vcf");
    fs::write(
        &input,
        "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Bad\r\nTEL;X-+34600000000\r\nEND:VCARD\r\n",
    )
    .unwrap();
    let out = run_cribar(&["cribar", input.to_str().unwrap(), "--dry-run"]);
    assert_ne!(out.status.code(), Some(0));
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("34600000000"),
        "el error no debe contener el valor TEL: {combined}"
    );
}
