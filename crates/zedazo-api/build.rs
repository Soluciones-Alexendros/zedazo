//! Metadatos de build embebidos (SHA / fecha). Runtime env no sustituye el artefacto.

fn main() {
    println!("cargo:rerun-if-env-changed=ZEDAZO_GIT_SHA");
    println!("cargo:rerun-if-env-changed=ZEDAZO_BUILD_DATE");

    let sha = std::env::var("ZEDAZO_GIT_SHA").unwrap_or_else(|_| "unknown".into());
    let date = std::env::var("ZEDAZO_BUILD_DATE").unwrap_or_else(|_| "unknown".into());
    // Vacío → unknown (evita rustc-env con valor en blanco).
    let sha = if sha.trim().is_empty() {
        "unknown".into()
    } else {
        sha
    };
    let date = if date.trim().is_empty() {
        "unknown".into()
    } else {
        date
    };
    println!("cargo:rustc-env=ZEDAZO_GIT_SHA={sha}");
    println!("cargo:rustc-env=ZEDAZO_BUILD_DATE={date}");
}
