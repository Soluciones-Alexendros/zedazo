#!/usr/bin/env bash
# E2E mínimo: un solo zedazo-api sirve apps/web/out (sin Caddy ni serve-out).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
WEB="$ROOT/apps/web"
DATA="$(mktemp -d)"
API_PID=""
BIND="${ZEDAZO_E2E_BIND:-127.0.0.1:8080}"
BASE="http://${BIND}"

cleanup() {
  if [[ -n "$API_PID" ]]; then kill "$API_PID" 2>/dev/null || true; fi
  rm -rf "$DATA"
}
trap cleanup EXIT

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
cargo build -p zedazo-api --all-features

cd "$WEB"
if [[ ! -d out ]] || [[ "${ZEDAZO_E2E_REBUILD:-}" == "1" ]]; then
  if [[ ! -x ./node_modules/.bin/next ]]; then
    echo "falta apps/web/node_modules (pnpm install) o out/; o copia un out/ válido" >&2
    exit 1
  fi
  ./node_modules/.bin/next build
fi
test -f out/index.html

ZEDAZO_DATA_DIR="$DATA" ZEDAZO_BIND="$BIND" ZEDAZO_AUTH_MODE=disabled \
  ZEDAZO_STATIC_DIR="$WEB/out" \
  "$CARGO_TARGET_DIR/debug/zedazo-api" >/tmp/zedazo-e2e-api-gui.log 2>&1 &
API_PID=$!

ok=0
for _ in $(seq 1 40); do
  if curl -fsS "$BASE/api/v1/health" >/dev/null 2>&1; then
    ok=1
    break
  fi
  sleep 0.5
done
test "$ok" -eq 1 || { echo "API health timeout"; tail -50 /tmp/zedazo-e2e-api-gui.log || true; exit 1; }

hdr="$(curl -sS -D- -o /tmp/zedazo-e2e-gui-body.html "$BASE/")"
echo "$hdr" | grep -qi 'HTTP/1.1 200'
grep -qi 'x-content-type-options: nosniff' <<<"$hdr"
grep -qi '<!doctype html\|<html' /tmp/zedazo-e2e-gui-body.html

curl -fsS "$BASE/api/v1/health" | grep -q '"status":"ok"'

miss_hdr="$(curl -sS -D- -o /tmp/zedazo-e2e-miss.body "$BASE/api/v1/missing")"
echo "$miss_hdr" | grep -qi 'HTTP/1.1 404'
! grep -qi '<!doctype html\|<html' /tmp/zedazo-e2e-miss.body

spa_code="$(curl -sS -o /dev/null -w '%{http_code}' "$BASE/procesar")"
test "$spa_code" = "200"

echo "e2e-api-serves-gui: OK (un binario, sin Caddy)"
