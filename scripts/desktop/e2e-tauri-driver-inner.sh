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
  rm -f "$DRIVER_LOG"
}
trap cleanup EXIT

echo "==> tauri-driver :${TAURI_DRIVER_PORT} (DISPLAY=${DISPLAY:-unset})"
tauri-driver --port "$TAURI_DRIVER_PORT" >"$DRIVER_LOG" 2>&1 &
DRIVER_PID=$!
sleep 2
if ! kill -0 "$DRIVER_PID" 2>/dev/null; then
  echo "error: tauri-driver no arrancó" >&2
  cat "$DRIVER_LOG" >&2 || true
  exit 1
fi

echo "==> webdriverio i7-sample.mjs"
node "$E2E_DIR/i7-sample.mjs"
