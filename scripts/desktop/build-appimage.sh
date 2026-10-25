#!/usr/bin/env bash
# Build AppImage zedazo-desktop (Tauri v2) con metadatos ZEDAZO_GIT_SHA / ZEDAZO_BUILD_DATE.
# Uso: desde la raíz del repo. Artefactos en dist/desktop/
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if [[ -z "${ZEDAZO_GIT_SHA:-}" ]]; then
  ZEDAZO_GIT_SHA="$(git rev-parse --short=12 HEAD)"
fi
if [[ -z "${ZEDAZO_BUILD_DATE:-}" ]]; then
  ZEDAZO_BUILD_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
fi
export ZEDAZO_GIT_SHA ZEDAZO_BUILD_DATE
# linuxdeploy strip puede fallar en ELF modernos (tauri#14796)
export NO_STRIP="${NO_STRIP:-true}"
export APPIMAGE_EXTRACT_AND_RUN="${APPIMAGE_EXTRACT_AND_RUN:-1}"

echo "ZEDAZO_GIT_SHA=${ZEDAZO_GIT_SHA}"
echo "ZEDAZO_BUILD_DATE=${ZEDAZO_BUILD_DATE}"

if [[ ! -d apps/desktop ]]; then
  echo "error: falta apps/desktop (Oleada W4)" >&2
  exit 1
fi

echo "==> pnpm build apps/web (output: export → out/)"
(
  cd apps/web
  if [[ ! -d node_modules ]]; then
    pnpm install --frozen-lockfile
  fi
  pnpm exec next build
)
test -f apps/web/out/index.html

echo "==> sincronizar out/ → apps/desktop/gui/ (frontendDist)"
# Conservar stub mínimo si el export fallara a medias; aquí out/ ya validado.
rsync -a --delete \
  --exclude '.git' \
  apps/web/out/ \
  apps/desktop/gui/

if ! command -v cargo-tauri >/dev/null 2>&1; then
  if ! cargo tauri --version >/dev/null 2>&1; then
    echo "==> instalando tauri-cli 2"
    cargo install tauri-cli --version "^2" --locked
  fi
fi

echo "==> cargo tauri build --bundles appimage"
(
  cd apps/desktop
  # Env visibles en runtime del proceso (versión API) cuando el smoke las reexporta.
  export ZEDAZO_GIT_SHA ZEDAZO_BUILD_DATE
  cargo tauri build --bundles appimage --ci
)

OUT_DIR="${ROOT}/dist/desktop"
mkdir -p "$OUT_DIR"

mapfile -t APPIMAGES < <(find "${ROOT}/target/release/bundle/appimage" -name '*.AppImage' -type f 2>/dev/null | sort)
if [[ ${#APPIMAGES[@]} -eq 0 ]]; then
  mapfile -t APPIMAGES < <(find "${ROOT}/target" -path '*/bundle/appimage/*.AppImage' -type f 2>/dev/null | sort)
fi
if [[ ${#APPIMAGES[@]} -eq 0 ]]; then
  echo "error: no se encontró ningún .AppImage tras el build" >&2
  find "${ROOT}/target" -name '*.AppImage' 2>/dev/null || true
  exit 1
fi

SRC="${APPIMAGES[0]}"
BASE="zedazo-${ZEDAZO_GIT_SHA}-x86_64.AppImage"
DEST="${OUT_DIR}/${BASE}"
cp -f "$SRC" "$DEST"
chmod +x "$DEST"

{
  echo "ZEDAZO_GIT_SHA=${ZEDAZO_GIT_SHA}"
  echo "ZEDAZO_BUILD_DATE=${ZEDAZO_BUILD_DATE}"
  echo "SOURCE_APPIMAGE=$(basename "$SRC")"
} >"${OUT_DIR}/build-info.txt"

# Env file para smoke/e2e en el mismo checkout (ServeDir + /version).
{
  echo "export ZEDAZO_GIT_SHA=${ZEDAZO_GIT_SHA}"
  echo "export ZEDAZO_BUILD_DATE=${ZEDAZO_BUILD_DATE}"
  echo "export ZEDAZO_STATIC_DIR=${ROOT}/apps/web/out"
  echo "export APPIMAGE_EXTRACT_AND_RUN=1"
  echo "export WEBKIT_DISABLE_DMABUF_RENDERER=1"
} >"${OUT_DIR}/run-env.sh"

(
  cd "$OUT_DIR"
  sha256sum "$BASE" build-info.txt >SHA256SUMS
)

echo "==> artefactos listos en ${OUT_DIR}"
ls -la "$OUT_DIR"
cat "${OUT_DIR}/SHA256SUMS"
