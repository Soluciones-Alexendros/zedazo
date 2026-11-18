---
title: "Implementation plan — auditoría de seguridad 2026"
date: "2026-11-18"
status: "Borrador"
phase: "2"
traza: "docs/security/audit-2026/MAP.md"
---

# Implementation Plan: auditoría de seguridad 2026

## Overview

Aterrizar las nueve specs de [`MAP.md`](./MAP.md) en cambios pequeños, en el orden de abajo. Cada tarea deja el repo compilable y `ci-gate` intacto. Esta fase no implementa: el humano revisa este plan antes de Phase 3 (lista de tareas).

Este archivo es el plan de **esta** iniciativa. [`tasks/plan.md`](../../../tasks/plan.md) y [`tasks/todo.md`](../../../tasks/todo.md) son la remediación P0–P2 ya cerrada (8/8, 2026-11-17). No se sobrescriben. Si Phase 3 se aprueba, la lista vive en `docs/security/audit-2026/TODO.md`.

## Architecture Decisions

- **D1.** Canal verificable = URL de Private Vulnerability Reporting (`https://github.com/Soluciones-Alexendros/zedazo/security/advisories/new`) más la tabla de versiones en **0.6.x**. No se inventa correo ni clave PGP. Activar el ajuste en GitHub queda en checklist del operador, fuera del diff.
- **D2.** Verificación de artefactos = sección nueva en [`docs/desktop.md`](../../desktop.md) con `sha256sum -c` y `gh attestation verify`, copiando el contrato de [`.github/workflows/publish.yml`](../../../.github/workflows/publish.yml) (`verify-release`). Sin minisign/GPG y sin editar el workflow.
- **D3.** SAST = workflow nuevo `.github/workflows/codeql.yml` (`github/codeql-action`, lenguaje `rust`, pin SHA en el momento de implementar). Se dispara en `pull_request`, `push` a `main` y `workflow_dispatch`, con filtro de paths `crates/zedazo-api/**`, `crates/zedazo-carddav/**` y ficheros de workspace que cambian el grafo (`Cargo.toml`, `Cargo.lock`). El análisis es el del workspace Rust: CodeQL no aísla dos crates. **No** se añade a `needs` de `ci-gate` en [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml). Si el action rechaza `rust`, se para y se informa; no se cambia a Semgrep en la misma tarea.
- **D4.** Revisión RustSec = volver a ejecutar `make deny` y `make audit`. Si `RUSTSEC-2024-0429` y `RUSTSEC-2024-0370` siguen sin parche publicable, se renueva la fecha en [`deny.toml`](../../../deny.toml), [`.cargo/audit.toml`](../../../.cargo/audit.toml) y el `reason` de [`osv-scanner.toml`](../../../osv-scanner.toml), y se actualiza S-18 en [`docs/security/owasp-review-0.6.0.md`](../owasp-review-0.6.0.md). El ignore no se quita en ese gesto. El issue de Tauri/gtk-rs se redacta en este directorio; no se crea en GitHub sin un sí.
- **D5.** SBOM de crates = workflow hermano `.github/workflows/sbom-crates.yml` (`workflow_dispatch`, input `ref`). Reutiliza el comando ya canónico en [`.github/workflows/release.yml`](../../../.github/workflows/release.yml) (job `sbom`: `cargo cyclonedx --manifest-path crates/zedazo-cli/Cargo.toml --format json`). Asset `sbom-crates.json` + `sbom-crates.json.sha256` en el GitHub Release de ese tag. Sin `cargo publish` y sin `CRATES_IO_TOKEN`. No se edita el camino de publish de [`publish-crates.yml`](../../../.github/workflows/publish-crates.yml).
- **D6.** Benchmarks = `criterion` como dev-dependency de `zedazo-core`, benches nuevos que llaman a `parse_vcards` ([`parser.rs`](../../../crates/zedazo-core/src/infrastructure/parser.rs)) y a `deduplicate_with_ctrl` ([`identity.rs`](../../../crates/zedazo-core/src/domain/identity.rs)), con corpus sintético del estilo de [`tests/dedup_perf.rs`](../../../crates/zedazo-core/tests/dedup_perf.rs). El test ignorado y el job `release-build` de `ci.yml` se quedan. Los benches no entran en `ci-gate`. **Aprobar este plan incluye el sí de AGENTS §4 para añadir `criterion`.** Si ese sí no vale, se saca la Task 6 antes de Phase 3.
- **D7.** ASVS = documento `docs/security/asvs-l1-matrix.md`, objetivo **L1**, estados `cubierto | parcial | no evaluado | no aplica`. La versión del estándar se cita al escribir la matriz (una sola numeración). Un control `cubierto` lleva ruta o job.
- **D8.** Empaquetado = nota de brecha + borrador en `docs/security/audit-2026/ADR-DRAFT-packaging.md`. Opción por defecto **A**: status quo hasta la validación real de ADR-0012 en v1.0. No se edita [`DECISIONS.md`](../../../DECISIONS.md) ni [`dist-workspace.toml`](../../../dist-workspace.toml). El borrador exige el mismo checksum + attestation que D2.
- **D9.** Scorecard = `.github/workflows/scorecard.yml`, cron **mensual** + `workflow_dispatch`, resultados SARIF (`security-events: write`), sin badge en README y sin PAT. Fuera de `ci-gate`. La guía anual es `docs/security/scorecard-review.md`. Se implementa después de D1 y D3 para que la primera corrida vea canal y SAST ya descritos en el árbol.

