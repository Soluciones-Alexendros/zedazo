---
title: "SPEC-scorecard-publish"
date: "2026-11-18"
status: "Borrador"
module: "scorecard-publish"
depends_on:
  - "vuln-report-channel"
  - "sast-critical-paths"
traza: "ESC-6; OpenSSF Scorecard; MAP.md ola B"
---

# Spec: scorecard-publish

## Objective

Publicar de forma **recurrente** la puntuación OpenSSF Scorecard del repo y documentar una **revisión anual**, sin convertir Scorecard en check requerido de merge.

Depende de las specs [vuln-report-channel](./SPEC-vuln-report-channel.md) y [sast-critical-paths](./SPEC-sast-critical-paths.md) para no fijar una foto pública que esos dos cambios invalidarían al día siguiente. La implementación de Scorecard puede esperar a que esas specs estén **aprobadas** e idealmente aterrizadas.

Criterios de aceptación:

1. Workflow programado (cron) + `workflow_dispatch` que ejecute OpenSSF Scorecard (Action oficial, pin por SHA).
2. Resultados publicados de forma consumible (SARIF a code scanning, badge, o artifact — elegir una y documentarla).
3. **No** añadir Scorecard a `ci-gate` / required checks.
4. Entrada en docs (`docs/security/` o SECURITY) con: frecuencia del cron, enlace a resultados, cadencia de revisión **anual**.
5. Permisos mínimos; sin secretos nuevos salvo los que Scorecard documente como opcionales (ask first).

## Tech Stack

- OpenSSF Scorecard GitHub Action (versión fijada por SHA en la implementación).
- Runners `ubuntu-latest`.
- Opcional: badge en README (ask first — toca README canónico).

## Commands

Tras implementación:

```bash
rg -n "scorecard|openssf" .github/workflows/*.yml docs/security/
# Disparo manual
gh workflow run scorecard.yml -R Soluciones-Alexendros/zedazo
# Revisar último run
gh run list -R Soluciones-Alexendros/zedazo --workflow=scorecard.yml --limit 5
```

Phase 1 (solo docs):

```bash
test -f docs/security/audit-2026/SPEC-scorecard-publish.md
```

## Project Structure

```text
.github/workflows/scorecard.yml   → futuro
docs/security/                    → guía de revisión anual
SECURITY.md                       → enlace opcional
README.md                         → badge solo con confirmación
```

## Code Style

- Pins SHA + comentario de versión.
- Job id estable (`scorecard`); español en docs.
- No duplicar la suite de `security.yml` / gitleaks; Scorecard es señal externa agregada.

## Testing Strategy

- Primer run manual (`workflow_dispatch`) registrado en la sesión de implementación.
- Revisión anual: checklist en docs (fecha, score, deltas vs año anterior, acciones).
- No PASS si el workflow no se ha ejecutado al menos una vez tras aterrizar.

## Boundaries

- **Always:** cron + dispatch; fuera de `ci-gate`; documentar revisión anual; esperar a specs de canal y SAST.
- **Ask first:** badge en README; token PAT si Scorecard lo pide para repos privados/branch protection; fallar PRs por score bajo.
- **Never:** required check; afirmar Scorecard «no verificable» tras publicarlo; secretos en el YAML en claro.

## Success Criteria

- [ ] Workflow Scorecard en el árbol con pin SHA.
- [ ] Docs de cadencia anual y dónde leer el resultado.
- [ ] Ausente de `ci-gate`.
- [ ] Phase 1: solo esta spec bajo `audit-2026/`.

## Open Questions

1. ¿Badge en README en el mismo PR de implementación o doc-only?
2. ¿Cron semanal (como audit) o mensual?
3. ¿Se espera a cerrar H-01 y H-02 (implementación) antes del primer publish público del score?
