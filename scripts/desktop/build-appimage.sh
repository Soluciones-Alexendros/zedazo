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

echo "==> pnpm build apps/web (output: export → out/; lockfile congelado)"
(
  cd apps/web
  CI=true pnpm install --frozen-lockfile
  pnpm exec next build
)
test -f apps/web/out/index.html

echo "==> sincronizar out/ → apps/desktop/gui/ (frontendDist)"
# Conservar stub mínimo si el export fallara a medias; aquí out/ ya validado.
rsync -a --delete \
  --exclude '.git' \
  apps/web/out/ \
  apps/desktop/gui/

# Copia para artefacto CI / depuración (ZEDAZO_ALLOW_EXTERNAL_GUI=1).
# Producción: bundle.resources gui/**/* → resource_dir; smoke/e2e autónomos no usan STATIC_DIR.
mkdir -p dist/desktop
rsync -a --delete \
  --exclude '.git' \
  apps/web/out/ \
  dist/desktop/gui/

# S-10: tauri-cli pin exacto (no ^2) + herramientas linuxdeploy con SHA256.
TAURI_CLI_VERSION="${TAURI_CLI_VERSION:-2.11.5}"
bash "$ROOT/scripts/desktop/prefetch-tauri-tools.sh"
if ! command -v cargo-tauri >/dev/null 2>&1 && ! cargo tauri --version >/dev/null 2>&1; then
  echo "==> instalando tauri-cli =${TAURI_CLI_VERSION}"
  cargo install tauri-cli --version "=${TAURI_CLI_VERSION}" --locked
else
  # Reinstalar si la versión instalada no coincide exactamente.
  INSTALLED="$(cargo tauri --version 2>/dev/null | awk '{print $NF}' || true)"
  if [[ "$INSTALLED" != "$TAURI_CLI_VERSION" ]]; then
    echo "==> tauri-cli ${INSTALLED:-ausente} ≠ ${TAURI_CLI_VERSION}; reinstalar"
    cargo install tauri-cli --version "=${TAURI_CLI_VERSION}" --locked --force
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

# Env file para smoke/e2e. STATIC_DIR relativo al checkout del runner (no path absoluto del builder).
{
  echo "export ZEDAZO_GIT_SHA=${ZEDAZO_GIT_SHA}"
  echo "export ZEDAZO_BUILD_DATE=${ZEDAZO_BUILD_DATE}"
  echo "# Preferir dist/desktop/gui (artefacto) o apps/web/out en el mismo checkout."
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
