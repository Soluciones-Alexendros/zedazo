---
title: "SPEC-sast-critical-paths"
date: "2026-11-18"
status: "Borrador"
module: "sast-critical-paths"
depends_on: []
traza: "H-02 / SEC-2 SEC-6; AGENTS §4 confirmación CI"
---

# Spec: sast-critical-paths

## Objective

Añadir **SAST dedicado** sobre las rutas críticas de red/adaptadores: `crates/zedazo-api` y `crates/zedazo-carddav`, sin sustituir `clippy`, `gitleaks`, `cargo audit`/`deny`, `zizmor` ni `hadolint`.

Supuesto del mapa (aprobado): **CodeQL** vía `github/codeql-action`, **no Semgrep**. Sin dependencia Cargo nueva.

Criterios de aceptación:

1. Workflow o job CodeQL que analiza al menos los paths de `zedazo-api` y `zedazo-carddav` (o el workspace Rust con foco documentado en esas crates).
2. Disparo en PR/push a `main` (o equivalente canónico); resultados visibles en Security / Code Scanning.
3. El job **no** entra en el `needs` de `ci-gate` hasta baseline limpio **y** confirmación explícita del operador.
4. Documentar en `SECURITY.md` o `docs/security/` que SAST = CodeQL y qué crates cubre.
5. Actions fijadas por SHA (estilo del resto del CI).

## Tech Stack

- GitHub CodeQL (`github/codeql-action` init / analyze), lenguaje Rust si soportado en la flota; si CodeQL Rust no está listo en el runner, la implementación documenta el fallback (p. ej. queries limitadas o job informativo) **antes** de merge — no se inventa Semgrep aquí.
- Runners `ubuntu-latest` (ADR-0020 / flota existente).
- Sin crates nuevos en el workspace.

## Commands

Tras implementación (verificación):

```bash
# El job existe y no está en ci-gate hasta confirmación
rg -n "codeql|CodeQL" .github/workflows/*.yml
rg -n "ci-gate|needs:" .github/workflows/ci.yml | head -80

# Consultar alertas CodeQL (requiere permisos)
gh api repos/Soluciones-Alexendros/zedazo/code-scanning/alerts --jq 'length'
```

Baseline local (cuando el job exista): abrir el run de Actions del PR y confirmar `analyze` verde o lista de findings triageados.

## Project Structure

```text
.github/workflows/ci.yml          → posible job nuevo (o workflow hermano)
.github/workflows/codeql.yml      → alternativa: workflow dedicado
crates/zedazo-api/                → superficie HTTP (Axum)
crates/zedazo-carddav/            → cliente CardDAV
SECURITY.md / docs/security/      → mención del SAST
```

## Code Style

- Pins SHA + comentario de versión, igual que el resto del CI.
- Permisos mínimos del job (`security-events: write` solo si hace falta para upload SARIF).
- Español en comentarios de workflow y docs; ids de job en inglés kebab/`snake` coherente con el repo.

## Testing Strategy

- Primera oleada: job no bloqueante (informativo) hasta triage del baseline.
- Segunda oleada (ask first): añadir a `ci-gate` solo si 0 findings abiertos de severidad acordada o ignores documentados.
- No sustituye tests de dominio ni parity O10.

## Boundaries

- **Always:** fijar Actions por SHA; documentar crates en alcance; respetar `ci-gate` actual hasta confirmación.
- **Ask first:** promover CodeQL a required / `ci-gate`; cambiar a Semgrep; ampliar a `apps/web` o `apps/desktop`; queries custom.
- **Never:** afirmar cobertura SAST sin job en el árbol; meter payloads de exploit; debilitar gitleaks/deny para «compensar».

## Success Criteria

- [ ] Existe workflow/job CodeQL referenciando api + carddav (o workspace con doc de foco).
- [ ] Documentación corta en SECURITY o docs/security.
- [ ] `ci-gate` **sin** CodeQL hasta sí explícito del operador.
- [ ] Pins SHA en las Actions nuevas.

## Open Questions

1. ¿Workflow dedicado (`codeql.yml`) o job dentro de `ci.yml`?
2. ¿Severidad mínima que fallará el job cuando pase a gate (error vs warning)?
3. ¿CodeQL Rust está suficientemente maduro en esta flota, o se acepta un periodo solo-informativo documentado?
