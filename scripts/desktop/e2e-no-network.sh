#!/usr/bin/env bash
# P-02: e2e AppImage bajo red aislada (unshare -rn) + strace de connect.
# Falla si hay connect IP distinto de 127.0.0.1 / ::1. AF_UNIX se ignora.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if ! command -v unshare >/dev/null 2>&1; then
  echo "error: falta unshare (util-linux)" >&2
  exit 1
fi
if ! command -v strace >/dev/null 2>&1; then
  echo "error: falta strace" >&2
  exit 1
fi

TRACE="$(mktemp -t zedazo-strace.XXXXXX)"
cleanup() { rm -f "$TRACE"; }
trap cleanup EXIT

echo "==> e2e sin red (unshare -rn + strace connect) → $TRACE"

# --map-root-user: poder levantar lo en el netns.
# -rn: montaje + red nuevos (sin interfaces externas).
set +e
unshare --map-root-user -rn bash -c "
  set -euo pipefail
  if command -v ip >/dev/null 2>&1; then
    ip link set lo up || true
  else
    ifconfig lo up 2>/dev/null || true
  fi
  exec strace -f -e trace=connect -o '$TRACE' \
    bash '$ROOT/scripts/desktop/run-e2e.sh'
"
STATUS=$?
set -e

echo "==> filtrar connect IP no-loopback en $TRACE"
set +e
BAD="$(
  python3 - "$TRACE" <<'PY'
import re, sys
path = sys.argv[1]
bad = []
try:
    text = open(path, errors="replace").read()
except FileNotFoundError:
    print("error: no hay traza strace", file=sys.stderr)
    sys.exit(2)
for i, line in enumerate(text.splitlines(), 1):
    if "connect(" not in line:
        continue
    if "AF_UNIX" in line or "sun_path=" in line:
        continue
    for m in re.finditer(r'inet_addr\("([^"]+)"\)', line):
        ip = m.group(1)
        if ip not in ("127.0.0.1", "0.0.0.0"):
            bad.append(f"{i}:{line.strip()}")
    for m in re.finditer(r'inet_pton\(AF_INET6,\s*"([^"]+)"', line):
        ip = m.group(1)
        if ip not in ("::1", "::"):
            bad.append(f"{i}:{line.strip()}")
for b in bad[:40]:
    print(b)
print(f"COUNT={len(bad)}")
sys.exit(1 if bad else 0)
PY
)"
PY_STATUS=$?
set -e

if [[ "$PY_STATUS" -ne 0 ]]; then
  echo "error: connect IP no-loopback detectado:" >&2
  echo "$BAD" >&2
  echo "--- strace (últimas 80 líneas) ---" >&2
  tail -80 "$TRACE" >&2 || true
  exit 1
fi

if [[ "$STATUS" -ne 0 ]]; then
  echo "error: e2e falló con exit=$STATUS (sin connects externos)" >&2
  exit "$STATUS"
fi

echo "OK e2e-no-network (solo loopback / AF_UNIX)"
