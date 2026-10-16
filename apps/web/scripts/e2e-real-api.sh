#!/usr/bin/env bash
# Arranca zedazo-api + next dev (rewrite /api → 8080) y Playwright real-api.
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
# next dev reescribe /api → loopback (same-origin relativo en el cliente).
./node_modules/.bin/next dev -p 3000 >/tmp/zedazo-e2e-next.log 2>&1 &
WEB_PID=$!
for _ in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:3000" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

export ZEDAZO_E2E_REAL_API=1
export PLAYWRIGHT_BASE_URL=http://127.0.0.1:3000
# Sin webServer embebido: ya hay next + api.
./node_modules/.bin/playwright test e2e/real-api.spec.ts --config=playwright.real-api.config.ts
