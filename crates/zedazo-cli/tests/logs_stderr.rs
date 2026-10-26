//! N-07 / B1: los logs de tracing van a stderr, no a stdout.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_zedazo"))
}

#[test]
fn cribar_info_logs_go_to_stderr_not_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.vcf");
    let output = dir.path().join("o.vcf");
    let audit = dir.path().join("a.tsv");
    std::fs::write(
        &input,
        "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:log-stderr\r\nFN:Example Org\r\nN:Org;Example;;;\r\nEMAIL:contact@example.invalid\r\nTEL;TYPE=cell:+34600000000\r\nEND:VCARD\r\n",
    )
    .unwrap();

    let out = Command::new(bin())
        .args([
            "cribar",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-a",
            audit.to_str().unwrap(),
        ])
        .output()
        .expect("ejecutar zedazo");

    assert!(
        out.status.success(),
        "cribar falló: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    let stdout_logs = stdout
        .lines()
        .filter(|l| l.contains(" INFO ") || l.contains(" WARN ") || l.contains(" ERROR "))
        .count();
    assert_eq!(
        stdout_logs, 0,
        "stdout no debe llevar logs de tracing; stdout={stdout:?}"
    );

    let stderr_info = stderr.lines().filter(|l| l.contains(" INFO ")).count();
    assert!(
        stderr_info >= 1,
        "stderr debe llevar al menos un INFO; stderr={stderr:?}"
    );
}
