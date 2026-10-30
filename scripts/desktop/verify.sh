#!/usr/bin/env bash
# Fachada: verificar AppImage (smoke autónomo; e2e opt-in).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

SHA_BASE="${ZEDAZO_SHA_BASE:-$(git rev-parse HEAD 2>/dev/null || echo unknown)}"
START=$(date +%s)

APPIMAGE="${APPIMAGE:-}"
if [[ -z "$APPIMAGE" ]]; then
  APPIMAGE="$(find dist/desktop -maxdepth 1 -name 'zedazo-*.AppImage' -type f 2>/dev/null | sort | head -1 || true)"
fi

if [[ -z "$APPIMAGE" || ! -f "$APPIMAGE" ]]; then
  echo "result=blocked reason=falta_AppImage (ejecuta scripts/desktop/build-appimage.sh)" >&2
  exit 2
fi

echo "==> desktop-verify smoke autónomo sha=${SHA_BASE} appimage=$(basename "$APPIMAGE")"
APPIMAGE="$APPIMAGE" bash "$ROOT/scripts/desktop/smoke-xvfb.sh"

if [[ "${ZEDAZO_DESKTOP_E2E:-}" == "1" ]]; then
  echo "==> desktop-verify e2e (ZEDAZO_DESKTOP_E2E=1)"
  TAURI_APPIMAGE="$APPIMAGE" ZEDAZO_RUN_PARITY="${ZEDAZO_RUN_PARITY:-0}" \
    bash "$ROOT/scripts/desktop/run-e2e.sh"
fi

if [[ "${ZEDAZO_DESKTOP_OFFLINE:-}" == "1" ]]; then
  echo "==> desktop-verify offline"
  TAURI_APPIMAGE="$APPIMAGE" bash "$ROOT/scripts/desktop/e2e-no-network.sh"
fi

END=$(date +%s)
echo "result=passed duration_s=$((END - START)) sha=${SHA_BASE}"
echo "OK desktop-verify"
