#!/usr/bin/env bash
# P-02 / BL-1: e2e AppImage bajo red aislada (unshare -rn).
# strace solo del AppImage (envoltorio en TAURI_APPIMAGE), no del arnés.
# Falla si hay connect IP no-loopback. AF_UNIX se ignora.
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
WRAPPER="$(mktemp -t zedazo-appimage-strace.XXXXXX)"
cleanup() {
  rm -f "$TRACE" "$WRAPPER"
}
trap cleanup EXIT

# Autotest del filtro (loopback vs fuga Azure DNS).
python3 - <<'PY'
import ipaddress
import sys

def is_loopback(s: str) -> bool:
    ip = ipaddress.ip_address(s)
    mapped = getattr(ip, "ipv4_mapped", None)
    if mapped is not None:
        ip = mapped
    return bool(ip.is_loopback)

ok = [
    "127.0.0.1",
    "127.0.0.53",
    "::1",
    "::ffff:127.0.0.1",
]
bad = ["168.63.129.16"]
for s in ok:
    if not is_loopback(s):
        print(f"error: se esperaba loopback: {s}", file=sys.stderr)
        sys.exit(1)
for s in bad:
    if is_loopback(s):
        print(f"error: no debe ser loopback: {s}", file=sys.stderr)
        sys.exit(1)
print("OK filtro loopback (127.0.0.1/53, ::1, ::ffff:127.0.0.1; 168.63.129.16 fuga)")
PY

APPIMAGE="${TAURI_APPIMAGE:-${APPIMAGE:-}}"
if [[ -z "$APPIMAGE" ]]; then
  APPIMAGE="$(find dist/desktop -maxdepth 1 -name 'zedazo-*.AppImage' -type f | sort | head -1 || true)"
fi
if [[ -z "$APPIMAGE" || ! -f "$APPIMAGE" ]]; then
  echo "error: falta AppImage (TAURI_APPIMAGE=...)" >&2
  exit 1
fi
APPIMAGE="$(cd "$(dirname "$APPIMAGE")" && pwd)/$(basename "$APPIMAGE")"
chmod +x "$APPIMAGE"

# Envoltorio: strace solo del AppImage e hijos (no bash/cargo/npm del arnés).
cat >"$WRAPPER" <<EOF
#!/usr/bin/env bash
set -euo pipefail
exec strace -f -e trace=connect -o '$TRACE' -- '$APPIMAGE' "\$@"
EOF
chmod +x "$WRAPPER"

echo "==> e2e sin red (unshare -rn; strace AppImage → $TRACE)"

export TAURI_APPIMAGE="$WRAPPER"
export ZEDAZO_OFFLINE="${ZEDAZO_OFFLINE:-1}"
export APPIMAGE_EXTRACT_AND_RUN="${APPIMAGE_EXTRACT_AND_RUN:-1}"
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"

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
  export TAURI_APPIMAGE='$WRAPPER'
  export ZEDAZO_OFFLINE='$ZEDAZO_OFFLINE'
  export APPIMAGE_EXTRACT_AND_RUN='$APPIMAGE_EXTRACT_AND_RUN'
  export WEBKIT_DISABLE_DMABUF_RENDERER='$WEBKIT_DISABLE_DMABUF_RENDERER'
  exec bash '$ROOT/scripts/desktop/run-e2e.sh'
"
STATUS=$?
set -e

echo "==> filtrar connect IP no-loopback en $TRACE"
set +e
BAD="$(
  python3 - "$TRACE" <<'PY'
import ipaddress
import re
import sys

path = sys.argv[1]
bad = []


def is_loopback(s: str) -> bool:
    ip = ipaddress.ip_address(s)
    mapped = getattr(ip, "ipv4_mapped", None)
    if mapped is not None:
        ip = mapped
    return bool(ip.is_loopback)


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
    addrs = []
    addrs.extend(re.findall(r'inet_addr\("([^"]+)"\)', line))
    addrs.extend(re.findall(r'inet_pton\(AF_INET6,\s*"([^"]+)"', line))
    # Forma compacta que a veces imprime strace en connect.
    addrs.extend(re.findall(r'sin_addr=inet_addr\("([^"]+)"\)', line))
    for s in addrs:
        try:
            if not is_loopback(s):
                bad.append(f"{i}:{line.strip()}")
        except ValueError:
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

echo "OK e2e-no-network (solo loopback / AF_UNIX; COUNT=0)"
