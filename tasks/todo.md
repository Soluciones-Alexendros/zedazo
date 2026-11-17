# Tasks: remediación de hallazgos P0–P2

> Plan en `tasks/plan.md`. Orden bottom-up; cada tarea deja el repo verde.
> Verificar siempre con el comando real del repo antes de marcar como hecho.

## Task 1: Bump quinn-udp 0.5.15 → 0.5.16 (RUSTSEC-2026-0538)

- [ ] **Descripción**: parchear `quinn-udp` (bucle de receive tras datagrama truncado)
      vía `cargo update -p quinn-udp`. Cadena: `quinn 0.11.11 → reqwest 0.12.28 → zedazo-carddav`.
- [ ] Acceptance: `cargo audit` termina sin `error: 1 vulnerability found`; solo quedan
      los 2 avisos permitidos (`chrono`, `rustix`).
- [ ] Verification: `cargo audit`; `cargo clippy --workspace --all-targets --all-features --exclude zedazo-desktop -- -D warnings`;
      `cargo test --workspace --all-features`; CI `Security Audit` + `Security/sca` + `ci-gate` verdes.
- [ ] Dependencies: Ninguna
- [ ] Implementation roles: — (cambio de lockfile; sin lógica de dominio/red)
- [ ] Test surfaces: `cargo audit`; suite Rust workspace completa
- [ ] Files: `Cargo.lock`
- [ ] Scope: XS (1 fichero)

## Task 2: Resolver Dependabot minimatch + glob (apps/web)

- [ ] **Descripción**: cerrar las 2 alertas abiertas: `minimatch`
      (CVE-2026-118764, GHSA-g3hv-75wg-m249, vulnerable `>=7.0.0,<11.0.1`, parche `11.0.1`)
      y `glob` (CVE-2026-118773, GHSA-3g3m-2fjr-6pw4, vulnerable `>=9.0.0,<=14.0.1`,
      parche `14.0.2`). Localizar el origen transitivo con `pnpm why` y subir el rango
      correspondiente (override `pnpm.overrides` o actualización Renovate).
- [ ] Acceptance: `pnpm audit --prod --audit-level=high` sin hallazgos; ambas alertas
      pasan a `fixed` en GitHub; sin regresión en `apps/web`.
- [ ] Verification: `pnpm install --frozen-lockfile && pnpm audit --prod --audit-level=high`;
      `make web-ci`; comprobar en la pestaña Dependabot que quedan 0 `open` medium+.
- [ ] Dependencies: Ninguna
- [ ] Implementation roles: — (gestión de dependencias; sin código)
- [ ] Test surfaces: `pnpm audit`; suite `apps/web` (web-ci: build + Playwright + a11y)
- [ ] Files: `apps/web/package.json` (overrides), `apps/web/pnpm-lock.yaml`
- [ ] Scope: S (1–2 ficheros + lock)

## Task 3: Pins Renovate Opción A (encoding_rs <0.8.40, uuid <1.27.0)

- [ ] **Descripción**: el lock-file-maintenance de Renovate rompe el build al subir
      `encoding_rs` (0.8.40+ declara rustc 1.88) y `uuid` (1.27.0+ declara rustc 1.89)
      por encima de `rust:1.85-bookworm` y MSRV 1.80. Añadir pins `allowedVersions` en
      `.github/renovate.json`, manager `cargo`, sin subir MSRV.
- [ ] Acceptance: los pins figuran en `packageRules` con comentario de motivo/fecha;
      un refresh de lock deja `make compose-smoke` y el check MSRV 1.80 verdes.
- [ ] Verification: `make compose-smoke`; re-ejecutar `cargo update -p encoding_rs -p uuid --dry-run`
      y confirmar que quedan anclados; validar JSON de `.github/renovate.json`; CI del PR
      de lock maintenance verde (o al menos sin fallo de compose/MSRV).
- [ ] Dependencies: Task 1 (mismo grafo Rust)
- [ ] Implementation roles: — (configuración de Renovate)
- [ ] Test surfaces: Compose smoke (Docker), check MSRV 1.80, job `Security`
- [ ] Files: `.github/renovate.json`
- [ ] Scope: S (1 fichero). Nota P9: `.github/**` → fusión con `policy-exception` por el operador.

## Task 4: Estabilizar Desktop e2e `unexpected alert open`

