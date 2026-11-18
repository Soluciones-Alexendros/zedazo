# Evidencia: verificación de artefactos — release v0.6.4

**Fecha:** 2026-11-18  
**Repositorio:** Soluciones-Alexendros/zedazo  
**Tag:** `v0.6.4`  
**URL:** https://github.com/Soluciones-Alexendros/zedazo/releases/tag/v0.6.4  
**Método:** `gh release view v0.6.4 -R Soluciones-Alexendros/zedazo` (solo listado de assets; sin descarga ni ejecución de binarios).

## Assets publicados

| # | Nombre |
| --- | --- |
| 1 | `build-info.txt` |
| 2 | `sbom-cargo-cli.json` |
| 3 | `sbom-cargo-desktop.json` |
| 4 | `sbom-pnpm-web.json` |
| 5 | `sbom.json` |
| 6 | `sbom.json.sha256` |
| 7 | `SBOM.SHA256SUMS` |
| 8 | `SHA256SUMS` |
| 9 | `zedazo-a62470dc25c065409c5fa60e83ad293f658ed9c8-x86_64.AppImage` |
| 10 | `zedazo-x86_64-apple-darwin.tar.xz` |
| 11 | `zedazo-x86_64-apple-darwin.tar.xz.sha256` |

**Total:** 11 assets.

## Criterios obligatorios

| Criterio | Resultado | Notas |
| --- | --- | --- |
| Al menos un `*.AppImage` | **PASS** | `zedazo-a62470dc25c065409c5fa60e83ad293f658ed9c8-x86_64.AppImage` |
| `SHA256SUMS` | **PASS** | Presente en la release |
| `build-info.txt` | **PASS** | Presente en la release |

## Resultado global

**PASS** — Los tres artefactos exigidos para esta comprobación están incluidos en la release `v0.6.4`.