## Orden y paralelismo

```text
Task 1 advisory-review
  → checkpoint supply-chain
Task 2 vuln-report-channel  ∥  Task 3 desktop verify
  → Task 4 codeql          (SECURITY.md después de Task 2)
  → checkpoint CI
Task 5 sbom-crates         ∥  (no toca ci.yml ni SECURITY.md)
Task 6 criterion           (sola: Cargo.toml / Cargo.lock)
  → checkpoint ola A
Task 7 asvs-matrix         (tras 2, 3 y 4)
Task 8 packaging draft     (tras 3)
Task 9 scorecard           (tras 2 y 4)
  → checkpoint ola B
```

Task 2 y Task 3 pueden ir en paralelo (ficheros distintos). Task 4 espera a Task 2 porque ambas tocan `SECURITY.md`. Task 5 puede ir en paralelo con Task 4. Task 6 no se mezcla con otro cambio de lock. Task 7, 8 y 9 pueden ir en paralelo entre sí cuando sus dependencias estén hechas: 7 y 9 no deben editar `SECURITY.md` a la vez que otra tarea abierta.

## Task List

Phase 3 copiará esta lista a `docs/security/audit-2026/TODO.md`. Aquí es el índice revisable.

### Ola A

- [ ] Task 1: Renovar revisión RustSec 2024-0429 / 2024-0370
- [ ] Task 2: Canal en `SECURITY.md` + versiones 0.6.x
- [ ] Task 3: Sección de verificación del AppImage en `docs/desktop.md`
- [ ] Task 4: Workflow CodeQL fuera de `ci-gate`
- [ ] Task 5: Workflow `sbom-crates` sin `cargo publish`
- [ ] Task 6: Benches `criterion` en `zedazo-core` (sí de dep = aprobar este plan)

### Checkpoint: supply-chain (tras Task 1)

- [ ] `make deny` ejecutado en la sesión
- [ ] `make audit` ejecutado en la sesión
- [ ] Ignores renovados solo si el advisory sigue sin parche; si hay parche, se para y se informa

### Checkpoint: CI (tras Tasks 2–4)

- [ ] `make docs-validate`
- [ ] `ci-gate` `needs` sin `codeql` ni `scorecard`
- [ ] Actions nuevas con SHA completo y comentario de versión
- [ ] `zizmor` sobre `.github/workflows` si el binario está en el entorno; si no, `not_run` con motivo

### Checkpoint: ola A (tras Tasks 5–6)

- [ ] `sbom-crates.yml` no contiene `cargo publish` ni `CRATES_IO_TOKEN`
- [ ] `cargo test -p zedazo-core` (tests no ignorados) verde si Task 6 tocó el crate
- [ ] `tests/dedup_perf.rs` sigue presente

### Ola B

- [ ] Task 7: Matriz `docs/security/asvs-l1-matrix.md`
- [ ] Task 8: Brecha + `ADR-DRAFT-packaging.md` (opción A)
- [ ] Task 9: Workflow Scorecard mensual + guía anual

### Checkpoint: ola B

- [ ] `make docs-validate`
- [ ] Diff de producto limitado a los ficheros de cada tarea; sin `apps/web/**` ni código de `apps/desktop/**`
- [ ] Ningún check nuevo marcado required