- [ ] **Descripción**: el spec `i7-sample.mjs` dispara el flujo «Borrar todos los datos»,
      que usa `window.confirm/alert` nativos (`apps/desktop/src/menu_es.rs`), y WebDriver
      falla con `unexpected alert open` al navegar después. Hacer el flujo determinista:
      aceptar/descartar el diálogo explícitamente tras la acción y antes de cualquier
      `browser.url`. No se sustituyen los diálogos nativos (fuera de alcance, D4).
- [ ] Acceptance: `Desktop e2e (tauri-driver)` y `(no network)` verdes en ≥3 ejecuciones
      consecutivas; sin `WebDriverError: unexpected alert open` en logs.
- [ ] Verification: `bash scripts/desktop/e2e-tauri-driver.sh` y `bash scripts/desktop/e2e-no-network.sh`
      ×3; `make web-ci` (no regresión web).
- [ ] Dependencies: Ninguna (independiente de Rust/npm)
- [ ] Implementation roles: role-front (flujos de interacción) + plan-validators (bucle de test)
- [ ] Test surfaces: e2e de escritorio (tauri-driver, no-network); flujo de borrado local
- [ ] Files: `scripts/desktop/e2e/i7-sample.mjs`
- [ ] Scope: M (1 fichero, con iteración de depuración)

## Task 5: Consolidar fixture duplicado sample.vcf

- [ ] **Descripción**: `crates/zedazo-core/tests/fixtures/sample-contacts.vcf` es
      byte-idéntico a `examples/sample.vcf` (sha256 `14212ea18318f285`). Dejar un único
      fixture canónico y referenciar desde `examples/` o eliminarlo.
- [ ] Acceptance: un solo fichero fuente; `rg -l "sample.vcf"` sin referencias rotas;
      `make ci` verde.
- [ ] Verification: `rg -l "sample.vcf"`; `cargo test --workspace --all-features`
- [ ] Dependencies: Ninguna
- [ ] Implementation roles: role-clean-code (limpieza de duplicación)
- [ ] Test surfaces: tests que usan el fixture (zedazo-core)
- [ ] Files: `examples/sample.vcf` (o el fixture de tests), README si lo referencia
- [ ] Scope: XS

## Task 6: Añadir .env.example

- [ ] **Descripción**: crear `.env.example` con las claves `ZEDAZO_*` documentadas
      (auth, static dir, CardDAV, bind) y valores de ejemplo sin secretos; enlazarlo
      desde `README.md`.
- [ ] Acceptance: fichero presente y documentado; `gitleaks` (job secrets) sin hallazgos.
- [ ] Verification: `bash -n`/`shellcheck` si aplica; job `Security/secrets` verde
- [ ] Dependencies: Ninguna
- [ ] Implementation roles: — (documentación de operación)
- [ ] Test surfaces: secret scanning (gitleaks)
- [ ] Files: `.env.example`, `README.md`
- [ ] Scope: XS

## Task 7: Registrar plan chrono→jiff y revisión rustix

- [ ] **Descripción**: anotar en `DECISIONS.md` (y `ROADMAP.md` si procede) la hoja de
      ruta de los avisos informativos no bloqueantes: `chrono` unmaintained (migrar a
      `jiff`, ya en el árbol; seguimiento del transitivo Tauri) y `rustix` unsound
      (revisión al publicar parche). Sin cambios de código.
- [ ] Acceptance: entradas con motivo y fecha de revisión (patrón `.cargo/audit.toml`);
      `make docs-validate` verde.
- [ ] Verification: `make docs-validate`
- [ ] Dependencies: Ninguna
- [ ] Implementation roles: — (documentación)
- [ ] Test surfaces: docs-validate (frontmatter, enlaces, trazabilidad)
- [ ] Files: `DECISIONS.md`, `ROADMAP.md` (opcional)
- [ ] Scope: XS

## Checkpoints

- [ ] **Checkpoint A (tras Task 1)**: `cargo audit` limpio; `Security Audit` + `Security/sca` + `ci-gate` verdes.
- [ ] **Checkpoint B (tras Tasks 2–3)**: `pnpm audit` limpio; 0 alertas Dependabot open; `make compose-smoke` + MSRV 1.80 verdes.
- [ ] **Checkpoint C (final)**: `make ci` verde; `ci-gate` + `Security` + `Web` + `Desktop e2e` verdes en CI.
