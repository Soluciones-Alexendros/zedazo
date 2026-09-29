---
version: "0.3.2"
date: "2026-09-09"
status: "Activo"
canonical: true
supersedes: "v0.3.1"
---

# ROADMAP.md

**Versión:** 0.3.2  
**Fecha:** 2026-09-09  
**Canónico:** este archivo. [`docs/tasks.md`](docs/tasks.md) redirige aquí.

---

## Reglas

- Cada hito referencia requisitos de [SPECS.md](./SPECS.md).
- Desviaciones relevantes → [DECISIONS.md](./DECISIONS.md).
- Trabajo de features en PRs pequeños; CI verde antes de pedir merge.
- Estimación relativa: S / M / L.

## Estado actual

| Dimensión | Estado |
|-----------|--------|
| Versión publicada | crates.io pendiente de tag **v0.5.1**; workspace Cargo.toml **0.5.1**; código = hitos v0.5.0 (GUI) + v0.5.1 (remoto ADR-0016) |
| Tests | unitarios core + verificación I4–I6 + integración; parity O10; `auth_http`; web (`tsc`/`lint`/`next build`) |
| CI | `ubuntu-latest`; Renovate; `make ci` = fmt + clippy + test + check + doc + docs-validate + parity + web-ci |
| GUI/API | Local loopback (ADR-0015) + remoto HTTPS+token (ADR-0016, O11) |
| Branch protection | Activa en `main` |
| Backlog trazable | Issues GitHub (post-cierre #24–27/#35: TMview humano; CardDAV v0.4; OTel post-v1.0) |

## Hitos

### v0.1.1 — Metadata y robustez — S

**Criterio de salida:**
- [x] `repository`/`homepage` y badges → org `Soluciones-Alexendros`
- [x] Sin `unwrap` panic-prone en `domain/` (C-01, C-02)
- [x] Tag `v0.1.1` + crates.io; aviso rename en **0.1.2** → `zedazo` (sin yank)

### v0.2.0 — Rename a Zedazo (migración + base phase4/5 ya en main) — M

**Criterio de salida:**
- [x] Crate/binario `zedazo`; repo `Soluciones-Alexendros/zedazo`
- [x] Props `X-ZEDAZO-*`, TOML `[zedazo]` (+ alias `[cribado]`), JSON `zedazo_result`
- [x] ADR-0014; CHANGELOG con tabla de migración
- [x] Subcomando `cribar` sin cambio (verbo de dominio)
- [x] `make ci` verde; docs canónicos alineados
- [x] Campo ADR + taxonomía N3 / tipos T4 (#28)
- [x] Pipeline + config TOML enriquecida (#29)
- [x] Tag `v0.2.0` + crates.io `zedazo`

**Fuera de alcance v0.2.0:** CardDAV, marca registrada, dominio de pago.

### v0.3.0 — Calidad y robustez — L

**Criterio de salida:**
- [x] Reglas C1, C5, C7, E4, E6 con tests (`screening.rs` 272–465 + tests 628–678, #21)
- [x] `domain::verification` aplica I1–I7 en pipeline (`verify` pre + `verify_post` post-escritura I4/I5/I6 en `cribar.rs:258`)
- [x] Documentación canónica raíz alineada (SPECS/ROADMAP/DECISIONS/AGENTS → 0.3.0 2026-09-02)
- [x] Issues #21/#25/#26/#27/#35 documentados para cierre en PR 0.3.0; cierre GitHub formal en chore/close-backlog-issues (2026-09-09)

**Fuera de alcance v0.3.0:** CardDAV, watch mode, GUI (GUI abierta en v0.5.0 vía ADR-0015).

### v0.4.0 — Integraciones — L

*(antes numerado v0.3.0)*

- CardDAV sync, watch mode, filtros por categoría
- Requiere ADR de proveedor/red antes de implementar

### v0.5.0 — Web self-hosted (GUI + API) — L

- [x] Workspace `zedazo-core` / `zedazo-cli` / `zedazo-api` (ADR-0015)
- [x] API HTTP `/api/v1` (Axum): jobs, uploads, SSE, artefactos
- [x] Frontend Next.js `apps/web`: paridad funcional con CLI (excluye `completions`)
- [x] Modo local single-user; storage efímero FS+manifest; sin telemetría remota por defecto
- [x] Matriz de paridad + tests de equivalencia CLI↔API sobre fixtures (`make parity`)
- [x] Docs: [docs/gui/](./docs/gui/), [docs/api/openapi.yaml](./docs/api/openapi.yaml)

**Criterio de salida:** criterios V1 local en ADR-0015 / plan GUI; `make ci` verde incluyendo contrato API y O10. Tag/release a confirmación humana.

### v0.5.1 — Hardening remoto self-hosted (ADR-0016) — M

- [x] Auth `token` / `disabled` fail-closed; login/logout cookie; Bearer
- [x] CORS acotado en modo token; health público
- [x] Compose remoto + Caddy same-origin/SSE; `.env.example`
- [x] GUI: pantalla de acceso; `credentials` + EventSource `withCredentials`
- [x] Docs: threat-model, deploy (este host → miniPC / túnel / Let's Encrypt)
- [x] Tests auth + `make ci` (O10 intacto)

**Criterio de salida:** O11; despliegue remoto usable sin publicar API/web directamente; CI verde. Tag `v0.5.1` (y bump crates.io) a confirmación humana.

### v1.0.0 — Producción — L

- API de crate estable (semver estricto del core/CLI)
- Benchmarks (criterion)
- Cross-compile macOS/Windows — artefactos vía **cargo-dist** (ADR-0012); validar releases reales en el hito
- Corpus de regresión >10k contactos
- GUI/API maduras (post-v0.5.0) opcionales en distribución Docker

### Post-v1.0 — Observabilidad (aplazado)

- OpenTelemetry / OTLP opt-in — **ADR-0017** (sin deps hasta entonces; ver [docs/otel.md](./docs/otel.md))

## Dependencias congeladas (ver DECISIONS)

- `nom` < 8, `toml` < 1, `chardetng` < 1 (Renovate `allowedVersions`)
