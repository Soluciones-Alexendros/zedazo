---
title: "SPEC-asvs-matrix"
date: "2026-11-18"
status: "Borrador"
module: "asvs-matrix"
depends_on:
  - "vuln-report-channel"
  - "release-artifact-verification"
  - "sast-critical-paths"
traza: "SEC-1; objetivo ASVS L1; MAP.md ola B"
---

# Spec: asvs-matrix

## Objective

Declarar el **nivel ASVS objetivo = L1** y mantener una matriz de cumplimiento en `docs/security/` que solo marque un control como **cubierto** si hay evidencia en el árbol; el resto queda **no evaluado** (nunca «cumplido» por inferencia del informe web).

Dependencias de spec (deben existir como documentos; la implementación de esas capacidades puede seguir abierta):

- [SPEC-vuln-report-channel.md](./SPEC-vuln-report-channel.md)
- [SPEC-release-artifact-verification.md](./SPEC-release-artifact-verification.md)
- [SPEC-sast-critical-paths.md](./SPEC-sast-critical-paths.md)

Criterios de aceptación:

1. Documento `docs/security/asvs-l1-matrix.md` (nombre canónico propuesto) con:
   - Declaración explícita: objetivo **OWASP ASVS L1**.
   - Columnas: ID ASVS (capítulo/control), estado (`cubierto` | `parcial` | `no evaluado` | `no aplica`), evidencia (ruta/commit/job), notas.
2. Controles relacionados con canal de vulns, verificación de artefactos y SAST se listan; su estado refleja el **árbol en el momento de la revisión**, no el deseo del informe.
3. Reutilizar evidencia ya escrita en [`docs/security/owasp-review-0.6.0.md`](../owasp-review-0.6.0.md) y `SECURITY.md` donde aplique.
4. No declarar L2 como objetivo en esta oleada.

## Tech Stack

- Markdown en `docs/security/`.
- Referencia: OWASP ASVS (versión a citar en la matriz; p. ej. 4.0.3 o la vigente al implementar — fijar en el PR).
- Sin código de producto.

## Commands

```bash
# La declaración existe
rg -n "ASVS L1|objetivo" docs/security/asvs-l1-matrix.md

# No se marca cubierto sin evidencia (revisión humana); grep de estados:
rg -n "cubierto|parcial|no evaluado|no aplica" docs/security/asvs-l1-matrix.md

make docs-validate
```

## Project Structure

```text
docs/security/asvs-l1-matrix.md          → futuro artefacto
docs/security/owasp-review-0.6.0.md      → evidencia S/V existente
docs/security/audit-2026/SPEC-*.md       → specs de controles relacionados
SECURITY.md                              → canal, auth, CardDAV
```

## Code Style

- Español en prosa; IDs ASVS en la forma canónica del estándar (`V1.1.1`, etc.).
- Estados cerrados del enum de la spec; no inventar «casi L2».
- Enlaces relativos a evidencia del repo.

## Testing Strategy

- Revisión por muestreo: 5 controles marcados `cubierto` deben tener ruta de archivo o job citables.
- `docs-validate` verde.
- Actualización de la matriz = cambio documental versionado (no CI score).

## Boundaries

- **Always:** objetivo L1 declarado; evidencia o `no evaluado`; depender de las tres specs A citadas.
- **Ask first:** subir objetivo a L2; automatizar la matriz en CI; omitir capítulos enteros sin marcarlos `no aplica`.
- **Never:** marcar `cubierto` solo porque el informe externo lo asumió; afirmar certificación ASVS formal.

## Success Criteria

- [ ] Archivo de matriz con declaración L1.
- [ ] Al menos los capítulos tocados por canal / artefactos / SAST / auth token (ADR-0016) aparecen con estado.
- [ ] Ningún control `cubierto` sin columna evidencia rellena.
- [ ] Phase 1: esta spec; la matriz completa es implementación posterior.

## Open Questions

1. Resuelta: ASVS **4.0.3**.
2. Resuelta: subset con capítulos no recorridos como `no evaluado` (ver matriz).
3. Resuelta: revisión anual alineada a Scorecard ([`scorecard-review.md`](../scorecard-review.md)).
