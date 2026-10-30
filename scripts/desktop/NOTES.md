# Scripts CI escritorio (W5A)

Gate local: `bash -n scripts/desktop/*.sh` + YAML parseable. El build AppImage completo (WebKitGTK) vive en GHA `ubuntu-22.04`.

## Jobs (`ci.yml` → `needs` de `ci-gate`)

| Job | Runner | Rol |
|-----|--------|-----|
| `desktop-appimage` | ubuntu-22.04 | AppImage + `ZEDAZO_GIT_SHA` / `ZEDAZO_BUILD_DATE` |
| `desktop-smoke` | ubuntu-22.04 | xvfb + `GET /api/v1/health` |
| `desktop-e2e` | ubuntu-22.04 | tauri-driver + sample.vcf + I7 + API↔CLI |
| `desktop-parity` | ubuntu-22.04 | CLI `zedazo cribar` + I7 (sin WebKit) |
| `desktop-sbom` | ubuntu-22.04 | CycloneDX Cargo + pnpm |
| `desktop-attest` | ubuntu-22.04 | `attest-build-provenance` sobre el AppImage |

AppImage atestado en release: [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml) (`on: release published`).
`release.yml` solo sube artefactos cargo-dist al draft de release-please (sin `gh release create`, sin job appimage).

## Ubuntu 26 (contenedor)

**No hay job.** WebKit/GTK en contenedor sin sesión gráfica fiable; un job `continue-on-error` informativo colgaría de `ci-gate` sin señal. Prueba Ubuntu 26 = manual (operador + guía W5B) sobre el AppImage del draft.

## Acciones

Pins por commit SHA en los jobs desktop (comentario = tag semver).
