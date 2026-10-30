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
# BL-1: con ZEDAZO_OFFLINE=1 no se instala nada (debe venir del paso CI previo).
TAURI_DRIVER_VERSION="${TAURI_DRIVER_VERSION:-2.0.6}"
E2E_DIR="$ROOT/scripts/desktop/e2e"
if [[ "${ZEDAZO_OFFLINE:-}" == "1" ]]; then
  if ! cargo install --list 2>/dev/null | grep -qE "^tauri-driver v${TAURI_DRIVER_VERSION}:"; then
    echo "error: falta tauri-driver =${TAURI_DRIVER_VERSION} (ZEDAZO_OFFLINE=1; instálalo antes de cortar la red)" >&2
    exit 1
  fi
  if [[ ! -d "$E2E_DIR/node_modules" ]]; then
    echo "error: falta $E2E_DIR/node_modules (ZEDAZO_OFFLINE=1; npm ci previo)" >&2
    exit 1
  fi
else
  if ! cargo install --list 2>/dev/null | grep -qE "^tauri-driver v${TAURI_DRIVER_VERSION}:"; then
    echo "==> instalando tauri-driver =${TAURI_DRIVER_VERSION}"
    cargo install tauri-driver --version "=${TAURI_DRIVER_VERSION}" --locked --force
  fi
  (
    cd "$E2E_DIR"
    if [[ ! -d node_modules ]]; then
      npm ci --ignore-scripts --no-fund --no-audit
    fi
  )
fi

# Extract/WebKit desde run-env; identidad va embebida en el binario (no reinyectar SHA).
if [[ -f dist/desktop/run-env.sh ]]; then
  # shellcheck disable=SC1091
  # shellcheck source=/dev/null
  source <(grep -E '^export (APPIMAGE_EXTRACT_AND_RUN|WEBKIT_DISABLE_DMABUF_RENDERER)=' dist/desktop/run-env.sh || true)
fi

# Autonomía: el AppImage debe servir GUI desde bundle.resources.
# Solo inyectar STATIC_DIR con ZEDAZO_ALLOW_EXTERNAL_GUI=1 (depuración).
if [[ "${ZEDAZO_ALLOW_EXTERNAL_GUI:-}" == "1" ]]; then
  if [[ -f dist/desktop/gui/index.html ]]; then
    export ZEDAZO_STATIC_DIR="$ROOT/dist/desktop/gui"
  elif [[ -f apps/web/out/index.html ]]; then
    export ZEDAZO_STATIC_DIR="$ROOT/apps/web/out"
  fi
  if [[ -z "${ZEDAZO_STATIC_DIR:-}" || ! -f "${ZEDAZO_STATIC_DIR}/index.html" ]]; then
    echo "error: ZEDAZO_ALLOW_EXTERNAL_GUI=1 pero falta dist/desktop/gui o apps/web/out" >&2
    exit 1
  fi
  echo "==> ZEDAZO_ALLOW_EXTERNAL_GUI=1 → STATIC_DIR=${ZEDAZO_STATIC_DIR}"
else
  unset ZEDAZO_STATIC_DIR || true
  echo "==> e2e autónomo (sin ZEDAZO_STATIC_DIR; resources del AppImage)"
fi

ENV_OUT="$(mktemp -t zedazo-e2e-env.XXXXXX)"
cleanup_env() { rm -f "$ENV_OUT"; }
trap cleanup_env EXIT

xvfb-run -a -s "-screen 0 1280x800x24" env \
  APPIMAGE_EXTRACT_AND_RUN="$APPIMAGE_EXTRACT_AND_RUN" \
  WEBKIT_DISABLE_DMABUF_RENDERER="$WEBKIT_DISABLE_DMABUF_RENDERER" \
  TAURI_DRIVER_PORT="$TAURI_DRIVER_PORT" \
  TAURI_APPIMAGE="$TAURI_APPIMAGE" \
  ZEDAZO_SAMPLE_VCF="$ZEDAZO_SAMPLE_VCF" \
  ZEDAZO_E2E_ENV_OUT="$ENV_OUT" \
  ${ZEDAZO_STATIC_DIR:+ZEDAZO_STATIC_DIR="$ZEDAZO_STATIC_DIR"} \
  ZEDAZO_RUN_PARITY="${ZEDAZO_RUN_PARITY:-}" \
  bash "$ROOT/scripts/desktop/e2e-tauri-driver-inner.sh"
