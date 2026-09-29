---
version: "0.3.2"
date: "2026-09-09"
status: "Activo"
canonical: true
supersedes: "v0.3.1"
---

# DECISIONS.md

**Versión:** 0.3.2  
**Fecha:** 2026-09-09  
**Canónico:** este archivo. [`docs/adr/README.md`](docs/adr/README.md) conserva el texto histórico de ADR-0001…0005 y apunta aquí para IDs nuevos.

---

## Convenciones

- IDs secuenciales **ADR-XXXX**.
- Estados: propuesta | aceptada | sustituida | rechazada | retirada.
- Una decisión aceptada no se reescribe: se sustituye con otra.
- Decisor: Alexendros.

---

<details>
<summary><strong>ADR-0001</strong> — Salida canónica siempre vCard 4.0</summary>

- Estado: aceptada
- Fecha: 2026-07-07
- Contexto: Entradas 3.0 y 4.0; hace falta un formato de salida único.
- Decisión: Salida siempre vCard 4.0 (RFC 6350); `v3_compat` adapta 3.0→4.0.
- Consecuencias: Un solo writer; AGENT/LABEL/MAILER no se propagan.
- Relacionado: [SPECS.md](./SPECS.md) I4, I5.

</details>

<details>
<summary><strong>ADR-0002</strong> — Separación ParsedVCard (infra) / Contact (dominio)</summary>

- Estado: aceptada
- Fecha: 2026-07-07
- Decisión: Dos modelos; `ParsedVCard::into_contact()` traduce.
- Consecuencias: Dominio sin escapes RFC ni PHOTO raw.

</details>

<details>
<summary><strong>ADR-0003</strong> — Union-Find para deduplicación transitiva</summary>

- Estado: aceptada
- Fecha: 2026-07-07
- Decisión: DSU + materialización con índices descendentes.
- Consecuencias: Cierre transitivo D2; `merged_uids` conserva absorbidos.

</details>

<details>
<summary><strong>ADR-0004</strong> — std::sync::LazyLock, no once_cell</summary>

- Estado: aceptada
- Fecha: 2026-07-07
- Decisión: `LazyLock` (MSRV 1.80); no añadir `once_cell`.

</details>

<details>
<summary><strong>ADR-0005</strong> — jiff en lugar de chrono</summary>

- Estado: aceptada
- Fecha: 2026-07-07
- Decisión: Usar `jiff` para timestamps de auditoría/trace.

</details>

<details>
<summary><strong>ADR-0006</strong> — Transferencia del repositorio a Soluciones-Alexendros</summary>

- Estado: aceptada
- Fecha: 2026-07
- Contexto: El repo vivía en la cuenta personal `Alexendros`.
- Decisión: Org `Soluciones-Alexendros`; actualizar Cargo.toml, badges, changelog y Notion.
- Consecuencias: v0.1.1 republish para corregir crates.io/docs.rs; Coveralls bajo la org nueva.

</details>

<details>
<summary><strong>ADR-0007</strong> — Migración Dependabot → Renovate</summary>

- Estado: aceptada
- Fecha: 2026-07-31
- Decisión: Renovate con reglas en `.github/renovate.json`; eliminar `dependabot.yml`.
- Consecuencias: Presets compartidos cross-repo vía PR #17 (pendiente de merge).

</details>

<details>
<summary><strong>ADR-0008</strong> — Runners self-hosted `[self-hosted, ts]`</summary>

