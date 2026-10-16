#!/usr/bin/env bash
# Smoke de Compose (no confundir con el job CI wrapper `smoke` = health+check).
# Build → salud → subida sample.vcf → espera completed → descarga → borrado.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
COMPOSE=(docker compose -f "$ROOT/deploy/docker-compose.yml")
# Preferir localhost: tls internal de Caddy a veces rechaza SNI/IP en 127.0.0.1.
BASE="${ZEDAZO_SMOKE_BASE:-https://localhost:8443}"
CURL=(curl -fsS -k --max-time 30)
SAMPLE="$ROOT/examples/sample.vcf"

json_field() {
  local json="$1" path="$2"
  python3 -c 'import json,sys; d=json.load(sys.stdin)
p=sys.argv[1].split(".")
cur=d
for k in p:
  cur=cur[k]
print(cur)' "$path" <<<"$json"
}

cleanup() {
  "${COMPOSE[@]}" down --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "==> compose build + up"
"${COMPOSE[@]}" up --build -d

echo "==> wait caddy healthy + health via proxy"
ok=0
for _ in $(seq 1 90); do
  caddy_h="$("${COMPOSE[@]}" ps --format json 2>/dev/null | python3 -c '
import json,sys
raw=sys.stdin.read().strip()
if not raw: raise SystemExit(1)
# docker compose puede emitir un JSON por línea
items=[]
for line in raw.splitlines():
  try: items.append(json.loads(line))
  except Exception: pass
if not items and raw.startswith("["):
  items=json.loads(raw)
svc=[i for i in items if i.get("Service")=="caddy"]
print((svc[0].get("Health") or svc[0].get("State") or "") if svc else "")
' 2>/dev/null || true)"
  if h="$("${CURL[@]}" "$BASE/api/v1/health" 2>/dev/null)" && echo "$h" | grep -q '"status"'; then
    ok=1
    break
  fi
  sleep 2
done
test "$ok" -eq 1 || { echo "health timeout (caddy=$caddy_h)"; "${COMPOSE[@]}" logs caddy --tail 40 || true; exit 1; }

echo "==> upload sample.vcf"
upload_json="$("${CURL[@]}" -X POST "$BASE/api/v1/uploads" \
  -F "file=@${SAMPLE};type=text/vcard")"
upload_id="$(json_field "$upload_json" "upload_id")"
test -n "$upload_id"

echo "==> create job"
job_json="$("${CURL[@]}" -X POST "$BASE/api/v1/jobs" \
  -H 'content-type: application/json' \
  -d "{\"upload_id\":\"$upload_id\",\"artifacts\":[\"vcf\",\"audit_tsv\"]}")"
job_id="$(json_field "$job_json" "job_id")"
test -n "$job_id"

echo "==> wait completed ($job_id)"
status=""
for _ in $(seq 1 90); do
  payload="$("${CURL[@]}" "$BASE/api/v1/jobs/$job_id")"
  status="$(json_field "$payload" "job.status")"
  case "$status" in
    completed) break ;;
    failed|cancelled|expired) echo "job $status: $payload"; exit 1 ;;
  esac
  sleep 1
done
test "$status" = "completed"

echo "==> download artifact vcf"
"${CURL[@]}" -o /tmp/zedazo-smoke.vcf "$BASE/api/v1/jobs/$job_id/artifacts/vcf"
grep -q 'BEGIN:VCARD' /tmp/zedazo-smoke.vcf

echo "==> SSE probe (first bytes)"
timeout 3 "${CURL[@]}" -N -H 'Accept: text/event-stream' \
  "$BASE/api/v1/jobs/$job_id/events" -o /tmp/zedazo-smoke.sse || true
# Tras completed el stream puede cerrar rápido; no fallar solo por vacío.

echo "==> delete job"
code="$("${CURL[@]}" -o /dev/null -w '%{http_code}' -X DELETE "$BASE/api/v1/jobs/$job_id")"
test "$code" = "204"

echo "OK compose-smoke ($BASE job=$job_id)"
