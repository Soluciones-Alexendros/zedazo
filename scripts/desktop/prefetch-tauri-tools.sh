#!/usr/bin/env bash
# Descarga herramientas AppImage versionadas a la caché de Tauri 2 (~/.cache/tauri)
# y verifica SHA256 (S-10). Falla si el checksum no coincide.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CHECKSUMS="${ROOT}/scripts/desktop/tool-checksums.sha256"
ARCH="${ZEDAZO_TOOLS_ARCH:-x86_64}"
CACHE="${XDG_CACHE_HOME:-${HOME}/.cache}/tauri"
mkdir -p "$CACHE"

if [[ ! -f "$CHECKSUMS" ]]; then
  echo "error: falta $CHECKSUMS" >&2
  exit 1
fi

download() {
  local url="$1" dest="$2"
  echo "==> fetch $(basename "$dest")"
  local tmp
  tmp="$(mktemp "${dest}.XXXXXX")"
  curl -fsSL --retry 3 --retry-delay 2 -o "$tmp" "$url"
  chmod +x "$tmp"
  mv -f "$tmp" "$dest"
}

expected_sha() {
  local name="$1"
  grep -E "^[0-9a-f]{64}  ${name}\$" "$CHECKSUMS" | awk '{print $1}'
}

ensure_file() {
  local name="$1" url="$2"
  local dest="$CACHE/$name"
  local want got
  want="$(expected_sha "$name")"
  if [[ -z "$want" ]]; then
    echo "error: $name no está en tool-checksums.sha256" >&2
    exit 1
  fi
  if [[ -f "$dest" ]]; then
    got="$(sha256sum "$dest" | awk '{print $1}')"
    if [[ "$got" == "$want" ]]; then
      echo "==> OK cache ${name}"
      return 0
    fi
    echo "==> checksum antiguo en cache (${got}); re-descargar ${name}"
    rm -f "$dest"
  fi
  download "$url" "$dest"
  got="$(sha256sum "$dest" | awk '{print $1}')"
  if [[ "$got" != "$want" ]]; then
    echo "error: SHA256 de ${name} no coincide (got=${got} want=${want})" >&2
    rm -f "$dest"
    exit 1
  fi
}

ensure_file "AppRun-${ARCH}" \
  "https://github.com/tauri-apps/binary-releases/releases/download/apprun-old/AppRun-${ARCH}"

ensure_file "linuxdeploy-07333c6-${ARCH}.AppImage" \
  "https://github.com/tauri-apps/binary-releases/releases/download/linuxdeploy-07333c6/linuxdeploy-${ARCH}.AppImage"

# Asset versionado (no continuous) → nombre canónico del bundler.
PLUGIN_NAME="linuxdeploy-plugin-appimage.AppImage"
PLUGIN_URL="https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/1-alpha-20250213-1/linuxdeploy-plugin-appimage-${ARCH}.AppImage"
want="$(expected_sha "$PLUGIN_NAME")"
dest="$CACHE/$PLUGIN_NAME"
if [[ -f "$dest" ]] && [[ "$(sha256sum "$dest" | awk '{print $1}')" == "$want" ]]; then
  echo "==> OK cache ${PLUGIN_NAME}"
else
  rm -f "$dest"
  tmp="$(mktemp)"
  download "$PLUGIN_URL" "$tmp"
  mv -f "$tmp" "$dest"
  chmod +x "$dest"
  got="$(sha256sum "$dest" | awk '{print $1}')"
  if [[ "$got" != "$want" ]]; then
    echo "error: SHA256 de ${PLUGIN_NAME} no coincide (got=${got} want=${want})" >&2
    rm -f "$dest"
    exit 1
  fi
fi

echo "==> verificar SHA256 en ${CACHE}"
(
  cd "$CACHE"
  grep -E '^[0-9a-f]{64}  ' "$CHECKSUMS" | sha256sum -c -
)

echo "==> herramientas OK en ${CACHE}"