- Estado: aceptada (enmienda operativa 2026-08-15)
- Fecha: 2026-07
- Decisión: CI/release Linux preferentemente en runners propios; `publish` a crates.io en `ubuntu-latest`.
- Enmienda 2026-08-15: con **0 runners registrados**, los workflows pasaron a `ubuntu-latest` (#30) para desbloquear CI/release. Restaurar `[self-hosted, ts]` cuando el runner `ts` vuelva a estar online.
- Consecuencias: Sin matrix multi-OS en CI de calidad; release multiplataforma vía cargo-dist (ADR-0012 aceptada).
- Relacionado: [ARCHITECTURE.md](./ARCHITECTURE.md), PR #30.

</details>

<details>
<summary><strong>ADR-0009</strong> — Rechazo temporal de nom 8, toml 1.x y chardetng 1.0</summary>

- Estado: aceptada
- Fecha: 2026-07
- Contexto: PRs Dependabot/Renovate de majors; impacto concentrado en `parser.rs` (nom 8: trait `Parser`).
- Decisión: Congelar con `allowedVersions` en Renovate (`nom<8`, `toml<1`, `chardetng<1`).
- Consecuencias: Upgrade solo con PR dedicado + migración; no mezclar con features.

</details>

<details>
<summary><strong>ADR-0010</strong> — panic = "abort" en release</summary>

- Estado: aceptada
- Fecha: 2026-07 / reafirmada 2026-08-15
- Decisión: Mantener `panic = "abort"` en `[profile.release]` por tamaño/binario.
- Consecuencias: El dominio DEBE ser total (sin `unwrap` de fallo). Fixes C-01/C-02 en v0.1.1.
- Alternativa rechazada: `unwind` (más código, poco beneficio en CLI batch).

</details>

<details>
<summary><strong>ADR-0011</strong> — Cobertura con cargo-llvm-cov + Coveralls</summary>

- Estado: aceptada
- Fecha: 2026-07 / enmienda 2026-09-09
- Decisión: Job `coverage` en CI genera LCOV con `cargo llvm-cov` y sube a Coveralls.
- Consecuencias: Proyecto Coveralls bajo org `Soluciones-Alexendros`; badge restaurado en README; umbral en CI activo (`--fail-under-lines 80`, `--fail-under-regions 75`). Cierra [#25](https://github.com/Soluciones-Alexendros/zedazo/issues/25).

</details>

<details>
<summary><strong>ADR-0012</strong> — Release multiplataforma (cargo-dist)</summary>

- Estado: aceptada
- Fecha: 2026-08-15 / aceptada 2026-09-09
- Contexto: v1.0 promete macOS/Windows; CI de calidad sigue en Linux.
- Decisión: **(B) `cargo-dist`** — [`dist-workspace.toml`](./dist-workspace.toml) + [`.github/workflows/release.yml`](./.github/workflows/release.yml) (targets `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`). SBOM y publish crates.io se mantienen como pasos manuales (`allow-dirty = ["ci"]`).
- Alternativa rechazada: (A) matrix GitHub-hosted pura sin cargo-dist.
- Consecuencias: Artefactos multiplataforma vía cargo-dist; validar releases reales en el hito v1.0.0. Cierra [#27](https://github.com/Soluciones-Alexendros/zedazo/issues/27).

</details>

<details>
<summary><strong>ADR-0013</strong> — Supply chain: cargo-deny + SBOM</summary>

- Estado: aceptada
- Fecha: 2026-08-15 / aceptada 2026-09-09
- Decisión: `cargo deny check` en CI (job Deny) y `make deny`; SBOM CycloneDX en el workflow de release (`cargo cyclonedx`). Config en [`deny.toml`](./deny.toml).
- Relacionado: [ROADMAP.md](./ROADMAP.md); cierra [#26](https://github.com/Soluciones-Alexendros/zedazo/issues/26).

</details>

<details>
<summary><strong>ADR-0014</strong> — Rename producto/crate a Zedazo</summary>

- Estado: aceptada
- Fecha: 2026-08-15 / enmienda 2026-09-09
- Contexto: `vcf-cribador` colisiona semánticamente con el verbo de dominio *cribar*; se busca marca de producto distinta (patrón Atlaps). `cedazo` descartado. Gate crates.io: `zedazo` libre (`ze/da/zedazo` → 404). TMview UE clases 9 y 42: checklist humano pendiente (no bloquea rename).
- Decisión: Renombrar producto/crate/binario a **Zedazo** (`zedazo`) en release **v0.2.0** solo rename+migración. Internos de dominio (`CribaError`, módulo `cribar`, «cribado») sin rename.
- Dominio de producto (cero coste): **`https://zedazo.alexendros.dev`** (wordmark lowercase). DNS/CNAME cuando exista landing.
- Relacionado: issues [#32](https://github.com/Soluciones-Alexendros/zedazo/issues/32), [#35](https://github.com/Soluciones-Alexendros/zedazo/issues/35); [ROADMAP.md](./ROADMAP.md), [CHANGELOG.md](./CHANGELOG.md).

| # | Elemento | Actual | Decisión | Tipo |
|---|---|---|---|---|
| I-01 | Crate + binario | `vcf-cribador` | `zedazo` | Breaking |
| I-02 | Props VCF | `X-CRIBADO-*` | `X-ZEDAZO-*` | Breaking |
| I-03 | Sección TOML | `[cribado]` | `[zedazo]` + alias `[cribado]` con warning deprecación | Suave |
| I-04 | Subcomando CLI | `cribar` | **Mantener** (verbo de dominio) | Sin cambio |
| I-05 | Versión debut | — | **v0.2.0** = solo rename/migración | Estrategia |
| I-06 | Crate antiguo | `vcf-cribador` 0.1.0/0.1.1 en crates.io | Publicar **0.1.2** final con aviso → `zedazo`; **sin yank** | Estrategia |
| I-07 | Metadata org | Ya `Soluciones-Alexendros` | Solo actualizar path a `/zedazo` | Fix menor |
| I-08 | Config file | `cribador.toml` | Doc → `zedazo.toml`; path libre vía `-c` | Docs + convención |
| I-09 | Sufijo salida help | `<input>_cribado.vcf` | `<input>_zedazo.vcf` | Cosmético |
| I-10 | JSON export | `cribado_result` | `zedazo_result` | Breaking suave |
| I-11 | Internos | `CribaError`, módulo `cribar`, dominio «cribado» | **Sin rename** | Explícito |
| I-12 | CSV export | `CRIBADO_RESULT` | `CLASSIFY_RESULT` (inglés, coherente con cabeceras vCard; no marca) | Breaking suave |

- Roadmap en el mismo PR: calidad (ex-v0.2.0) → **v0.3.0**; CardDAV/watch → **v0.4.0**.
- Consecuencias: Breaking en crate name, props VCF y campo JSON; configs `[cribado]` siguen funcionando con deprecación; repo GitHub → `zedazo` (redirects).

</details>

<details>
<summary><strong>ADR-0015</strong> — GUI web self-hosted + API HTTP sobre core compartido</summary>

- Estado: aceptada
- Fecha: 2026-09-08
- Contexto: La CLI es el único punto de entrada. Se necesita paridad funcional desde browser sin duplicar reglas de dominio ni invocar el binario vía `child_process`. SPECS v0.3 listaba GUI como no-objetivo.
- Decisión:
  1. Workspace Rust: `zedazo-core` (domain/application/infra reutilizable), `zedazo-cli` (adaptador Clap), `zedazo-api` (Axum/Tokio).
  2. Frontend separado `apps/web` (Next.js App Router + TypeScript). Sin lógica de cribado/dedup en TS.
  3. Primera release: **single-user local** (`127.0.0.1`), sin cuentas ni colaboración; `ZEDAZO_AUTH_MODE=disabled` solo en loopback.
  4. Persistencia inicial: filesystem + `manifest.json` + `events.ndjson` (`ZEDAZO_STORAGE_MODE=ephemeral`). SQLite solo si el historial lo exige después.
  5. Sin telemetría remota ni CDN de terceros por defecto; OTLP opt-in aplazado post-v1.0 (`ZEDAZO_OTEL_ENABLED=false` reservado/no-op; ADR-0017).
  6. CLI permanece interfaz oficial de automatización; `completions` sin pantalla GUI equivalente.
  7. API versionada bajo `/api/v1`; cambios incompatibles → `/api/v2`.
- Consecuencias: Deps nuevas (axum, tokio, etc.) solo en `zedazo-api`. Actualizar SPECS/ROADMAP/ARCHITECTURE. Hito **v0.5.0** web self-hosted (CardDAV sigue en v0.4.0, PRs separados).
- Relacionado: [SPECS.md](./SPECS.md) O10, [ROADMAP.md](./ROADMAP.md) v0.5.0, [docs/gui/](./docs/gui/), [docs/api/openapi.yaml](./docs/api/openapi.yaml).
- Enmienda: exposición remota single-user → **ADR-0016** (no altera el alcance local de v0.5.0).

</details>

<details>
<summary><strong>ADR-0016</strong> — Exposición remota self-hosted single-user (HTTPS + token)</summary>

- Estado: aceptada
- Fecha: 2026-09-09
- Contexto: La GUI/API V1 (ADR-0015) opera en loopback sin auth. El operador quiere gestiones VCF desde Internet en el mismo dispositivo (luego miniPC), sin SaaS ni multi-usuario.
- Decisión:
  1. **Single-user remoto:** un operador; sin cuentas, colaboración ni multi-tenant.
  2. **`ZEDAZO_AUTH_MODE`:** `disabled` solo si el bind es loopback (**fail-closed** al arrancar si bind no-loopback), salvo `ZEDAZO_AUTH_ALLOW_DISABLED_NON_LOOPBACK=true` para Docker local con puertos host en `127.0.0.1`. `token` exige `ZEDAZO_AUTH_TOKEN` no vacío.
  3. Credencial: `Authorization: Bearer` o cookie HttpOnly `zedazo_auth` (misma secreto) para SSE/`EventSource` con `withCredentials`.
  4. Endpoints públicos: `GET /api/v1/health`, `POST /api/v1/auth/login`, `POST /api/v1/auth/logout`. Resto protegido en modo `token`.
  5. Despliegue: Caddy same-origin (`/` → web, `/api/*` → api); API/web no publicados a Internet; TLS en el proxy. Compose profile `remote`.
  6. CORS: en modo `token`, orígenes vía `ZEDAZO_CORS_ORIGIN` o sin `Any` (mismo origen detrás del proxy).
  7. Acceso sin abrir puertos: Tailscale / Cloudflare Tunnel documentado; miniPC + dominio → Let's Encrypt.
- Consecuencias: hito **v0.5.1**; threat-model y deploy actualizados; no OAuth/OIDC ni equipos.
- Relacionado: ADR-0015, [docs/gui/deploy.md](./docs/gui/deploy.md), [docs/gui/threat-model.md](./docs/gui/threat-model.md).

</details>

<details>
<summary><strong>ADR-0017</strong> — OpenTelemetry aplazado post-v1.0</summary>

- Estado: aceptada (aplazamiento)
- Fecha: 2026-09-09
- Contexto: [#24](https://github.com/Soluciones-Alexendros/zedazo/issues/24) pedía instrumentación OTLP; existe receta en [docs/otel.md](./docs/otel.md) pero no hay feature `otel` ni deps exportables. ADR-0015 exige telemetría remota off por defecto (`ZEDAZO_OTEL_ENABLED=false` en deploy = reservado/no-op).
- Decisión: **No** añadir `opentelemetry*` / `tracing-opentelemetry` hasta **post-v1.0**. Logging con `tracing` permanece. Variables `ZEDAZO_OTEL_*` / `OTEL_*` documentadas como reservadas.
- Consecuencias: Cierra #24 como diferido (no won't-fix); reabrir con ADR de deps cuando toque implementar. Guía en `docs/otel.md` marcada como aplazada.

</details>
