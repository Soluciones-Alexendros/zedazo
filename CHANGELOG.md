# Changelog

Todas las modificaciones notables de este proyecto se documentan en este archivo.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [Unreleased]

### Fixed
- Release: job `Publish to crates.io` con `if: always() && needs.host.result == 'success'` (evita skip cuando los builds de cargo-dist se saltan); workflow manual `publish-crates.yml`

### Changed
- Cierre documentado backlog: Coveralls badge restaurado (#25); ADR-0012/0013 aceptadas (#26/#27); OTel aplazado ADR-0017 (#24); dominio `zedazo.alexendros.dev` (#35)
- Docs: `product-card.md` alineado con ROADMAP; runners `ubuntu-latest` en `contract.md` y `ARCHITECTURE.md` (enmienda ADR-0008)
- GUI: favicon + `theme-color`; cabeceras de seguridad en Caddy/Next; Playwright e2e/a11y en `web-ci` y CI

## [0.5.1] - 2026-09-09

### Changed
- Workspace Cargo.toml `0.3.0` → `0.5.1` (alineado con hitos GUI/remoto)
- Release: `cargo publish -p zedazo-core` y luego `-p zedazo` (workspace virtual; `zedazo-api` no se publica)

### Añadido (ADR-0016 — exposición remota self-hosted)
- Auth `ZEDAZO_AUTH_MODE=token|disabled` con fail-closed fuera de loopback
- `POST /api/v1/auth/login` / `logout`; Bearer y cookie HttpOnly `zedazo_auth` (SSE)
- Compose remoto [`deploy/docker-compose.remote.yml`](deploy/docker-compose.remote.yml) + Caddy same-origin; `.env.example`; `Caddyfile.public`
- GUI: pantalla `/acceso`, `credentials: include`, EventSource `withCredentials`
- Tests `crates/zedazo-api/tests/auth_http.rs`; SPECS **O11**; OpenAPI 0.5.1
- Docs: threat-model y deploy (este host → miniPC / túnel / Let's Encrypt)

## [0.5.0] - 2026-09-08

> Hito de código en `main` **sin tag ni artefactos** en GitHub Releases/crates.io.
> El primer release con binarios GUI/API es **v0.5.1**.

### Añadido (ADR-0015 — GUI web self-hosted)
- Workspace Rust: `zedazo-core`, `zedazo-cli` (binario `zedazo`), `zedazo-api` (Axum)
- Contrato `ProcessRequest` / `ProcessResult`, progress y cancelación cooperativa
- API HTTP `/api/v1` (uploads, jobs, SSE, contactos, duplicados, audit, stats, artefactos, reglas)
- Frontend Next.js `apps/web` (Procesar, Ejecuciones, exploración, identidad Archivo Vivo)
- Docs GUI: matriz de paridad, threat model, retención, estados de job, OpenAPI, deploy
- Test de equivalencia CLI/core `equivalence_cli_api` (multi-fixture + artefactos)
- Harness O10 CLI↔API HTTP `zedazo-api/tests/equivalence_http.rs` + `make parity` en CI
- Jobs CI: `docs-validate`, `parity`, `web` (`apps/web`)
- GUI/API V1: cancelación en UI, SSE con `events.ndjson` + `last_event_id`, wipe en Ajustes, warnings `[cribado]`, evidencias dedup
- `deploy/docker-compose.yml` + Dockerfiles

## [0.3.0] - 2026-09-02

### Añadido
- Reglas de cribado C1 (contacto sin datos útiles), C5 (capitalización sospechosa), C7 (vCard malformada), E4 (duplicado por email normalizado), E6 (teléfono no normalizable) con tests (`screening.rs:272-465`, [#21](https://github.com/Soluciones-Alexendros/zedazo/issues/21))
- Verificación post-escritura I4/I5/I6: `domain::verification::verify_post` valida salida VCF 4.0 (folding ≤75 octetos, VERSION 4.0, sin AGENT/LABEL/MAILER) y `audit.tsv` (11 columnas, filas, UID/REGLA) integrada en `cribar.rs:258` (warnings no críticos)
- 6 tests de verificación I4/I5/I6 + `verify_post` (`verification.rs:420-512`)

### Changed
- Columna CSV: `CRIBADO_RESULT` → `CLASSIFY_RESULT` (inglés, coherente con cabeceras vCard; ADR-0014 I-12). JSON `zedazo_result` y props `X-ZEDAZO-*` sin cambio.
- README: badge Coveralls oculto hasta [#25](https://github.com/Soluciones-Alexendros/zedazo/issues/25).
- Docs canónicos alineados a 0.3.0: `SPECS`/`ROADMAP`/`DECISIONS`/`AGENTS` (v0.3.0 2026-09-02)
- ROADMAP: hito v0.3.0 marcado completo; estado tests 150 + 17 integración

## [0.2.0] - 2026-08-15

### Changed
- **Rename de producto:** `vcf-cribador` → **Zedazo** (crate/binario `zedazo`). Ver [ADR-0014](DECISIONS.md).
- Props VCF de salida: `X-CRIBADO-*` → `X-ZEDAZO-*`
- Sección TOML canónica: `[zedazo]` (alias `[cribado]` con warning de deprecación)
- Campo JSON export: `cribado_result` → `zedazo_result`
- Convención de config documentada: `zedazo.toml` (path libre vía `-c`)
- Help CLI: sufijo de salida `<input>_zedazo.vcf`
- URLs de repo/docs → `Soluciones-Alexendros/zedazo`
- ROADMAP: rename = **v0.2.0**; integraciones → **v0.4.0**

### Añadido
- Taxonomía de clasificación ampliada a tres niveles (N1/N2/N3)
- Soporte completo para tipos de teléfono T4 con mapeo desde vCard
- Soporte para propiedades ADR (direcciones postales) en parser, `Contact`, writer y exports
- Regla E3 de huérfanos actualizada para considerar direcciones
- Módulos `domain::audit` y `domain::verification` (invariantes)
- Pipeline de aplicación conectado de extremo a extremo con configuración TOML enriquecida

### Migración desde vcf-cribador

| Elemento | Antes | Ahora |
|---|---|---|
| Instalar | `cargo install vcf-cribador` | `cargo install zedazo` |
| Invocación | `vcf-cribador cribar …` | `zedazo cribar …` |
| Props VCF | `X-CRIBADO-RESULT\|VERSION\|DATE` | `X-ZEDAZO-RESULT\|VERSION\|DATE` |
| TOML | `[cribado]` | `[zedazo]` (alias `[cribado]` deprecado) |
| Config file (docs) | `cribador.toml` | `zedazo.toml` |
| JSON | `cribado_result` | `zedazo_result` |
| Subcomando | `cribar` | `cribar` (sin cambio) |

El crate `vcf-cribador` en crates.io permanece publicado (sin yank); la última línea 0.1.x apunta a `zedazo`.

## [0.1.1] - 2026-08-15

### Corregido
- Metadata `repository`/`homepage` en `Cargo.toml` tras transferencia a la org `Soluciones-Alexendros`
- Badges y enlaces del README apuntando a la cuenta personal antigua
- Enlace de licencia dual (`LICENSE-MIT` / `LICENSE-APACHE`)
- Fecha incorrecta de `0.1.0` en el changelog (2025 → 2026)
- Panic potencial en `normalize_fn` al extraer roles (C-01)
- Panic potencial en `deduplicate` al materializar clusters (C-02)

## [0.1.0] - 2026-07-08

### Añadido
- Pipeline completo de cribado: parsing → normalización → clasificación → screening → dedup → escritura
- Comando `cribar` con soporte `--dry-run`
- Comando `audit` para auditoría standalone sin modificar VCF
- Comando `stats` con formatos texto, JSON y Markdown
- Comando `export` con formatos CSV y JSON
- Parser vCard 4.0/3.0 RFC 6350 con unfold, desescapado y propiedades agrupadas
- Compatibilidad automática vCard 3.0 → 4.0 (Google, Apple, Proton)
- Detección automática de fuente (ProtonAutosave, GoogleContacts, Apple)
- Transcodificación ISO-8859-1 → UTF-8 (chardetng + encoding_rs)
- Escritor VCF 4.0 con folding a 75 octetos respetando UTF-8 multibyte
- Escritor TSV de auditoría con 11 columnas (trazabilidad completa)
- Clasificación automática C2-C6 con 16 categorías N2
- Eliminación E1-E3 (huerfanos, spam, email-only)
- Deduplicación con Union-Find (cierre transitivo TEL + EMAIL + FN)
- Normalización E.164 para teléfonos españoles (+34)
- Normalización de nombres propios (capitalización, títulos, cargos)
- Configuración externa TOML con soporte replace/append
- CI/CD con GitHub Actions (check + fmt + clippy + test + doc)
- Release automatizado con binario + SHA256
- 129 tests unitarios y de integración

[Unreleased]: https://github.com/Soluciones-Alexendros/zedazo/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/Soluciones-Alexendros/zedazo/compare/v0.3.0...v0.5.1
[0.5.0]: https://github.com/Soluciones-Alexendros/zedazo/compare/v0.3.0...v0.5.1
[0.3.0]: https://github.com/Soluciones-Alexendros/zedazo/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Soluciones-Alexendros/zedazo/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/Soluciones-Alexendros/zedazo/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Soluciones-Alexendros/zedazo/releases/tag/v0.1.0
