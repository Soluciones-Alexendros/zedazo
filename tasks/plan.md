# Plan técnico: remediación de hallazgos P0–P2 (auditoría 2026-11-17)

> Sustituye al plan v0.6.0 (cerrado; release 0.6.1 ya en `main`). Base: `plan/remediacion-p0-p2`.
> Ficha de trazabilidad en `docs/validation/informe-ejecutivo-estado.md` y
> `docs/validation/plan-ssd.md`. Tareas operativas en `tasks/todo.md`.

## Overview

Remediar los hallazgos de la auditoría de estado (semáforo rojo, health 50/100) para
devolver el gate de seguridad y `ci-gate` a verde y cerrar la deuda P1/P2. No añade
funcionalidad de producto; es un cierre de salud del repo.

## Decisiones de arquitectura

- **D1 (Opción A, elegida por el operador)**: mantener **MSRV 1.80** y la imagen
  `deploy/Dockerfile.api` en `rust:1.85-bookworm`. Se contiene a Renovate con pins
  `allowedVersions` en `.github/renovate.json`, no se sube el mínimo de Rust.
- **D2**: el advisory RUSTSEC-2026-0538 (`quinn-udp`) se resuelve con parche de
  versión dentro de la misma serie (`0.5.15 → 0.5.16`), sin tocar `reqwest`/`quinn`
  ni introducir deps nuevas (ADR-0018, ADR-0013).
- **D3**: los avisos RustSec no bloqueantes (`chrono` unmaintained, `rustix` unsound)
  **no** entran como parche en este plan: se documentan con fecha de revisión, igual
  que las excepciones ya vigentes en `.cargo/audit.toml`/`deny.toml`.
- **D4**: el fallo `unexpected alert open` se corrige en el **harness de e2e**
  (determinismo de WebDriver ante `window.confirm/alert` nativos de `menu_es.rs`).
  La sustitución de los diálogos nativos por UI in-app queda **fuera** de este plan
  (cambio de GUI que necesita ancla propia en SPECS/ADR; AGENTS.md §Learned Preferences).

## Límites (boundaries)

- **Always**: verificación real con el comando del repo antes de dar por cerrado;
  commits en español; un cambio por unidad; no secretos.
- **Ask first**: cualquier subida de MSRV, majors (`nom 8`, `toml 1`, `chardetng 1`),
  features de red, o cambio de UI de producto.
- **Never**: `--force`/reescritura de historia sin permiso; merge a `main` (operador,
  P9); tocar `renovate.json`/`release-please`/`scripts/check-*.sh` sin pasar por la
  regla de policy-exception (estas rutas ya existen: ver Task 3).

## Task List

### Fase 1 — P0: desbloquear el gate de seguridad

- [x] **Task 1**: `quinn-udp` 0.5.15 → 0.5.16 (RUSTSEC-2026-0538).

### Checkpoint A (tras Task 1)

- [x] `cargo audit` sin `error: 1 vulnerability found`
- [ ] `Security Audit` y `Security / sca` verdes en CI; `ci-gate` verde en el PR

### Fase 2 — P1: alertas npm + Renovate/MSRV

- [x] **Task 2**: resolver Dependabot `minimatch` (CVE-2026-118764) y `glob` (CVE-2026-118773).
  Corregido respecto al borrador: están en `scripts/desktop/e2e`, no en `apps/web`.
- [x] **Task 3**: pins Renovate Opción A — `encoding_rs <0.8.40`, `uuid <1.27.0`, más
  `lockFileMaintenance` de cargo desactivado (los pins solos no lo contienen).

### Checkpoint B (tras Tasks 2–3)

- [x] `npm audit` del e2e limpio; `renovate-config-validator` OK
- [ ] 0 alertas Dependabot `open` (medium+) tras el merge
- [ ] `make compose-smoke` y check MSRV 1.80 verdes sobre lock refrescado (CI: sin docker local)

### Fase 3 — P1: estabilidad e2e

- [x] **Task 4**: determinismo de `Desktop e2e` ante diálogos nativos.

### Fase 4 — P2: higiene

- [x] **Task 5**: consolidar fixture duplicado `sample-contacts.vcf` / `examples/sample.vcf`.
- [x] **Task 6**: añadir `.env.example`.
- [x] **Task 7**: registrar plan de migración `chrono → jiff` y revisión `rustix` (ADR-0022).

### Checkpoint C (final)

- [ ] `make ci` local verde y `ci-gate` verde en CI
- [ ] `Security Audit`, `Security`, `Web (apps/web)`, `Desktop e2e` verdes

## Riesgos

| Riesgo | Mitigación |
| --- | --- |
| El pin `encoding_rs <0.8.40` deja escapar otro transitivo que rompa MSRV en el próximo refresh | El check MSRV 1.80 + Compose smoke en CI detectan el siguiente ofensor; se repite el mismo patrón de pin con `cargo info <crate>@<ver>` |
| Dependabot sigue reportando `minimatch` abierta por versión anidada fuera del override | `pnpm why minimatch` localiza el origen; si es un override `pnpm.overrides`, ampliar el override |
| El fallo e2e es flaky y no reproducible localmente | Ejecutar el spec ×3 en CI; capturar el punto exacto (confirm de borrado) y aceptar/descartar el diálogo explícitamente |
| `cargo update -p quinn-udp` arrastra `quinn-proto`/`quinn` por rango | Verificado en dry-run: solo toca `quinn-udp` (1 crate); si apareciera otro, revisar el diff de `Cargo.lock` |

## Verificación global (Definition of Done)

- Jobs canónicos `quality`, `test`, `smoke` verdes (`make ci` cubre el conjunto local).
- `Security Audit` + `Security` + `ci-gate` verdes en `main`.
- 0 alertas Dependabot abiertas de severidad ≥ medium; 0 vulnerabilidades RustSec bloqueantes.
- Docs contractuales actualizados solo si cambia contrato (Task 7 toca ROADMAP/DECISIONS → `make docs-validate`).
