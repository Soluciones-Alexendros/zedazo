#!/usr/bin/env bash
# Wrapper: tauri-driver + I7 bajo un único Xvfb (ver e2e-tauri-driver.sh).
# ZEDAZO_RUN_PARITY=1 → también CLI↔API tras el e2e (misma sesión).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
exec bash "$ROOT/scripts/desktop/e2e-tauri-driver.sh" "$@"
