#!/usr/bin/env bash
# Start zedazo-api, Next.js development server, and Playwright real-api.
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
# Cliente → API real (auth disabled + CORS abierto). Más fiable en CI.
NEXT_PUBLIC_API_BASE=http://127.0.0.1:8080 \
  ./node_modules/.bin/next dev -p 3000 >/tmp/zedazo-e2e-next.log 2>&1 &
WEB_PID=$!
for _ in $(seq 1 90); do
  if curl -fsS "http://127.0.0.1:3000" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

export ZEDAZO_E2E_REAL_API=1
export NEXT_PUBLIC_API_BASE=http://127.0.0.1:8080
export PLAYWRIGHT_BASE_URL=http://127.0.0.1:3000
./node_modules/.bin/playwright test e2e/real-api.spec.ts --config=playwright.real-api.config.ts
