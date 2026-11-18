---
title: "SPEC-release-artifact-verification"
date: "2026-11-18"
status: "Borrador"
module: "release-artifact-verification"
depends_on: []
traza: "H-03 / INT-1 INT-2; publish.yml ya sube SHA256SUMS + attestation"
---

# Spec: release-artifact-verification

## Objective

Documentar el procedimiento de **verificación de artefactos AppImage** para el usuario final, usando los assets que el publish ya produce.

Hecho verificable en el árbol (no es hueco de pipeline):

- [`.github/workflows/publish.yml`](../../../.github/workflows/publish.yml) sube `*.AppImage`, `SHA256SUMS`, `build-info.txt`, `sbom-*.json`, `SBOM.SHA256SUMS`.
- El job `verify-release` ejecuta `sha256sum -c SHA256SUMS` y `gh attestation verify`.

Hueco real (H-03 corregido): [`docs/desktop.md`](../../desktop.md) explica descarga/ejecución y **no** checksum ni attestation.

Criterios de aceptación:

1. Nueva sección en `docs/desktop.md` (o doc enlazado desde ella) con pasos reproducibles:
   - Descargar AppImage + `SHA256SUMS` del release.
   - `sha256sum -c SHA256SUMS` (o equivalente filtrando la línea del AppImage).
   - `gh attestation verify <AppImage> --repo Soluciones-Alexendros/zedazo`.
2. Enlace desde README o sección «Ver también» de desktop hacia este procedimiento.
3. **Sin** introducir firma minisign/GPG en este módulo (mapa: otra iniciativa + secreto).

## Tech Stack

- Markdown operativo (`docs/desktop.md`, opcional `docs/release-validation.md` como referencia cruzada).
- GitHub CLI (`gh`) para attestation; `sha256sum` / `shasum -a 256`.
- Assets ya generados por el workflow de publish (ADR-0021).

## Commands

Documentar (y verificar manualmente contra un tag real, p. ej. `v0.6.4`):

```bash
TAG=v0.6.4
mkdir -p /tmp/zedazo-verify && cd /tmp/zedazo-verify
gh release download "$TAG" -R Soluciones-Alexendros/zedazo \
  -p '*.AppImage' -p 'SHA256SUMS' -p 'build-info.txt'
sha256sum -c SHA256SUMS
APP=$(find . -maxdepth 1 -name '*.AppImage' -type f | sort | head -1)
gh attestation verify "$APP" --repo Soluciones-Alexendros/zedazo
```

Comprobar que la guía existe tras la implementación:

```bash
rg -n "sha256sum|attestation verify|SHA256SUMS" docs/desktop.md
```

## Project Structure

```text
docs/desktop.md                 → sección «Verificar el AppImage»
.github/workflows/publish.yml   → fuente de verdad del pipeline (no editar en este módulo)
docs/release-validation.md      → referencia cruzada opcional (CI/operador)
```

## Code Style

- Español; comandos en bloques bash copiables.
- No afirmar SLSA L2/L3; describir lo que el usuario puede ejecutar hoy (checksum + attestation GitHub).
- No mezclar firma minisign/GPG en esta sección.

## Testing Strategy

- Smoke manual: seguir la guía contra el último release con AppImage.
- `make docs-validate` si aplica a `docs/desktop.md`.
- No cambiar `publish.yml` ni `release-validation.yml` en este módulo.

## Boundaries

- **Always:** anclar comandos a assets reales del release; enlazar ADR-0021 / publish.
- **Ask first:** añadir firma minisign/GPG; cambiar el workflow de publish; exigir attestation como check de usuario sin `gh`.
- **Never:** afirmar que faltan `SHA256SUMS` en el pipeline; inventar checksums; publicar claves privadas.

## Success Criteria

- [x] `docs/desktop.md` incluye pasos `sha256sum` + `gh attestation verify`.
- [x] Evidencia real: [evidence/appimage-verify.md](./evidence/appimage-verify.md).
- [x] Diff de guía en docs; `publish.yml` sin cambios en este módulo.

## Open Questions

1. ¿Se documenta también la verificación del SBOM (`SBOM.SHA256SUMS`) en la misma sección o en `docs/release-validation.md`?
2. ¿Se menciona Windows/macOS vía cargo-dist (tarballs) o se limita a AppImage (ADR-0021)?
3. ¿Se añade un enlace corto en el body del release (release-please) o solo en docs?
