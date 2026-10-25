#!/usr/bin/env bash
# Arranca zedazo-api, sirve el export estático `out/` (proxy /api) y Playwright real-api.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
WEB="$ROOT/apps/web"
DATA="$(mktemp -d)"
API_PID=""
WEB_PID=""

cleanup() {
  if [[ -n "$WEB_PID" ]]; then kill "$WEB_PID" 2>/dev/null || true; fi
  if [[ -n "$API_PID" ]]; then kill "$API_PID" 2>/dev/null || true; fi
  rm -rf "$DATA"
}
trap cleanup EXIT

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
cargo build -p zedazo-api --all-features
ZEDAZO_DATA_DIR="$DATA" ZEDAZO_BIND=127.0.0.1:8080 ZEDAZO_AUTH_MODE=disabled \
  "$CARGO_TARGET_DIR/debug/zedazo-api" &
API_PID=$!

for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:8080/api/v1/health" >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done
curl -fsS "http://127.0.0.1:8080/api/v1/health" >/dev/null

cd "$WEB"
if [[ ! -d out ]] || [[ "${ZEDAZO_E2E_REBUILD:-}" == "1" ]]; then
  ./node_modules/.bin/next build
fi

ZEDAZO_PROXY_API=1 ZEDAZO_API_ORIGIN=http://127.0.0.1:8080 PORT=3000 HOST=127.0.0.1 \
  node scripts/serve-out.mjs >/tmp/zedazo-e2e-static.log 2>&1 &
WEB_PID=$!

for _ in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:3000" >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done

ok=0
for _ in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:3000/api/v1/health" >/dev/null 2>&1; then
    ok=1
    break
  fi
  sleep 0.5
done
test "$ok" -eq 1 || { echo "proxy /api health timeout"; tail -50 /tmp/zedazo-e2e-static.log || true; exit 1; }

export ZEDAZO_E2E_REAL_API=1
unset NEXT_PUBLIC_API_BASE
export PLAYWRIGHT_BASE_URL=http://127.0.0.1:3000
./node_modules/.bin/playwright test e2e/real-api.spec.ts --config=playwright.real-api.config.ts
