# Issue propuesto: upgrade Tauri/gtk-rs (glib y proc-macro-error)

**Estado (2026-11-18):** aplazado. No crear el issue en GitHub hasta verificar y profundizar el upgrade Tauri/gtk-rs. Este fichero es el borrador.

## Título

Upgrade Tauri/gtk-rs para salir de `glib` 0.18.5 y `proc-macro-error` 1.0.4

## Cuerpo

La revisión del 2026-11-18 mantiene los ignores de `RUSTSEC-2024-0429` y `RUSTSEC-2024-0370`.

- `glib` 0.18.5 sigue en `Cargo.lock` (Tauri/wry). El advisory unsound se parchea en `glib` >= 0.20.0, que este grafo no tiene.
- `proc-macro-error` 1.0.4 sigue como transitiva de Tauri. RustSec marca `patched = []`.
- `cargo audit` no falla por ellos (avisos informativos e ignore). `cargo deny` avisa `advisory-not-detected` en 0429 porque el check de advisories no trata `unsound` como vulnerability; el ignore se conserva para no divergir de `.cargo/audit.toml`.

Próxima revisión documentada: 2027-02-18.

Al subir Tauri/gtk-rs, quitar los mismos IDs de `deny.toml`, `.cargo/audit.toml` y `osv-scanner.toml` en un solo cambio, y repetir `make deny` y `make audit`.
