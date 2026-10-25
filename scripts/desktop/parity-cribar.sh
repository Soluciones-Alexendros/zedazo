#!/usr/bin/env bash
# Paridad `zedazo cribar` + I7.
# - Siempre: CLI sobre sample.vcf (origen intacto) → dist/desktop/parity-*.
# - Si ZEDAZO_ADDR + ZEDAZO_TOKEN: compara sha256 del VCF CLI con artefacto API.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

SAMPLE="${ZEDAZO_SAMPLE_VCF:-$ROOT/examples/sample.vcf}"
if [[ ! -f "$SAMPLE" ]]; then
  echo "error: no existe $SAMPLE" >&2
  exit 1
fi

WORK="$(mktemp -d)"
cleanup() { rm -rf "$WORK"; }
trap cleanup EXIT

cp -f "$SAMPLE" "$WORK/sample.vcf"
SHA_BEFORE="$(sha256sum "$WORK/sample.vcf" | awk '{print $1}')"

echo "==> cargo build -p zedazo"
cargo build -p zedazo --release
BIN="${CARGO_TARGET_DIR:-$ROOT/target}/release/zedazo"
if [[ ! -x "$BIN" ]]; then
  BIN="$ROOT/target/release/zedazo"
fi

echo "==> zedazo cribar (CLI + I7)"
"$BIN" cribar "$WORK/sample.vcf" -o "$WORK/limpio.vcf" -a "$WORK/audit.tsv"

SHA_AFTER="$(sha256sum "$WORK/sample.vcf" | awk '{print $1}')"
if [[ "$SHA_BEFORE" != "$SHA_AFTER" ]]; then
  echo "error: I7 violada — sample.vcf mutó durante cribar" >&2
  exit 1
fi

test -s "$WORK/limpio.vcf"
test -s "$WORK/audit.tsv"
CLI_SHA="$(sha256sum "$WORK/limpio.vcf" | awk '{print $1}')"

OUT_DIR="${ROOT}/dist/desktop"
mkdir -p "$OUT_DIR"
cp -f "$WORK/limpio.vcf" "$OUT_DIR/parity-limpio.vcf"
cp -f "$WORK/audit.tsv" "$OUT_DIR/parity-audit.tsv"
{
  echo "sample_sha256=${SHA_BEFORE}"
  echo "limpio_sha256=${CLI_SHA}"
  echo "audit_sha256=$(sha256sum "$WORK/audit.tsv" | awk '{print $1}')"
} | tee "$OUT_DIR/parity-cribar.txt"

if [[ -n "${ZEDAZO_ADDR:-}" && -n "${ZEDAZO_TOKEN:-}" ]]; then
  echo "==> paridad API↔CLI (${ZEDAZO_ADDR})"
  BASE="http://${ZEDAZO_ADDR}"
  HDR=(-H "Authorization: Bearer ${ZEDAZO_TOKEN}" -H "Host: ${ZEDAZO_ADDR}")

  UP_JSON="$(curl -sS "${HDR[@]}" -F "file=@${WORK}/sample.vcf;type=text/vcard" \
    "${BASE}/api/v1/uploads")"
  UPLOAD_ID="$(echo "$UP_JSON" | jq -r '.upload_id // empty')"
  if [[ -z "$UPLOAD_ID" ]]; then
    echo "error: upload sin upload_id: $UP_JSON" >&2
    exit 1
  fi

  JOB_JSON="$(curl -sS "${HDR[@]}" -H "Content-Type: application/json" \
    -d "{\"upload_id\":\"${UPLOAD_ID}\"}" \
    "${BASE}/api/v1/jobs")"
  JOB_ID="$(echo "$JOB_JSON" | jq -r '.job_id // empty')"
  if [[ -z "$JOB_ID" ]]; then
    echo "error: job sin job_id: $JOB_JSON" >&2
    exit 1
  fi

  STATUS=""
  for _ in $(seq 1 90); do
    STATUS="$(curl -sS "${HDR[@]}" "${BASE}/api/v1/jobs/${JOB_ID}" | jq -r '.status // empty')"
    echo "job status: ${STATUS}"
    case "$STATUS" in
      completed|failed) break ;;
    esac
    sleep 1
  done
  if [[ "$STATUS" != "completed" ]]; then
    echo "error: job no completed (${STATUS})" >&2
    exit 1
  fi

  curl -sS "${HDR[@]}" "${BASE}/api/v1/jobs/${JOB_ID}/artifacts/vcf" -o "$WORK/api.vcf"
  test -s "$WORK/api.vcf"
  API_SHA="$(sha256sum "$WORK/api.vcf" | awk '{print $1}')"

  SHA_AFTER_API="$(sha256sum "$WORK/sample.vcf" | awk '{print $1}')"
  if [[ "$SHA_BEFORE" != "$SHA_AFTER_API" ]]; then
    echo "error: I7 API — origen mutó" >&2
    exit 1
  fi

  echo "CLI sha256=${CLI_SHA}"
  echo "API sha256=${API_SHA}"
  if [[ "$CLI_SHA" != "$API_SHA" ]]; then
    echo "error: paridad cribar fallida (VCF distinto)" >&2
    exit 1
  fi
  echo "api_sha256=${API_SHA}" >>"$OUT_DIR/parity-cribar.txt"
  echo "OK paridad API↔CLI"
fi

echo "OK parity cribar + I7"
