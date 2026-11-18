# Evidencia: verificación AppImage v0.6.4

**Fecha:** 2026-11-18  
**Tag:** `v0.6.4`  
**Método:** `gh release download` + `sha256sum -c SHA256SUMS` + `gh attestation verify` (artefacto temporal fuera del git; borrado tras la prueba).

## Resultados

| Paso | Resultado |
| --- | --- |
| Descarga `*.AppImage`, `SHA256SUMS`, `build-info.txt` | OK |
| `sha256sum -c SHA256SUMS` | **PASS** — `zedazo-a62470dc25c065409c5fa60e83ad293f658ed9c8-x86_64.AppImage: La suma coincide` |
| `gh attestation verify … --repo Soluciones-Alexendros/zedazo` | **PASS** — exit 0; bundle Sigstore / SLSA provenance del workflow `publish.yml` en `refs/tags/v0.6.4` |

## Resultado global

**PASS** — integridad (checksum) y procedencia (attestation) del AppImage de `v0.6.4`.
