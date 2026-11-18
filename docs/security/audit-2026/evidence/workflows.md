# Evidencia: workflows de seguridad y CI

**Fecha de auditoría:** 2026-11-18  
**Alcance:** `.github/workflows/ci.yml` (job `ci-gate`) y workflows paralelos `codeql.yml`, `scorecard.yml`, `sbom-crates.yml`.  
**Método:** Revisión estática del repositorio en el working tree (sin `workflow_dispatch` ni ejecución en GitHub).

## Resumen ejecutivo

Los análisis CodeQL, OpenSSF Scorecard y la generación manual de SBOM de crates **no bloquean** el check requerido `ci-gate`. Los tres workflows revisados **fijan** las acciones de terceros por SHA completo con comentario de versión en cada referencia `uses:`.

## Hallazgo 1 — `ci-gate` no depende de workflows de seguridad asíncronos

**Archivo:** `.github/workflows/ci.yml`  
**Job:** `ci-gate` (líneas 714–760)

El bloque `needs:` enumera únicamente jobs definidos en el mismo workflow `ci.yml`. **No** aparecen:

| Workflow / job esperado | Presente en `needs` |
| --- | --- |
| `codeql.yml` (job `analyze`) | **No** |
| `scorecard.yml` (job `analysis`) | **No** |
| `sbom-crates.yml` (job `sbom`) | **No** |

**Jobs incluidos en `ci-gate` (referencia):** `health`, `check`, `msrv`, `fmt`, `clippy`, `test`, `doc`, `docs-validate`, `parity`, `web`, `web-real-api`, `compose-smoke`, `deny`, `coverage`, `gitleaks`, `supply-chain`, `zizmor`, `hadolint`, `dockerfile-digests`, y la cadena `desktop-*` (`desktop-appimage`, `desktop-smoke`, `desktop-e2e`, `desktop-no-network`, `desktop-parity`, `desktop-test`, `desktop-sbom`, `desktop-attest`).

**Implicación:** Un PR puede fusionarse con `ci-gate` verde aunque CodeQL en PR, Scorecard mensual o `sbom-crates` (dispatch manual) no se hayan ejecutado o fallen; la seguridad de merge descansa en el rollup de `ci.yml` más los jobs de `reusable-security.yml` ya enlazados (`gitleaks`, `supply-chain`).

**Nota:** Los propios workflows auditados declaran explícitamente estar fuera de `ci-gate` (`codeql.yml` comentario en cabecera; `scorecard.yml` línea 3).

## Hallazgo 2 — Fijación por SHA de acciones (pinning)

Criterio: cada `uses:` de acción reutilizable debe usar `@<40-caracteres-hex>` seguido de comentario `# vX.Y.Z` (o etiqueta equivalente documentada).

### `codeql.yml`

| Paso | Referencia | Comentario versión |
| --- | --- | --- |
| checkout | `actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0` | `# v7.0.0` |
| init | `github/codeql-action/init@33d7b85da1a04171e211befe2c739303b9a4c066` | `# v4.39.3` |
| analyze | `github/codeql-action/analyze@33d7b85da1a04171e211befe2c739303b9a4c066` | `# v4.39.3` |

**Resultado:** **CUMPLE** — tres referencias, todas con SHA y comentario.

### `scorecard.yml`

| Paso | Referencia | Comentario versión |
| --- | --- | --- |
| checkout | `actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0` | `# v7.0.0` |
| Run Scorecard | `ossf/scorecard-action@fa7117db83c52d787ae5a89f350999701505d78e` | `# v2.5.0` |
| Upload SARIF | `github/codeql-action/upload-sarif@33d7b85da1a04171e211befe2c739303b9a4c066` | `# v4.39.3` |

**Resultado:** **CUMPLE** — tres referencias, todas con SHA y comentario.

### `sbom-crates.yml`

| Paso | Referencia | Comentario versión |
| --- | --- | --- |
| checkout | `actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0` | `# v7.0.0` |
| toolchain | `dtolnay/rust-toolchain@b725eda525362e7cf34af9a7243cac1e3f01a7ec` | `# stable` |

**Resultado:** **CUMPLE** — dos referencias de acción; instalación de `cargo-cyclonedx` vía `run:` (no acción de marketplace).

No se observan referencias del tipo `@v4` sin SHA en estos tres archivos.

## Hallazgo 3 — Disparadores y superficie operativa (contexto)

| Workflow | Disparo principal | En `ci-gate` |
| --- | --- | --- |
| `codeql.yml` | `push`/`pull_request` en rutas críticas + `workflow_dispatch` | No |
| `scorecard.yml` | cron mensual + `workflow_dispatch` | No |
| `sbom-crates.yml` | solo `workflow_dispatch` (input `ref`) | No |

Esta separación es coherente con un gate de PR centrado en calidad, pruebas, supply-chain en PR (`gitleaks`, `supply-chain`, `deny`) y escritorio, sin acoplar SAST programado ni publicación de SBOM al release.

## Conclusión

| Control verificado | Estado |
| --- | --- |
| `codeql` / `scorecard` / `sbom-crates` ausentes de `ci-gate.needs` | **Confirmado** |
| Pinning SHA + comentario de versión en los tres workflows | **Confirmado** |

**Riesgo residual aceptado (documentado en repo):** análisis CodeQL y Scorecard no son prerrequisito de auto-merge; la cobertura depende de ejecución por path filter, schedule o intervención manual (`sbom-crates`).