---
title: "SPEC-packaging-matrix"
date: "2026-11-18"
status: "Borrador"
module: "packaging-matrix"
depends_on:
  - "release-artifact-verification"
traza: "ESC-2; ADR-0012 ADR-0021 dist-workspace.toml; MAP.md ola B"
---

# Spec: packaging-matrix

## Objective

Documentar la **brecha de empaquetado multi-arch / multi-distro** frente a las decisiones ya aceptadas, y dejar un **borrador de ADR** — sin añadir runners, `.deb`, `.rpm` ni matriz CI en esta fase.

Depende de [SPEC-release-artifact-verification.md](./SPEC-release-artifact-verification.md): cualquier artefacto futuro debe reutilizar el mismo modelo de checksum + attestation.

## Estado actual (evidencia)

| Superficie | Decisión / hecho | Limitación |
|---|---|---|
| AppImage Linux x86_64 | ADR-0021; `publish.yml`; prueba Ubuntu 26 | Sin aarch64 AppImage |
| CLI multiplataforma | ADR-0012 cargo-dist | [`dist-workspace.toml`](../../../dist-workspace.toml): solo `x86_64-apple-darwin` activo; aarch64 cancelado por runner; Linux/Windows rotos en cargo-dist 0.32 (comentario en dist-workspace) |
| Docker Compose | deploy documentado | No es matriz multi-arch de producto |
| `.deb` / `.rpm` | No existen | Fuera de ADR actuales |

Criterios de aceptación:

1. Nota de brecha en `docs/security/audit-2026/` o `docs/` (p. ej. `packaging-gap.md`) citando ADR-0012, ADR-0021 y `dist-workspace.toml`.
2. Borrador de ADR (texto en docs o sección lista para pegar en `DECISIONS.md`) que proponga **una** de:
   - (A) Mantener status quo hasta validación real v1.0 (ADR-0012).
   - (B) Ampliar targets cargo-dist cuando haya runners.
   - (C) Añadir formatos distro (deb/rpm) — solo con justificación y confirmación.
3. El borrador exige: mismos `SHA256SUMS` + attestation que el módulo de verificación de artefactos.
4. **No** se mergea el ADR ni se cambian runners sin aceptación humana (AGENTS §4 / DECISIONS).

## Tech Stack

- Documentación + plantilla ADR del repo (`DECISIONS.md`).
- Referencias: cargo-dist, AppImage, flota `ubuntu-latest` (sin self-hosted).

## Commands

```bash
rg -n "targets|aarch64|AppImage" dist-workspace.toml DECISIONS.md docs/desktop.md
rg -n "ADR-0012|ADR-0021" DECISIONS.md
```

No hay build multi-arch en este módulo.

## Project Structure

```text
dist-workspace.toml
.github/workflows/publish.yml
DECISIONS.md                 → ADR-0012, ADR-0021; futuro ADR borrador
docs/security/audit-2026/    → gap + borrador
docs/desktop.md              → verificación (dependencia)
```

## Code Style

- ADR en el formato de `DECISIONS.md` (estado, fecha, contexto, decisión, consecuencias).
- Español; no reescribir ADRs aceptados — el borrador es **sustitución propuesta**, no edición silenciosa.

## Testing Strategy

- Revisión humana del borrador frente a ROADMAP v1.0 (cross-compile / cargo-dist).
- Sin jobs nuevos; sin `make ci` adicional obligatorio para Phase 1.

## Boundaries

- **Always:** citar ADRs vigentes; enlazar verificación de artefactos; parar tras el borrador.
- **Ask first:** aceptar el ADR; añadir runners; `.deb`/`.rpm`; reactivar targets aarch64.
- **Never:** editar `dist-workspace.toml` o flota CI en este módulo; afirmar multi-arch «conseguido».

## Success Criteria

- [ ] Nota de brecha escrita con tabla de estado actual.
- [ ] Borrador ADR con opciones A/B/C y requisito de checksum+attestation.
- [ ] Sin cambios de runners ni formatos de paquete.
- [ ] Phase 1: spec (+ opcionalmente el texto del borrador en el mismo dir al implementar).

## Open Questions

1. ¿El operador prefiere A (status quo hasta v1.0) como default del borrador?
2. ¿AppImage aarch64 está en el horizonte o queda explícitamente fuera?
3. ¿Los tarballs cargo-dist deben entrar en la misma guía de verificación que el AppImage?
