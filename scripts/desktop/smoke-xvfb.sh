#!/usr/bin/env bash
# Smoke AppImage bajo Xvfb: arranca, espera log loopback, health + version, apaga.
# Por defecto NO inyecta ZEDAZO_STATIC_DIR (prueba autonomía del AppImage).
# Uso: APPIMAGE=dist/desktop/*.AppImage ./scripts/desktop/smoke-xvfb.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# Solo extract/WebKit desde run-env; NO reexportar SHA/fecha como prueba de identidad.
if [[ -f dist/desktop/run-env.sh ]]; then
  # shellcheck disable=SC1091
  # shellcheck source=/dev/null
  source <(grep -E '^export (APPIMAGE_EXTRACT_AND_RUN|WEBKIT_DISABLE_DMABUF_RENDERER)=' dist/desktop/run-env.sh || true)
fi

APPIMAGE="${APPIMAGE:-}"
if [[ -z "$APPIMAGE" ]]; then
  APPIMAGE="$(find dist/desktop -maxdepth 1 -name 'zedazo-*.AppImage' -type f | sort | head -1 || true)"
fi
if [[ -z "$APPIMAGE" || ! -f "$APPIMAGE" ]]; then
  echo "error: no hay AppImage (pasa APPIMAGE=... o genera dist/desktop/)" >&2
  exit 1
fi
chmod +x "$APPIMAGE"

export APPIMAGE_EXTRACT_AND_RUN="${APPIMAGE_EXTRACT_AND_RUN:-1}"
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"

# Autonomía: sin STATIC_DIR salvo ZEDAZO_ALLOW_EXTERNAL_GUI=1 (depura harness).
if [[ "${ZEDAZO_ALLOW_EXTERNAL_GUI:-}" == "1" ]]; then
  if [[ -f dist/desktop/gui/index.html ]]; then
    export ZEDAZO_STATIC_DIR="${ROOT}/dist/desktop/gui"
  elif [[ -z "${ZEDAZO_STATIC_DIR:-}" && -f apps/web/out/index.html ]]; then
    export ZEDAZO_STATIC_DIR="${ROOT}/apps/web/out"
  fi
  echo "==> ZEDAZO_ALLOW_EXTERNAL_GUI=1 → STATIC_DIR=${ZEDAZO_STATIC_DIR:-unset}"
else
  unset ZEDAZO_STATIC_DIR || true
  echo "==> smoke autónomo (sin ZEDAZO_STATIC_DIR)"
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
# HOME/XDG limpios para no depender de estado del runner.
SMOKE_HOME="$(mktemp -d)"
export HOME="$SMOKE_HOME"
export XDG_CONFIG_HOME="$SMOKE_HOME/.config"
export XDG_DATA_HOME="$SMOKE_HOME/.local/share"
export XDG_STATE_HOME="$SMOKE_HOME/.local/state"

xvfb-run -a -s "-screen 0 1280x800x24" \
  env RUST_LOG=zedazo_desktop=info \
  HOME="$HOME" \
  XDG_CONFIG_HOME="$XDG_CONFIG_HOME" \
  XDG_DATA_HOME="$XDG_DATA_HOME" \
  XDG_STATE_HOME="$XDG_STATE_HOME" \
  APPIMAGE_EXTRACT_AND_RUN="$APPIMAGE_EXTRACT_AND_RUN" \
  WEBKIT_DISABLE_DMABUF_RENDERER="$WEBKIT_DISABLE_DMABUF_RENDERER" \
  ${ZEDAZO_STATIC_DIR:+ZEDAZO_STATIC_DIR="$ZEDAZO_STATIC_DIR"} \
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

echo "==> version sin Bearer → 401"
VER_CODE="$(curl -sS -o /tmp/zedazo-version.body -w '%{http_code}' \
  --connect-timeout 3 --max-time 10 \
  -H "Host: ${ADDR}" \
  "http://${ADDR}/api/v1/version" || true)"
echo "version HTTP ${VER_CODE}"
if [[ "$VER_CODE" != "401" ]]; then
  echo "error: version sin Bearer debía ser 401 (got ${VER_CODE})" >&2
  cat /tmp/zedazo-version.body >&2 || true
  exit 1
fi

# GUI real: GET / debe devolver HTML con marca zedazo (no stub vacío).
echo "==> GUI GET /"
GUI_CODE="$(curl -sS -o /tmp/zedazo-gui.body -w '%{http_code}' \
  --connect-timeout 3 --max-time 10 \
  -H "Host: ${ADDR}" \
  "http://${ADDR}/" || true)"
if [[ "$GUI_CODE" != "200" ]]; then
  echo "error: GUI / HTTP ${GUI_CODE}" >&2
  cat /tmp/zedazo-gui.body >&2 || true
  cat "$LOG" >&2 || true
  exit 1
fi
if ! grep -qiE 'zedazo|<!doctype html' /tmp/zedazo-gui.body; then
  echo "error: cuerpo de / no parece GUI HTML" >&2
  head -c 400 /tmp/zedazo-gui.body >&2 || true
  exit 1
fi
if grep -q 'Escritorio local (stub)' /tmp/zedazo-gui.body; then
  echo "error: AppImage sirve stub de desarrollo; resources GUI ausentes" >&2
  exit 1
fi
echo "OK GUI /"

mkdir -p dist/desktop
echo "${ADDR}" >dist/desktop/last-addr.txt
echo "OK smoke xvfb autónomo (${ADDR})"
rm -rf "$SMOKE_HOME" || true
