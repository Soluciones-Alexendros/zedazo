#!/usr/bin/env bash
# SBOM CycloneDX: Cargo (cli + desktop) + pnpm (apps/web) → dist/desktop/sbom-*.json
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

OUT="${ROOT}/dist/desktop"
mkdir -p "$OUT"

if ! command -v cargo-cyclonedx >/dev/null 2>&1; then
  cargo install cargo-cyclonedx --locked
fi

echo "==> SBOM Cargo zedazo-cli"
cargo cyclonedx --manifest-path crates/zedazo-cli/Cargo.toml --format json --override-filename sbom-cli
if [[ -f sbom-cli.json ]]; then
  mv -f sbom-cli.json "$OUT/sbom-cargo-cli.json"
elif [[ -f crates/zedazo-cli/sbom-cli.json ]]; then
  mv -f crates/zedazo-cli/sbom-cli.json "$OUT/sbom-cargo-cli.json"
else
  echo "error: no se generó sbom-cli.json" >&2
  exit 1
fi

echo "==> SBOM Cargo zedazo-desktop"
cargo cyclonedx --manifest-path apps/desktop/Cargo.toml --format json --override-filename sbom-desktop
if [[ -f sbom-desktop.json ]]; then
  mv -f sbom-desktop.json "$OUT/sbom-cargo-desktop.json"
elif [[ -f apps/desktop/sbom-desktop.json ]]; then
  mv -f apps/desktop/sbom-desktop.json "$OUT/sbom-cargo-desktop.json"
else
  echo "error: no se generó sbom-desktop.json" >&2
  exit 1
fi

echo "==> SBOM pnpm apps/web"
(
  cd apps/web
  if [[ ! -d node_modules ]]; then
    pnpm install --frozen-lockfile
  fi
  # pnpm dlx: el primer posicional es el paquete; --package como flag rompe (404 --package).
  pnpm dlx @cyclonedx/cyclonedx-npm@3.1.0 \
    --output-file "${OUT}/sbom-pnpm-web.json" \
    --output-format JSON \
    --ignore-npm-errors
)

test -f "$OUT/sbom-cargo-cli.json"
test -f "$OUT/sbom-cargo-desktop.json"
test -f "$OUT/sbom-pnpm-web.json"

(
  cd "$OUT"
  sha256sum sbom-cargo-cli.json sbom-cargo-desktop.json sbom-pnpm-web.json >SBOM.SHA256SUMS
)

echo "==> SBOM en ${OUT}"
ls -la "$OUT"/sbom-*.json "$OUT"/SBOM.SHA256SUMS
cat "$OUT/SBOM.SHA256SUMS"
