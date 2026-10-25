#!/usr/bin/env bash
# Smoke AppImage bajo Xvfb: arranca, espera log loopback, GET /api/v1/health, apaga.
# Uso: APPIMAGE=dist/desktop/*.AppImage ./scripts/desktop/smoke-xvfb.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if [[ -f dist/desktop/run-env.sh ]]; then
  # shellcheck disable=SC1091
  source dist/desktop/run-env.sh
fi

APPIMAGE="${APPIMAGE:-}"
if [[ -z "$APPIMAGE" ]]; then
  APPIMAGE="$(find dist/desktop -maxdepth 1 -name 'zedazo-*.AppImage' -type f | sort | head -1 || true)"
fi
if [[ -z "$APPIMAGE" || ! -f "$APPIMAGE" ]]; then
  echo "error: no hay AppImage (pasa APPIMAGE=... o genera dist/desktop/)" >&2
  exit 1
fi
# download-artifact no conserva el bit de ejecución.
chmod +x "$APPIMAGE"

export APPIMAGE_EXTRACT_AND_RUN="${APPIMAGE_EXTRACT_AND_RUN:-1}"
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
if [[ -f dist/desktop/gui/index.html ]]; then
  export ZEDAZO_STATIC_DIR="${ROOT}/dist/desktop/gui"
elif [[ -z "${ZEDAZO_STATIC_DIR:-}" && -f apps/web/out/index.html ]]; then
  export ZEDAZO_STATIC_DIR="${ROOT}/apps/web/out"
fi

LOG="$(mktemp)"
PID=""
cleanup() {
  if [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
  rm -f "$LOG"
}
trap cleanup EXIT

echo "==> xvfb smoke: $APPIMAGE"
xvfb-run -a -s "-screen 0 1280x800x24" \
  env RUST_LOG=zedazo_desktop=info \
  "$APPIMAGE" >"$LOG" 2>&1 &
PID=$!

ADDR=""
for _ in $(seq 1 90); do
  if ! kill -0 "$PID" 2>/dev/null; then
    echo "error: AppImage terminó antes del smoke" >&2
    cat "$LOG" >&2 || true
    exit 1
  fi
  if command -v rg >/dev/null 2>&1; then
    ADDR="$(rg -o '127\.0\.0\.1:[0-9]+' "$LOG" | head -1 || true)"
  else
    ADDR="$(grep -Eo '127\.0\.0\.1:[0-9]+' "$LOG" | head -1 || true)"
  fi
  if [[ -n "$ADDR" ]]; then
    break
  fi
  sleep 1
done

if [[ -z "$ADDR" ]]; then
  echo "error: no se detectó addr loopback en el log" >&2
  cat "$LOG" >&2 || true
  exit 1
fi

echo "==> health en http://${ADDR}/api/v1/health"
CODE="$(curl -sS -o /tmp/zedazo-health.body -w '%{http_code}' \
  --connect-timeout 3 --max-time 10 \
  -H "Host: ${ADDR}" \
  "http://${ADDR}/api/v1/health" || true)"
echo "HTTP ${CODE}"
case "$CODE" in
  200) ;;
  *)
    echo "error: health inesperado (${CODE})" >&2
    cat /tmp/zedazo-health.body >&2 || true
    cat "$LOG" >&2 || true
    exit 1
    ;;
esac

# Metadatos de build si el smoke reexportó ZEDAZO_* (endpoint version es protegido → 401 sin Bearer OK)
VER_CODE="$(curl -sS -o /tmp/zedazo-version.body -w '%{http_code}' \
  --connect-timeout 3 --max-time 10 \
  -H "Host: ${ADDR}" \
  "http://${ADDR}/api/v1/version" || true)"
echo "version HTTP ${VER_CODE} (esperado 401 sin Bearer)"

echo "${ADDR}" >dist/desktop/last-addr.txt
echo "OK smoke xvfb (${ADDR})"
