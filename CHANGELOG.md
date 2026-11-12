# Changelog

Todas las modificaciones notables de este proyecto se documentan en este archivo.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [0.6.1](https://github.com/Soluciones-Alexendros/zedazo/compare/v0.6.0...v0.6.1) (2026-10-31)


### Bug Fixes

* **deps:** limpiar alertas altas del e2e de escritorio ([#112](https://github.com/Soluciones-Alexendros/zedazo/issues/112)) ([219906d](https://github.com/Soluciones-Alexendros/zedazo/commit/219906d4dd4d327bba534fa2a5de44ca887f3d9b))

## [0.6.0](https://github.com/Soluciones-Alexendros/zedazo/compare/v0.5.1...v0.6.0) (2026-10-30)


### Features

* CardDAV write, watch y filtros N1/N2 (ADR-0018 / [#48](https://github.com/Soluciones-Alexendros/zedazo/issues/48)) ([47750ec](https://github.com/Soluciones-Alexendros/zedazo/commit/47750ec107386e607974705d4e3ab71ab237d59b))
* CardDAV write, watch y filtros N1/N2 (ADR-0018 / [#48](https://github.com/Soluciones-Alexendros/zedazo/issues/48)) ([#66](https://github.com/Soluciones-Alexendros/zedazo/issues/66)) ([1e5e6f9](https://github.com/Soluciones-Alexendros/zedazo/commit/1e5e6f94833eb9fc49f2543dce7e2e03597f3260))
* **desktop:** AppImage v0.6.0 — ADR-0021, ci-gate, export estático y Tauri ([#91](https://github.com/Soluciones-Alexendros/zedazo/issues/91)) ([0830b5d](https://github.com/Soluciones-Alexendros/zedazo/commit/0830b5da99c7c706c8bb5059c01863c67ae3239d))
* **desktop:** v0.6.0 AppImage local-first ([#104](https://github.com/Soluciones-Alexendros/zedazo/issues/104)) ([42cb4d7](https://github.com/Soluciones-Alexendros/zedazo/commit/42cb4d7fc40f9e55bbdcc00a03e2ac765567ec06))
* primer slice CardDAV pull (ADR-0018 / [#48](https://github.com/Soluciones-Alexendros/zedazo/issues/48)) ([c391102](https://github.com/Soluciones-Alexendros/zedazo/commit/c39110222da9fe26cada36d13a86bf9c48e79e0e))
* primer slice CardDAV pull (ADR-0018 / [#48](https://github.com/Soluciones-Alexendros/zedazo/issues/48)) ([#61](https://github.com/Soluciones-Alexendros/zedazo/issues/61)) ([2f90785](https://github.com/Soluciones-Alexendros/zedazo/commit/2f90785f42af95650ae8db4196373af5552849ec))
* **web:** contrato de tokens v1 y GUI sin API ([#69](https://github.com/Soluciones-Alexendros/zedazo/issues/69)) ([9423901](https://github.com/Soluciones-Alexendros/zedazo/commit/94239012b97e9f40f59179193cb39c3874588f8f))
* **web:** GUI DS fase 2 — átomos alineados a tokens --zed-* ([996daf0](https://github.com/Soluciones-Alexendros/zedazo/commit/996daf09717a893010fb51837fb65a872c3e71b8))
* **web:** GUI DS fase 3 — patrones y pantallas alineados a --zed-* ([7b8bac6](https://github.com/Soluciones-Alexendros/zedazo/commit/7b8bac677e36830c965f686b2f2967c1a4d8e23a))
* **web:** GUI DS fase 4 — catálogo, axe y regresión visual ([ce06caf](https://github.com/Soluciones-Alexendros/zedazo/commit/ce06caf4994d3720a6dbdc0ef69afe356d956923))
* **web:** pipeline DTCG de tokens GUI (fase 1, ADR-0019) ([5c75baf](https://github.com/Soluciones-Alexendros/zedazo/commit/5c75baf85ece56b49356b6d65b3de0165d6a9129))
* **web:** tipografía de flota y morfología neubat ([#70](https://github.com/Soluciones-Alexendros/zedazo/issues/70)) ([5fbfc44](https://github.com/Soluciones-Alexendros/zedazo/commit/5fbfc440263de40fe4c641ad316173e7b003398d))


### Bug Fixes

* anclar hrefs CardDAV al origen y aligerar cargo-deny ([d8ec6aa](https://github.com/Soluciones-Alexendros/zedazo/commit/d8ec6aa31cbc13188d5c249cb9dee4d646e86f90))
* **api:** CORS con cabeceras explícitas y test Prove-It ([95f3ddf](https://github.com/Soluciones-Alexendros/zedazo/commit/95f3ddfa65435cc2f6b0147579f77503f727d2ef))
* **api:** matriz auth remota y tope de jobs concurrentes ([8187505](https://github.com/Soluciones-Alexendros/zedazo/commit/8187505c5e55be64d08a1d9793752701d3e96662))
* **api:** reconciliación, retención y borrado reintentable ([9e8ab46](https://github.com/Soluciones-Alexendros/zedazo/commit/9e8ab46958d0f0f212606dc5b01e9563fc588789))
* **carddav:** cerrar rust/cleartext-logging en test de redacción ([#82](https://github.com/Soluciones-Alexendros/zedazo/issues/82)) ([bed5e12](https://github.com/Soluciones-Alexendros/zedazo/commit/bed5e12fd13aa651f1b5680e157c9867b7b5963a))
* **carddav:** no volcar el debug de credenciales en el assert ([0ec65d1](https://github.com/Soluciones-Alexendros/zedazo/commit/0ec65d1c485b2b715e4cd7ad814a1fa6614236af))
* **ci:** E2E real vía rewrite same-origin y CORS credentials ([9ccd89d](https://github.com/Soluciones-Alexendros/zedazo/commit/9ccd89d1caf5b0ed76225b31072103c7378206a2))
* **ci:** gates de release que exigen binarios y SBOM ([17936df](https://github.com/Soluciones-Alexendros/zedazo/commit/17936df1b1dfd9dcb019be40b1ff0589bdc4ce15))
* **ci:** no saltar publish crates.io tras builds skipped ([3f76b23](https://github.com/Soluciones-Alexendros/zedazo/commit/3f76b23bccc277cccc554f0e9eccae850db7a088))
* **ci:** publish crates.io + recovery workflow ([#55](https://github.com/Soluciones-Alexendros/zedazo/issues/55)) ([1d73c3f](https://github.com/Soluciones-Alexendros/zedazo/commit/1d73c3f7f2ac67cf9b94bf548168a3d3e8aab589))
* **ci:** quinn-proto 0.11.19 y E2E API con NEXT_PUBLIC_API_BASE ([3f432e9](https://github.com/Soluciones-Alexendros/zedazo/commit/3f432e99ea6ae5effca93f03f07b17574b771e96))
* **ci:** rustls 0.23.46 y mocks Playwright same-origin ([e356f34](https://github.com/Soluciones-Alexendros/zedazo/commit/e356f34f1b8e2e45ddfdbe0f6c9a0d13e6a19f2b))
* **deploy:** stack Docker same-origin con Caddy de entrada ([495c81d](https://github.com/Soluciones-Alexendros/zedazo/commit/495c81db9ef8a64f3f8d329889fdf413afd24f9b))
* **deps:** rustls-webpki, tokio y webpki-roots (RUSTSEC-2026) ([eea0e42](https://github.com/Soluciones-Alexendros/zedazo/commit/eea0e423bae61ec57eeb3178b420f86d31a0f4ab))
* **landing:** nombre accesible del enlace a crates.io ([02b9687](https://github.com/Soluciones-Alexendros/zedazo/commit/02b9687724e61790fb1fb2238951de0042f80030))
* rustls 0.23.45 y política de cargo audit (RUSTSEC-2026-0285) ([18193e1](https://github.com/Soluciones-Alexendros/zedazo/commit/18193e113fa91086029e7c0c183c000626b3cbe4))
* rustls 0.23.45 y política de cargo audit (RUSTSEC-2026-0285) ([#67](https://github.com/Soluciones-Alexendros/zedazo/issues/67)) ([e0bf74b](https://github.com/Soluciones-Alexendros/zedazo/commit/e0bf74b4fae5db3c09e6255b0d87186a7258a3e3))
* **web:** cerrar alertas de next, sharp, postcss y transitivas ([8f9e797](https://github.com/Soluciones-Alexendros/zedazo/commit/8f9e797f538ebb50acb869cc401ed107d3119b85))
* **web:** cerrar alertas Dependabot de apps/web ([#83](https://github.com/Soluciones-Alexendros/zedazo/issues/83)) ([d29822e](https://github.com/Soluciones-Alexendros/zedazo/commit/d29822e8d44ef39515d67e7580a18b6819be64d1))
* **web:** copy en español del catálogo mínimo de átomos ([ec40e09](https://github.com/Soluciones-Alexendros/zedazo/commit/ec40e09f617a8171e4b22a0ca73a5be292b1fd8a))
* **web:** estructura válida de MetadataList para axe en /ajustes ([677a925](https://github.com/Soluciones-Alexendros/zedazo/commit/677a92592d5d2a52ebf10cf92bff4c195c0ea7f1))
* **web:** mapear OKLCH al gamut sRGB antes del contraste ([31046b9](https://github.com/Soluciones-Alexendros/zedazo/commit/31046b9370674214869c4c9ab4578f2c80d83c7f))
* **web:** zod 4.6.6 (alertas Dependabot high) ([#85](https://github.com/Soluciones-Alexendros/zedazo/issues/85)) ([684511d](https://github.com/Soluciones-Alexendros/zedazo/commit/684511d4930f215666f507bd57d591d7984e06f1))

## [Unreleased]

### Added
- Cierre operativo: Docker same-origin canónico; retención/reconcile; gates de release (assets+SBOM); Compose smoke; E2E API real; acta [`docs/validation/acta-cierre-operativo.md`](docs/validation/acta-cierre-operativo.md)
- Alineación P1+P2 (canon de flota): meta-sección **Propósito** en docs contractuales; wrappers CI `quality` / `smoke` (el job `test` ya era canónico); `SUPPORT.md` + `ISSUE_TEMPLATE/config.yml`. Jobs Rust maduros intactos (ADR-0020).
- CardDAV v0.4.0 restante ([#48](https://github.com/Soluciones-Alexendros/zedazo/issues/48), ADR-0018): write opt-in (`zedazo carddav put|delete --confirm`, `If-Match` / ETag, HTTP 412 = conflicto), watch (`CTag` / `sync-token`, sin auto-sync al arrancar) y filtros `--category` N1/N2 client-side; tests con servidor DAV mockeado; [`docs/carddav.md`](docs/carddav.md)
- GUI DS fase 4: catálogo `/documentacion/ds` de átomos de producto; axe WCAG 2.2 A/AA en todas las rutas + vacío/error/loading/running; regresión visual Playwright (`toHaveScreenshot`, light/dark, desktop + móvil) en `web-ci`; plan marcado ejecutado ([#59](https://github.com/Soluciones-Alexendros/zedazo/issues/59))
- GUI DS fase 3: patrones de pantalla (shell, formularios, tablas, drawer, jobs, estados) sobre `--zed-*`; axe en rutas de producto; catálogo `/documentacion/ds` con stepper y empty/error/loading ([#59](https://github.com/Soluciones-Alexendros/zedazo/issues/59))
- GUI DS fase 2: átomos (`Button`/`Badge`/`Card`/`Callout`/input) alineados a `--zed-*`; catálogo mínimo `/documentacion/ds`; frontera Web Awesome acotada ([#59](https://github.com/Soluciones-Alexendros/zedazo/issues/59))
- GUI DS fase 1: pipeline DTCG → CSS custom properties OKLCH + tipos TS; check de contraste WCAG 2.2 AA en `web-ci` ([ADR-0019](DECISIONS.md), [#59](https://github.com/Soluciones-Alexendros/zedazo/issues/59))
- Plan de modernización del design system GUI (OKLCH / DTCG → GUI profesional) en [`docs/gui/design-system-plan.md`](docs/gui/design-system-plan.md) ([#59](https://github.com/Soluciones-Alexendros/zedazo/issues/59)); sin cambios de UI en este ítem
- Primer slice CardDAV (ADR-0018 / [#48](https://github.com/Soluciones-Alexendros/zedazo/issues/48), no cierra el issue): crate `zedazo-carddav` (`publish = false`), CLI `zedazo carddav list|pull`, Basic + app-password (`ZEDAZO_CARDDAV_*`), descubrimiento RFC 6764 y perfil Nextcloud; guía [`docs/carddav.md`](docs/carddav.md)

### Security
- `rustls` → 0.23.46 (RUSTSEC-2026-0348 / RUSTSEC-2026-0349); `quinn-proto` → 0.11.19 (RUSTSEC-2026-0338 / 0340 / 0343)
- `rustls` 0.23.44 → 0.23.45 ([RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285): handshake TLS 1.3 aceptado en el encryption level incorrecto)
- Política `cargo audit`: el workflow falla solo ante vulnerabilidades; avisos informativos no bloquean; excepciones en `.cargo/audit.toml` + `deny.toml`
- CardDAV: los hrefs DAV absolutos quedan anclados al origen del servidor; no se reenvía Basic a loopback u otro host inyectado en el 207
- **ADR-0018** (aceptada): red/proveedor CardDAV como precondición de v0.4.0 / [#48](https://github.com/Soluciones-Alexendros/zedazo/issues/48) — cliente RFC 6352, una cuenta, crate dedicado; sin código de sync en este cambio
- Convención de marca y pesquisa TMview UE clases 9/42 en [`docs/brand.md`](docs/brand.md) (#49)
- Landing pública estática [`apps/landing/`](apps/landing/index.html) para `https://zedazo.alexendros.dev` (#50)
- Receta Caddy/Compose [`deploy/Caddyfile.landing`](deploy/Caddyfile.landing) + [`deploy/docker-compose.landing.yml`](deploy/docker-compose.landing.yml)
- Guía DNS CNAME `zedazo` → `<HOST_DESTINO>` (operador) y same-origin futuro en [`docs/gui/deploy.md`](docs/gui/deploy.md)

### Fixed
- Release: matriz de cargo-dist en output compacto (evita filtrado de `GITHUB_OUTPUT` que dejó `v0.5.1` sin binarios); `host` exige builds locales + SBOM en success; validación de assets/checksums antes de `gh release create`; espera acotada del índice crates.io ([docs/validation/release-v0.5.1.md](docs/validation/release-v0.5.1.md))
- Release: job `Publish to crates.io` con `if: always() && needs.host.result == 'success'` (evita skip cuando los builds de cargo-dist se saltan); workflow manual `publish-crates.yml`

### Changed
- GUI: `theme-color` y favicon usan hex **generado** desde tokens OKLCH (canvas / accent); los hex a mano de `layout.tsx` dejan de ser origen (ADR-0019)
- Wordmark lowercase **`zedazo`** en landing, GUI chrome y README (ADR-0014 / #49)
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
