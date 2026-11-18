---
title: "SPEC-crates-sbom"
date: "2026-11-18"
status: "Borrador"
module: "crates-sbom"
depends_on: []
traza: "INT-4 INT-6; ADR-0013; publish-crates.yml"
---

# Spec: crates-sbom

## Objective

Cuando se publica un tag en **crates.io** (`zedazo-core` + `zedazo`), el **mismo tag** en el release de GitHub debe exponer un **SBOM CycloneDX** (y checksum asociado) reutilizando el patrón de [ADR-0013](../../../DECISIONS.md).

Hechos del árbol:

- Escritorio: `publish.yml` ya sube `sbom-*.json` + `SBOM.SHA256SUMS`.
- Crates: [`.github/workflows/publish-crates.yml`](../../../.github/workflows/publish-crates.yml) hace `cargo publish` y **no** genera/sube SBOM.
- crates.io **no** aloja el fichero SBOM; el artefacto vive en el GitHub Release del tag.

Criterios de aceptación (spec; implementación posterior):

1. Procedimiento documentado: generar CycloneDX del workspace (o de `zedazo-core` + `zedazo`) en el ref del tag.
2. Subir el SBOM (+ `SHA256SUMS` o `.sha256`) al GitHub Release del mismo tag, sin exigir que crates.io lo almacene.
3. Integración preferente: paso en `publish-crates.yml` **o** job hermano disparado tras publish exitoso; sin `cargo publish` automático en Phase 1.
4. Documentar en `docs/` o SECURITY el cómo descargar/verificar el SBOM del tag de crates.
5. Esta fase **no** ejecuta `cargo publish` ni toca el token `CRATES_IO_TOKEN`.

## Tech Stack

- `cargo cyclonedx` (o la herramienta ya usada en el release de escritorio — alinear versión/comando con ADR-0013 / scripts existentes).
- GitHub Releases (`gh release upload`).
- Workflow `workflow_dispatch` existente (`publish-crates.yml`).

## Commands

Referencia (implementación futura; no ejecutar publish aquí):

```bash
# Generar SBOM en el ref del tag (ejemplo; alinear con el script canónico del repo)
git checkout "v0.6.4"
cargo cyclonedx -p zedazo-core -p zedazo --format json
sha256sum sbom*.json > SBOM-crates.SHA256SUMS

# Subir al release existente (operador / workflow)
gh release upload "v0.6.4" sbom*.json SBOM-crates.SHA256SUMS \
  -R Soluciones-Alexendros/zedazo
```

Verificación de la spec documental:

```bash
rg -n "cyclonedx|SBOM" .github/workflows/publish.yml docs/
rg -n "cargo publish" .github/workflows/publish-crates.yml
```

## Project Structure

```text
.github/workflows/publish-crates.yml  → extensión futura
.github/workflows/publish.yml         → patrón SBOM escritorio (referencia)
DECISIONS.md                          → ADR-0013
docs/security/audit-2026/             → esta spec
```

## Code Style

- Nombres de asset claros (`sbom-crates-*.json` o convención ya usada) para no colisionar con SBOM de AppImage.
- Pins SHA si se añaden Actions.
- Español en docs; ids de job en inglés.

## Testing Strategy

- Dry-run: generar SBOM localmente en un tag ya publicado y validar JSON CycloneDX (schema / `jq`).
- Tras implementar el workflow: un `workflow_dispatch` en entorno de prueba o tag ya publicado que solo suba SBOM (sin republicar crates).
- No marcar PASS sin evidencia del comando en sesión.

## Boundaries

- **Always:** SBOM en GitHub Release del tag; alinear con ADR-0013; separar asset de AppImage.
- **Ask first:** `cargo publish`; cambiar `publish-crates.yml`; nueva dep CLI de SBOM si no es la canónica.
- **Never:** afirmar que crates.io hospeda el SBOM; ejecutar publish en Phase 1; secretos en docs.

## Success Criteria

- [ ] Spec aprobada describe comando + destino del asset.
- [ ] Implementación posterior: release del tag muestra SBOM crates además del de escritorio (si ambos aplican).
- [ ] Docs de verificación enlazadas.
- [ ] Phase 1: solo este documento (y el mapa); sin diff de workflow.

## Open Questions

1. ¿Un SBOM por crate o un SBOM de workspace?
2. ¿Paso dentro de `publish-crates.yml` o workflow `sbom-crates.yml` reutilizable sin publish?
3. ¿Nombre canónico del asset para no pisar `sbom-*.json` del AppImage?
