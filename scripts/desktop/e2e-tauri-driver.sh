#!/usr/bin/env bash
# Orquesta tauri-driver + WebdriverIO (sample.vcf + I7) bajo un único Xvfb.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

APPIMAGE="${TAURI_APPIMAGE:-${APPIMAGE:-}}"
if [[ -z "$APPIMAGE" ]]; then
  APPIMAGE="$(find dist/desktop -maxdepth 1 -name 'zedazo-*.AppImage' -type f | sort | head -1 || true)"
fi
if [[ -z "$APPIMAGE" || ! -f "$APPIMAGE" ]]; then
  echo "error: falta AppImage (TAURI_APPIMAGE=...)" >&2
  exit 1
fi
APPIMAGE="$(cd "$(dirname "$APPIMAGE")" && pwd)/$(basename "$APPIMAGE")"
# download-artifact no conserva el bit de ejecución.
chmod +x "$APPIMAGE"

export APPIMAGE_EXTRACT_AND_RUN="${APPIMAGE_EXTRACT_AND_RUN:-1}"
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
export TAURI_DRIVER_PORT="${TAURI_DRIVER_PORT:-4444}"
export TAURI_APPIMAGE="$APPIMAGE"
export ZEDAZO_SAMPLE_VCF="${ZEDAZO_SAMPLE_VCF:-$ROOT/examples/sample.vcf}"

if ! command -v tauri-driver >/dev/null 2>&1; then
  echo "==> instalando tauri-driver"
  cargo install tauri-driver --locked
fi

E2E_DIR="$ROOT/scripts/desktop/e2e"
(
  cd "$E2E_DIR"
  if [[ ! -d node_modules ]]; then
    npm install --no-fund --no-audit
  fi
)

# Todo el e2e comparte el mismo DISPLAY de Xvfb.
if [[ -f dist/desktop/run-env.sh ]]; then
  # shellcheck disable=SC1091
  source dist/desktop/run-env.sh
fi
if [[ -z "${ZEDAZO_STATIC_DIR:-}" && -f apps/web/out/index.html ]]; then
  export ZEDAZO_STATIC_DIR="$ROOT/apps/web/out"
fi
# En CI el job e2e solo descarga el AppImage: run-env puede apuntar a out/ inexistente.
# Extraer la GUI embebida (frontendDist) para ServeDir + deep links V-01.
if [[ -z "${ZEDAZO_STATIC_DIR:-}" || ! -f "${ZEDAZO_STATIC_DIR}/index.html" ]]; then
  echo "==> ZEDAZO_STATIC_DIR inválido; extrayendo GUI del AppImage"
  EXTRACT_GUI="$(mktemp -d "${TMPDIR:-/tmp}/zedazo-gui-XXXXXX")"
  (
    cd "$EXTRACT_GUI"
    # --appimage-extract escribe squashfs-root/ (no requiere FUSE).
    "$APPIMAGE" --appimage-extract >/dev/null
  )
  GUI_HTML="$(find "$EXTRACT_GUI/squashfs-root" -name 'ejecuciones.html' -type f 2>/dev/null | head -1 || true)"
  if [[ -z "$GUI_HTML" ]]; then
    GUI_HTML="$(find "$EXTRACT_GUI/squashfs-root" -path '*/gui/index.html' -type f 2>/dev/null | head -1 || true)"
  fi
  if [[ -z "$GUI_HTML" ]]; then
    GUI_HTML="$(find "$EXTRACT_GUI/squashfs-root" -name 'index.html' -type f 2>/dev/null | head -1 || true)"
  fi
  if [[ -z "$GUI_HTML" || ! -f "$GUI_HTML" ]]; then
    echo "error: no se encontró GUI (index/ejecuciones.html) dentro del AppImage" >&2
    find "$EXTRACT_GUI/squashfs-root" -name '*.html' 2>/dev/null | head -40 >&2 || true
    exit 1
  fi
  export ZEDAZO_STATIC_DIR="$(cd "$(dirname "$GUI_HTML")" && pwd)"
  echo "==> ZEDAZO_STATIC_DIR=${ZEDAZO_STATIC_DIR}"
fi

xvfb-run -a -s "-screen 0 1280x800x24" env \
  APPIMAGE_EXTRACT_AND_RUN="$APPIMAGE_EXTRACT_AND_RUN" \
  WEBKIT_DISABLE_DMABUF_RENDERER="$WEBKIT_DISABLE_DMABUF_RENDERER" \
  TAURI_DRIVER_PORT="$TAURI_DRIVER_PORT" \
  TAURI_APPIMAGE="$TAURI_APPIMAGE" \
  ZEDAZO_SAMPLE_VCF="$ZEDAZO_SAMPLE_VCF" \
  ZEDAZO_STATIC_DIR="${ZEDAZO_STATIC_DIR:-}" \
  ZEDAZO_GIT_SHA="${ZEDAZO_GIT_SHA:-}" \
  ZEDAZO_BUILD_DATE="${ZEDAZO_BUILD_DATE:-}" \
  ZEDAZO_RUN_PARITY="${ZEDAZO_RUN_PARITY:-}" \
  bash "$ROOT/scripts/desktop/e2e-tauri-driver-inner.sh"
