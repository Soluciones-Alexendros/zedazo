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

# S-10: tauri-driver pin exacto (no latest / no ^).
TAURI_DRIVER_VERSION="${TAURI_DRIVER_VERSION:-2.0.6}"
if ! cargo install --list 2>/dev/null | grep -qE "^tauri-driver v${TAURI_DRIVER_VERSION}:"; then
  echo "==> instalando tauri-driver =${TAURI_DRIVER_VERSION}"
  cargo install tauri-driver --version "=${TAURI_DRIVER_VERSION}" --locked --force
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
# ServeDir necesita ficheros en disco. La GUI del AppImage va embebida en el binario
# Tauri (no hay .html al --appimage-extract); el artefacto sube dist/desktop/gui/.
if [[ -f dist/desktop/gui/index.html ]]; then
  export ZEDAZO_STATIC_DIR="$ROOT/dist/desktop/gui"
elif [[ -f apps/web/out/index.html ]]; then
  export ZEDAZO_STATIC_DIR="$ROOT/apps/web/out"
fi
if [[ -z "${ZEDAZO_STATIC_DIR:-}" || ! -f "${ZEDAZO_STATIC_DIR}/index.html" ]]; then
  echo "error: falta GUI estática (dist/desktop/gui o apps/web/out) para ZEDAZO_STATIC_DIR" >&2
  exit 1
fi
echo "==> ZEDAZO_STATIC_DIR=${ZEDAZO_STATIC_DIR}"

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