## Tasks (detalle)

### Task 1: Renovar revisión RustSec 2024-0429 / 2024-0370

**Description:** Confirmar que glib 0.18 y `proc-macro-error` siguen sin bump publicable y renovar la fecha de revisión. Redactar el body del issue de upgrade Tauri/gtk-rs en este directorio.

**Acceptance criteria:**

- [ ] `make deny` y `make audit` corridos; la salida queda citada en el commit o en la nota S-18
- [ ] Misma pareja de IDs en `deny.toml` y `.cargo/audit.toml`, con fecha nueva
- [ ] `osv-scanner.toml` menciona la misma fecha en el `reason` de esos IDs
- [ ] S-18 actualizado
- [ ] Body del issue en `docs/security/audit-2026/issue-tauri-glib.md`; issue de GitHub sin crear

**Verification:**

- [ ] `make deny`
- [ ] `make audit`
- [ ] `rg -n "RUSTSEC-2024-0429|RUSTSEC-2024-0370" deny.toml .cargo/audit.toml osv-scanner.toml`

**Dependencies:** None

**Implementation roles:** role-back, role-linux

**Test surfaces:** jobs `deny` y `supply-chain` (no se relajan)

**Files likely touched:**

- `deny.toml`
- `.cargo/audit.toml`
- `osv-scanner.toml`
- `docs/security/owasp-review-0.6.0.md`
- `docs/security/audit-2026/issue-tauri-glib.md`

**Estimated scope:** Small

### Task 2: Canal en SECURITY.md + versiones 0.6.x

**Description:** Sustituir la frase genérica a mantenedores por el enlace de avisos privados y alinear la tabla de soporte con el tag v0.6.4 (línea activa 0.6.x).

**Acceptance criteria:**

- [ ] `SECURITY.md` enlaza `https://github.com/Soluciones-Alexendros/zedazo/security/advisories/new`
- [ ] Sigue la prohibición de issue público y el plazo de 48 h
- [ ] La tabla marca 0.6.x como línea activa
- [ ] No aparece una dirección de correo inventada
- [ ] Checklist del operador: habilitar Private Vulnerability Reporting en la UI (no vía API en esta tarea)

**Verification:**

- [ ] `rg -n "security/advisories|0\\.6" SECURITY.md`
- [ ] `make docs-validate`

**Dependencies:** None (no en paralelo con Task 4)

**Implementation roles:** role-back

**Test surfaces:** `docs-validate`

**Files likely touched:**

- `SECURITY.md`

**Estimated scope:** Small

### Task 3: Verificación del AppImage en docs/desktop.md

**Description:** Añadir la sección de comprobación antes de «Ver también», con los comandos del job `verify-release`. Enlace cruzado a [`docs/release-validation.md`](../../release-validation.md).

**Acceptance criteria:**

- [ ] La sección incluye `sha256sum -c SHA256SUMS` y `gh attestation verify … --repo Soluciones-Alexendros/zedazo`
- [ ] Alcance AppImage (ADR-0021); tarballs cargo-dist quedan fuera
- [ ] Sin cambios en `.github/workflows/publish.yml`

**Verification:**

- [ ] `rg -n "sha256sum|attestation verify" docs/desktop.md`
- [ ] `make docs-validate`
- [ ] Manual contra un tag real: solo si `gh` está autenticado; si no, `not_run` con motivo

**Dependencies:** None

**Implementation roles:** role-linux

**Test surfaces:** documentación de release; job `verify-release` existente (no se modifica)

**Files likely touched:**

- `docs/desktop.md`

**Estimated scope:** Small

### Task 4: Workflow CodeQL fuera de ci-gate

**Description:** Añadir `.github/workflows/codeql.yml` según D3 y un párrafo en `SECURITY.md` que nombre CodeQL y las crates de foco.

**Acceptance criteria:**

- [ ] Action pinneada por SHA con comentario de versión
- [ ] Permisos: `actions: read`, `contents: read`, `security-events: write`
- [ ] El job no está en `needs` de `ci-gate`
- [ ] `SECURITY.md` menciona el workflow
- [ ] Si `languages: rust` no está soportado, la tarea se detiene sin Semgrep

**Verification:**

