#!/usr/bin/env bash
# Fachada local: preflight de escritorio (sin publicar ni mutar remoto).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

UNIT="${ZEDAZO_UNIT:-preflight}"
SHA_BASE="${ZEDAZO_SHA_BASE:-$(git rev-parse HEAD 2>/dev/null || echo unknown)}"
START=$(date +%s)
RESULT=passed
REASON=""

echo "==> desktop-preflight unit=${UNIT} sha=${SHA_BASE}"

fail() {
  RESULT=failed
  REASON="$1"
  echo "result=${RESULT} reason=${REASON}" >&2
  exit 1
}

command -v cargo >/dev/null || fail "falta cargo"
command -v pnpm >/dev/null || fail "falta pnpm"
command -v xvfb-run >/dev/null || fail "falta xvfb-run"

if [[ ! -f apps/desktop/tauri.conf.json ]]; then
  fail "falta apps/desktop"
fi

# Lockfile web presente y usable en freeze.
if [[ ! -f apps/web/pnpm-lock.yaml ]]; then
  fail "falta apps/web/pnpm-lock.yaml"
fi

# Test negativo del gate local (sin fabricar checks en GitHub).
bash "$ROOT/scripts/desktop/check-ci-gate-logic.sh" || fail "check-ci-gate-logic"

# Lockfile: mutación temporal debe romper --frozen-lockfile; no deja cambios.
bash "$ROOT/scripts/desktop/check-frozen-lockfile.sh" || fail "check-frozen-lockfile"

END=$(date +%s)
echo "result=${RESULT} duration_s=$((END - START)) sha=${SHA_BASE}"
echo "OK desktop-preflight"
