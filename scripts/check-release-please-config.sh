#!/usr/bin/env bash
# Valida release-please (P-01): tipo simple, flags, extra-files y versiones alineadas.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CFG="$ROOT/release-please-config.json"
WEB="$ROOT/apps/web/package.json"
TAURI="$ROOT/apps/desktop/tauri.conf.json"
CARGO="$ROOT/Cargo.toml"

fail() {
  echo "❌ check-release-please-config: $*" >&2
  exit 1
}

command -v python3 >/dev/null || fail "python3 requerido"

python3 - "$CFG" <<'PY' || fail "config release-please inválida"
import json, sys
path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    cfg = json.load(f)
pkg = cfg.get("packages", {}).get(".", {})
if pkg.get("release-type") != "simple":
    raise SystemExit(f"release-type={pkg.get('release-type')!r}, esperado 'simple'")
if pkg.get("bump-minor-pre-major") is not True:
    raise SystemExit("falta bump-minor-pre-major: true")
if pkg.get("draft") is not True:
    raise SystemExit("falta draft: true")
if pkg.get("force-tag-creation") is not True:
    raise SystemExit("falta force-tag-creation: true")
if pkg.get("include-component-in-tag") is not False:
    raise SystemExit("include-component-in-tag debe ser false")
extras = pkg.get("extra-files") or []
want = {
    ("toml", "Cargo.toml", "$.workspace.package.version"),
    ("toml", "Cargo.toml", "$.workspace.dependencies['zedazo-core'].version"),
    ("json", "apps/web/package.json", "$.version"),
    ("json", "apps/desktop/tauri.conf.json", "$.version"),
}
got = {(e.get("type"), e.get("path"), e.get("jsonpath")) for e in extras}
missing = want - got
if missing:
    raise SystemExit(f"extra-files incompletos: {sorted(missing)}")
print("✓ release-please-config.json (tipo, flags, extra-files)")
PY

# Versiones alineadas en los tres ficheros (no el manifest; release-please lo gestiona).
CARGO_VER="$(python3 - "$CARGO" <<'PY'
import re, sys
text = open(sys.argv[1], encoding="utf-8").read()
# Solo workspace.package.version (primer bloque [workspace.package])
m = re.search(r"(?ms)^\[workspace\.package\].*?^version\s*=\s*\"([^\"]+)\"", text)
if not m:
    raise SystemExit("no hallé workspace.package.version")
print(m.group(1))
PY
)" || fail "no se pudo leer Cargo.toml version"

WEB_VER="$(python3 -c "import json; print(json.load(open('$WEB'))['version'])")"
TAURI_VER="$(python3 -c "import json; print(json.load(open('$TAURI'))['version'])")"

if [[ "$CARGO_VER" != "$WEB_VER" || "$CARGO_VER" != "$TAURI_VER" ]]; then
  fail "versiones desalineadas: Cargo=$CARGO_VER web=$WEB_VER tauri=$TAURI_VER"
fi

echo "✓ versiones alineadas en $CARGO_VER (Cargo.toml, package.json, tauri.conf.json)"