- [ ] `rg -n "codeql" .github/workflows/ci.yml .github/workflows/codeql.yml` muestra el workflow nuevo y ausencia en `ci-gate`
- [ ] `zizmor --format plain --min-severity medium .github/workflows/codeql.yml` si el binario existe; si no, `not_run`
- [ ] La corrida real es el primer workflow en GitHub; en local no se afirma PASS del análisis

**Dependencies:** Task 2

**Implementation roles:** role-back, role-linux

**Test surfaces:** workflow nuevo; `ci-gate` sigue agregando los jobs actuales

**Files likely touched:**

- `.github/workflows/codeql.yml`
- `SECURITY.md`

**Estimated scope:** Medium

### Task 5: Workflow sbom-crates sin cargo publish

**Description:** Workflow `workflow_dispatch` que genera el CycloneDX del manifiesto CLI, escribe checksum y lo sube al release del `ref` indicado. Documentar la descarga en `docs/release-validation.md` o en un párrafo de `docs/security/`.

**Acceptance criteria:**

- [ ] Comando alineado con el job `sbom` de `release.yml`
- [ ] Assets `sbom-crates.json` y `sbom-crates.json.sha256`
- [ ] El YAML no contiene `cargo publish` ni `CRATES_IO_TOKEN`
- [ ] Permiso `contents: write` solo en el job de upload
- [ ] Docs dicen que crates.io no aloja el SBOM

**Verification:**

- [ ] `rg -n "cargo publish|CRATES_IO_TOKEN" .github/workflows/sbom-crates.yml` vacío
- [ ] Generación local con `cargo cyclonedx` si el binario está instalado; si no, `not_run` (el install no es parte de la tarea salvo que ya sea el método del release)
- [ ] `make docs-validate`

**Dependencies:** None

**Implementation roles:** role-back, role-linux

**Test surfaces:** release de GitHub del tag elegido en el dispatch (operador); no el publish a crates.io

**Files likely touched:**

- `.github/workflows/sbom-crates.yml`
- `docs/release-validation.md`

**Estimated scope:** Medium

### Task 6: Benches criterion en zedazo-core

**Description:** Dos benches de dominio, fixtures sintéticos, sin presupuesto de CI. Se conserva `tests/dedup_perf.rs`.

**Acceptance criteria:**

- [ ] `criterion` solo en dev-dependencies de `zedazo-core`
- [ ] `benches/parse_vcf.rs` llama a `parse_vcards` con VCF sintético
- [ ] `benches/dedup.rs` llama a `deduplicate_with_ctrl` con contactos sintéticos (`example.invalid`)
- [ ] `cargo test -p zedazo-core` verde
- [ ] El test ignorado `dedup_40k_under_budget` no se borra ni se mete en `ci-gate`

**Verification:**

- [ ] `cargo test -p zedazo-core`
- [ ] `cargo bench -p zedazo-core --bench parse_vcf -- --noplot` (corrida corta)
- [ ] `cargo bench -p zedazo-core --bench dedup -- --noplot`
- [ ] `cargo deny check` tras el lock, si la licencia de `criterion` no está en el allow de `deny.toml` se para y se informa (no se amplía la allowlist sin sí)

**Dependencies:** None (no mezclar con otro cambio de `Cargo.lock`)

**Implementation roles:** role-back, role-clean-code

**Test surfaces:** `cargo test -p zedazo-core`; job `release-build` existente

**Files likely touched:**

- `crates/zedazo-core/Cargo.toml`
- `crates/zedazo-core/benches/parse_vcf.rs`
- `crates/zedazo-core/benches/dedup.rs`
- `Cargo.lock`

**Estimated scope:** Medium

### Task 7: Matriz ASVS L1

**Description:** Escribir `docs/security/asvs-l1-matrix.md` con evidencia del árbol después de Tasks 2–4. Lo no comprobado queda `no evaluado`.

**Acceptance criteria:**

- [ ] Declaración de objetivo L1 y una sola versión de ASVS
- [ ] Filas para canal de reporte, verificación de artefactos, SAST y controles ya descritos en `SECURITY.md` (token, TLS, cookie)
- [ ] Cada `cubierto` tiene evidencia (ruta o job)
- [ ] No se afirma certificación

**Verification:**

- [ ] `rg -n "cubierto|no evaluado" docs/security/asvs-l1-matrix.md`
- [ ] `make docs-validate`
- [ ] Muestreo de 5 filas `cubierto` con fichero existente

