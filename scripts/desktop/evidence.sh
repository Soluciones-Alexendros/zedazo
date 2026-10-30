#!/usr/bin/env bash
# Fachada: expediente local de evidencia (sin tokens ni publicación).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

SHA_BASE="${ZEDAZO_SHA_BASE:-$(git rev-parse HEAD 2>/dev/null || echo unknown)}"
OUT_DIR="${ZEDAZO_EVIDENCE_DIR:-dist/desktop/evidence}"
mkdir -p "$OUT_DIR"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
EVIDENCE="${OUT_DIR}/evidence-${STAMP}.txt"

{
  echo "zedazo desktop evidence"
  echo "generated_utc=${STAMP}"
  echo "git_sha=${SHA_BASE}"
  echo "uname=$(uname -a)"
  command -v rustc >/dev/null && echo "rustc=$(rustc --version)"
  command -v cargo >/dev/null && echo "cargo=$(cargo --version)"
  if [[ -f dist/desktop/build-info.txt ]]; then
    echo "--- build-info.txt ---"
    cat dist/desktop/build-info.txt
  fi
  if [[ -f dist/desktop/SHA256SUMS ]]; then
    echo "--- SHA256SUMS ---"
    cat dist/desktop/SHA256SUMS
  fi
  APPIMAGE="$(find dist/desktop -maxdepth 1 -name 'zedazo-*.AppImage' -type f 2>/dev/null | sort | head -1 || true)"
  if [[ -n "$APPIMAGE" ]]; then
    echo "appimage=$(basename "$APPIMAGE")"
    echo "appimage_bytes=$(wc -c <"$APPIMAGE")"
    echo "appimage_sha256=$(sha256sum "$APPIMAGE" | awk '{print $1}')"
  else
    echo "appimage=absent"
  fi
  echo "note=sin tokens de sesión; Ubuntu 26 = aceptación del operador"
} >"$EVIDENCE"

echo "result=passed evidence=${EVIDENCE}"
echo "OK desktop-evidence → ${EVIDENCE}"
