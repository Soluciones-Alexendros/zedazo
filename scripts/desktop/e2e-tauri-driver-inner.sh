#!/usr/bin/env bash
# Inner: asume DISPLAY ya puesto por xvfb-run.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
E2E_DIR="$ROOT/scripts/desktop/e2e"
DRIVER_LOG="$(mktemp)"
DRIVER_PID=""

cleanup() {
  if [[ -n "$DRIVER_PID" ]] && kill -0 "$DRIVER_PID" 2>/dev/null; then
    kill "$DRIVER_PID" 2>/dev/null || true
    wait "$DRIVER_PID" 2>/dev/null || true
  fi
  # Conservar log de driver ante fallo; borrar si OK.
  if [[ "${E2E_KEEP_DRIVER_LOG:-}" == "1" || "${E2E_FAILED:-0}" == "1" ]]; then
    echo "==> tauri-driver log (redactado) en $DRIVER_LOG" >&2
    # No filtramos tokens del driver; el mjs ya no los imprime en claro.
    tail -n 80 "$DRIVER_LOG" >&2 || true
  else
    rm -f "$DRIVER_LOG"
  fi
}
trap cleanup EXIT

echo "==> fase: arranque tauri-driver :${TAURI_DRIVER_PORT} (DISPLAY=${DISPLAY:-unset})"
tauri-driver --port "$TAURI_DRIVER_PORT" >"$DRIVER_LOG" 2>&1 &
DRIVER_PID=$!

# Readiness del driver (puerto abierto), sin sleep fijo como única sincronización.
ready=0
for _ in $(seq 1 40); do
  if ! kill -0 "$DRIVER_PID" 2>/dev/null; then
    echo "error: tauri-driver terminó al arrancar" >&2
    cat "$DRIVER_LOG" >&2 || true
    E2E_FAILED=1
    exit 1
  fi
  if (echo >/dev/tcp/127.0.0.1/"$TAURI_DRIVER_PORT") >/dev/null 2>&1; then
    ready=1
    break
  fi
  # Fallback sin bash /dev/tcp
  if command -v nc >/dev/null 2>&1 && nc -z 127.0.0.1 "$TAURI_DRIVER_PORT" 2>/dev/null; then
    ready=1
    break
  fi
  sleep 0.25
done
if [[ "$ready" != "1" ]]; then
  echo "error: tauri-driver no escuchó en :${TAURI_DRIVER_PORT}" >&2
  cat "$DRIVER_LOG" >&2 || true
  E2E_FAILED=1
  exit 1
fi
echo "==> fase: driver listo"

echo "==> fase: webdriverio i7-sample.mjs"
set +e
node "$E2E_DIR/i7-sample.mjs"
STATUS=$?
set -e
if [[ "$STATUS" -ne 0 ]]; then
  E2E_FAILED=1
  exit "$STATUS"
fi
echo "==> fase: e2e OK"