**Dependencies:** Task 2, Task 3, Task 4

**Implementation roles:** role-back

**Test surfaces:** documental

**Files likely touched:**

- `docs/security/asvs-l1-matrix.md`

**Estimated scope:** Medium

### Task 8: Borrador ADR de empaquetado

**Description:** Nota de brecha (ADR-0012, ADR-0021, `dist-workspace.toml`) y borrador con opciones A/B/C, default A, requisito de checksum + attestation.

**Acceptance criteria:**

- [ ] `docs/security/audit-2026/ADR-DRAFT-packaging.md` existe
- [ ] No hay diff en `DECISIONS.md` ni en `dist-workspace.toml`
- [ ] El borrador cita la sección de verificación de Task 3

**Verification:**

- [ ] `make docs-validate`
- [ ] `git diff --name-only` sin `DECISIONS.md` ni `dist-workspace.toml` en esta tarea

**Dependencies:** Task 3

**Implementation roles:** role-linux

**Test surfaces:** ninguna de CI

**Files likely touched:**

- `docs/security/audit-2026/ADR-DRAFT-packaging.md`
- `docs/security/audit-2026/packaging-gap.md`

**Estimated scope:** Small

### Task 9: Scorecard mensual

**Description:** Workflow programado, SARIF, guía de revisión anual vacía de la primera fila. Sin badge y sin entrar en `ci-gate`.

**Acceptance criteria:**

- [ ] Cron mensual + `workflow_dispatch`
- [ ] Action pinneada por SHA
- [ ] Permisos mínimos documentados en el YAML (`contents: read`, `security-events: write`, `id-token: write` si la action lo exige)
- [ ] `docs/security/scorecard-review.md` describe la revisión anual y dónde leer el resultado
- [ ] Ausente de `needs` de `ci-gate`

**Verification:**

- [ ] `rg -n "scorecard" .github/workflows/ci.yml` no añade el job al gate
- [ ] `zizmor` del workflow nuevo, o `not_run`
- [ ] Primera corrida real = `workflow_dispatch` en GitHub tras el merge; en local, `not_run`

**Dependencies:** Task 2, Task 4

**Implementation roles:** role-linux

**Test surfaces:** workflow `scorecard`; no el gate de merge

**Files likely touched:**

- `.github/workflows/scorecard.yml`
- `docs/security/scorecard-review.md`

**Estimated scope:** Medium

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| CodeQL no acepta Rust en el action pinneado | Alto | Parar Task 4; no sustituir por Semgrep en el mismo cambio |
| Un workflow nuevo queda required por el ruleset | Alto | No tocar rulesets; comprobar que el check exigido sigue siendo `ci-gate` |
| `criterion` o una transitiva fuera de `deny.toml` `[licenses]` | Medio | `cargo deny check` antes de cerrar Task 6; no ampliar allowlist sin sí |
| Renovar ignores esconde un parche ya publicable | Alto | Task 1 lee la salida de audit/deny antes de mover la fecha |
| `contents: write` en `sbom-crates` sube al release equivocado | Medio | Input `ref` explícito; el workflow no publica crates |
| Dos tareas editan `SECURITY.md` a la vez | Medio | Task 4 después de Task 2; Task 9 no edita ese fichero |
| Scorecard publica una foto previa al canal y al SAST | Bajo | Task 9 después de Tasks 2 y 4 |
| Enlace interno nuevo rompe `docs-validate` | Medio | `make docs-validate` en cada checkpoint documental |

## Open Questions

- Resuelta el 2026-11-18: el operador pidió ejecutar el plan **con Task 6**. `criterion` queda autorizado como dev-dependency de `zedazo-core`.
- ¿Se crea el issue de Tauri en GitHub al cerrar Task 1, o basta `issue-tauri-glib.md`?
- ¿La matriz ASVS cita 4.0.3 o 5.x? Se fija una versión al escribir Task 7.
- ¿El primer `workflow_dispatch` de SBOM y de Scorecard lo lanza el operador tras el merge?

## Fuera de este plan

- Clave minisign/GPG.
- CodeQL o Scorecard dentro de `ci-gate`.
- `cargo publish` y tags.
- Runners nuevos, `.deb`/`.rpm`, edición de ADR-0012 o ADR-0021.
- Fuzzing (S-17, fase C de la revisión OWASP).
- `apps/web` y código de `apps/desktop`.
