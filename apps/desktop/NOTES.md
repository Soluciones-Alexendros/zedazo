# zedazo-desktop (W4)

## Gates locales

```bash
# Requiere WebKitGTK (scripts/desktop/install-linux-deps.sh). El workspace
# excluye este crate en `make check` / CI Check|Clippy|Test (sin GTK).
cargo check -p zedazo-desktop
cargo test -p zedazo-desktop --lib
```

## AppImage / WebKit

- Target de bundle: AppImage (`tauri.conf.json` → `bundle.targets`).
- Build completo: `cargo tauri build` (requiere WebKitGTK 4.1, librsvg, patchelf, etc.).
- Si falta WebKit en el host de desarrollo, el **gate W4** es `cargo check` + tests 401/403; el empaquetado AppImage queda para CI (W5A) en `ubuntu-22.04`.
- En Ubuntu 26 de prueba: FUSE o `--appimage-extract-and-run` (ver guía W5B).

## GUI estática

- Preferencia: `ZEDAZO_STATIC_DIR` o `apps/web/out` tras `output: "export"` (W2).
- Fallback: stub `apps/desktop/gui/` (solo humo hasta W2).
