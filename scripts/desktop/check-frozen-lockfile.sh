#!/usr/bin/env bash
# Test negativo: lockfile desincronizado → pnpm install --frozen-lockfile debe fallar.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WEB="$ROOT/apps/web"
LOCK="$WEB/pnpm-lock.yaml"
if [[ ! -f "$LOCK" ]]; then
  echo "error: falta pnpm-lock.yaml" >&2
  exit 1
fi

TMP="$(mktemp)"
cp -a "$LOCK" "$TMP"
# Mutación mínima que invalida el hash del lock sin tocar package.json real.
# Restaurar siempre.
restore() { cp -a "$TMP" "$LOCK"; rm -f "$TMP"; }
trap restore EXIT

# Alterar un byte en una zona de hash (append ruido al final rompe integridad).
echo "# zedazo-frozen-lockfile-negative-test" >>"$LOCK"

set +e
(
  cd "$WEB"
  CI=true pnpm install --frozen-lockfile --ignore-scripts >/tmp/zedazo-frozen-neg.log 2>&1
)
STATUS=$?
set -e

if [[ "$STATUS" -eq 0 ]]; then
  echo "error: frozen-lockfile aceptó lockfile desincronizado" >&2
  tail -20 /tmp/zedazo-frozen-neg.log >&2 || true
  exit 1
fi

echo "OK check-frozen-lockfile (install frozen falló como se esperaba, exit=${STATUS})"
