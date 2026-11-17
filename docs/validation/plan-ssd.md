# Plan SSD de arreglos y soluciones — zedazo

**Fecha**: 2026-11-17 · Clave: S=Severidad, S=Solución, D=Detalle/verificación.
Ordenado por prioridad. Antes de tocar nada: `git pull --ff-only` (estamos 1 commit detrás).

## S1 — Reparar advisory RustSec (P0, bloqueante)

**Problema**: RUSTSEC-2026-0538 `quinn-udp 0.5.15` (bucle de receive tras datagrama
truncado). Rompe `Security Audit`, `Security/sca` y `ci-gate`.

**Solución**:
```bash
cargo update -p quinn-udp   # resuelve 0.5.15 → 0.5.16 (verificado en dry-run)
```

**Verificación**: `cargo audit` debe salir sin `error: 1 vulnerability found` y
`make ci` (o al menos `cargo clippy --workspace -- -D warnings && cargo test --workspace --all-features`)
verde. Commit + PR a rama de trabajo.

**Alcance**: `Cargo.lock` (1 crate). **Exclusiones**: no tocar `reqwest`/`quinn` major.

---

## S2 — Cerrar alertas Dependabot medium (P1, seguridad npm)

**Problema**: `minimatch` CVE-2026-118764 y `glob` CVE-2026-118773 (transitivas en `apps/web`).

**Solución**: aprobar/crear y fusionar las actualizaciones Renovate pendientes en el
Dependency Dashboard (#89): `brace-expansion@1 → 1.1.22`, `brace-expansion@5 → 5.0.13`,
`minimatch → 11.0.8` y la subida de `glob` que resuelva el advisory.

**Verificación**: `pnpm install --frozen-lockfile && pnpm audit --prod --audit-level=high`
sin hallazgos; `make web-ci` verde. Confirmar en GitHub que las 2 alertas pasan a `fixed`.

---

## S3 — Corregir desajuste MSRV en Renovate lock-file-maintenance (P1)

**Problema**: el refresh de locks sube `encoding_rs` 0.8.44 (rustc ≥1.88) y `uuid`
1.29.2 (≥1.89), pero `deploy/Dockerfile.api` usa `rust:1.85-bookworm` y el repo declara
MSRV 1.80. Compose smoke falla en #114; también rompería el check MSRV al fusionar.

**Decisión tomada (operador)**: Opción A — mantener MSRV 1.80 con pins en Renovate.

**Solución**:
- Añadir en `.github/renovate.json`, en `packageRules` para el manager `cargo`:
  - `{"matchPackageNames":["encoding_rs"],"allowedVersions":"<0.8.40"}` — 0.8.40+ declara rust-version 1.88; 0.8.39 es el último compatible (0.8.35 actual en lock).
  - `{"matchPackageNames":["uuid"],"allowedVersions":"<1.27.0"}` — 1.27.0+ declara rust-version 1.89; 1.26.0 actual compila en MSRV 1.80 y rust:1.85.
- El detalle técnico completo, tareas y verificación viven en `tasks/plan.md` / `tasks/todo.md`.

**Verificación**: re-ejecutar el Compose smoke (`make compose-smoke`) y el check MSRV 1.80 en verde sobre el lock refrescado.

**Alcance**: `.github/renovate.json`. **Exclusiones**: no subir MSRV (Opción B descartada); no tocar `Cargo.toml` ni `deploy/Dockerfile.api`.

---

## S4 — Estabilizar Desktop e2e `unexpected alert open` (P1)

**Problema**: los jobs `Desktop e2e (tauri-driver)` y `(no network)` fallan en #114 con
`WebDriverError: unexpected alert open`, verdes en main.

**Solución**: reproducir localmente; identificar el `alert()`/`confirm()` del front o del
WebView; añadir manejo de diálogo en el spec (`dismissAlert`/`acceptAlert` de WebdriverIO)
o corregir el disparo si es un bug real de la GUI.

**Verificación**: `make web-ci`/e2e desktop verde en 3 ejecuciones consecutivas (descartar flaky).

---

## S5 — Consolidar fixture duplicado (P2, higiene)

**Problema**: `crates/zedazo-core/tests/fixtures/sample-contacts.vcf` es byte-idéntico a
`examples/sample.vcf`.

**Solución**: eliminar `examples/sample.vcf` si no lo referencia `README.md`/docs, o
convertir `examples/sample.vcf` en referencia documental al fixture canónico de tests.

**Verificación**: `rg -l "sample.vcf"` sin referencias rotas; `make ci` verde.

---

## S6 — Añadir `.env.example` (P2, operación)

**Problema**: las variables `ZEDAZO_*` (auth, static dir, CardDAV) no tienen plantilla.

**Solución**: crear `.env.example` en la raíz con las claves documentadas en
`README.md`/`docs/` (valores de ejemplo, sin secretos) y enlazarlo desde el README.

**Verificación**: `gitleaks` (job secrets) sigue verde; sin secretos reales en el diff.

---

## S7 — Planificar migración de avisos informativos (P2, no bloqueante)

**Problema**: `chrono` unmaintained y `rustix` unsound (warnings; no rompen el gate).

**Solución**: registrar en `DECISIONS.md`/`ROADMAP.md` la hoja de ruta:
- `chrono → jiff` en `zedazo-api` (jiff ya está en el árbol) y seguimiento del upgrade de Tauri
  (chrono transitivo vía `serde_with`/`tauri-utils`).
- `rustix` vía `tempfile`: revisión al publicar parche.

**Verificación**: N/A ahora; revisión documentada con fecha (mismo patrón que
`.cargo/audit.toml`).

---

## Orden de ejecución recomendado

1. `git pull --ff-only` → S1 (quinn-udp) → PR y CI verde.
2. S2 (Dependabot npm) → S3 (Renovate MSRV, con decisión del operador) → S4 (e2e).
3. S5/S6/S7 en paralelo, sin urgencia.

**Criterio de cierre global**: `Security Audit`, `Security`, y `ci-gate` verdes en main;
0 alertas Dependabot `open` de severidad ≥ medium; `make ci` local verde.
