# Tasks: auditoría de seguridad 2026

Plan: [PLAN.md](./PLAN.md). Task 6 incluida por el operador (2026-11-18).
No sustituye a `tasks/todo.md`.

## Task 1: Renovar revisión RustSec — HECHA

- [x] `make deny` y `make audit` (2026-11-18). Audit: solo `RUSTSEC-2026-0543` chrono, aviso permitido. Deny: advisories/bans/licenses/sources ok. `RUSTSEC-2024-0429` sale como `advisory-not-detected` porque deny no trata `unsound` como vulnerability.
- [x] `glib` 0.18.5 sigue en `Cargo.lock` (parche >=0.20 ausente). `proc-macro-error` 1.0.4, RustSec `patched = []`.
- [x] Fecha de revisión 2027-02-18 en `deny.toml`, `.cargo/audit.toml` y `osv-scanner.toml`. Ignores no quitados.
- [x] S-18 actualizado. Body del issue en [issue-tauri-glib.md](./issue-tauri-glib.md). Issue de GitHub no creado.

## Task 2: Canal en SECURITY.md — HECHA

- [x] Enlace a `https://github.com/Soluciones-Alexendros/zedazo/security/advisories/new`.
- [x] Tabla de versiones con 0.6.x como línea activa.
- [x] Sin correo inventado. Private vulnerability reporting activado: la API devolvió `enabled: true` el 2026-11-18.

## Task 3: Verificación del AppImage — HECHA

- [x] Sección en `docs/desktop.md` con `sha256sum -c` y `gh attestation verify`.
- [x] `publish.yml` sin cambios.
- [x] Descarga real del tag: `not_run` (no se ha comprobado el asset del release en esta sesión).

## Task 4: CodeQL — HECHA en árbol

- [x] `.github/workflows/codeql.yml`, `github/codeql-action` `33d7b85da1a04171e211befe2c739303b9a4c066` (v4.39.3), `languages: rust`, `build-mode: none`.
- [x] Ausente de `ci-gate`.
- [x] Párrafo en `SECURITY.md`.
- [x] `zizmor` v1.31.1 sobre el workflow: sin findings (offline; 7 suppressed por la config del repo).
- [x] Corrida de CodeQL en GitHub: `not_run` en local.

## Task 5: SBOM crates — HECHA en árbol

- [x] `.github/workflows/sbom-crates.yml` sin `cargo publish` ni `CRATES_IO_TOKEN`.
- [x] Nota en `docs/release-validation.md`.
- [x] `cargo cyclonedx` local: `not_run` (no se ha generado el JSON en esta sesión).
- [x] `zizmor` sin findings.

## Task 6: criterion — HECHA

- [x] `criterion` 0.5.1, dev-dependency de `zedazo-core`, sin features default.
- [x] `cargo test -p zedazo-core`: 152 unit + integración verdes; `dedup_perf` sigue ignorado.
- [x] `parse_vcards_100`: ~64 µs. `dedup_512`: ~373 µs (`--noplot`).
- [x] `cargo deny check` tras el lock: licenses ok. No se amplió `deny.toml`.

## Checkpoint ola A

- [x] `make docs-validate`
- [x] `make deny` y `make audit`

## Task 7: Matriz ASVS L1 — HECHA

- [x] `docs/security/asvs-l1-matrix.md`, ASVS 4.0.3, objetivo L1.
- [x] Capítulos no recorridos declarados `no evaluado`.

## Task 8: Borrador de empaquetado — HECHA

- [x] `packaging-gap.md` y `ADR-DRAFT-packaging.md` (opción A).
- [x] Sin diff de `DECISIONS.md` ni `dist-workspace.toml` en esta tarea.

## Task 9: Scorecard — HECHA en árbol

- [x] `.github/workflows/scorecard.yml`, cron mensual, `ossf/scorecard-action` `fa7117db83c52d787ae5a89f350999701505d78e` (v2.5.0).
- [x] `docs/security/scorecard-review.md` con la primera fila vacía.
- [x] Ausente de `ci-gate`. `zizmor` sin findings.
- [x] Primer `workflow_dispatch`: `not_run` (lo lanza el operador tras el merge).

## Checkpoint ola B

- [x] `make docs-validate` (enlaces internos OK).
