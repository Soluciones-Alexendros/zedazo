---
version: "0.3.2"
date: "2026-09-09"
status: "Activo"
canonical: true
supersedes: "v0.3.1"
---

# DECISIONS.md

### Propósito de este documento

- **Objetivos:** Registro de ADRs con IDs estables (fuente de verdad; no se mueve a `docs/architecture/decisions/`).
- **Estructura:** Propósito → convenciones → ADR-0001… en `<details>`.
- **Contenido a integrar según contexto:** Una decisión aceptada no se reescribe: se sustituye. No copies ADRs de otro repo. Confirmación humana antes de deps nuevas, majors, runners o `panic`/`profile`.

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
- Fecha: 2026-08-15 / enmienda 2026-09-09 / wordmark+TMview 2026-09-11
- Contexto: `vcf-cribador` colisiona semánticamente con el verbo de dominio *cribar*; se busca marca de producto distinta (patrón Atlaps). `cedazo` descartado. Gate crates.io: `zedazo` libre (`ze/da/zedazo` → 404).
- Decisión: Renombrar producto/crate/binario a **Zedazo** (`zedazo`) en release **v0.2.0** solo rename+migración. Internos de dominio (`CribaError`, módulo `cribar`, «cribado») sin rename.
- Wordmark: **`zedazo` en minúsculas** en lockups, pestaña, landing y H1 de README (patrón Atlaps). Prosa: «Zedazo». Identificadores: `zedazo` / `ZEDAZO_*` / `X-ZEDAZO-*`. Convención: [`docs/brand.md`](./docs/brand.md).
- Dominio de producto (cero coste): **`https://zedazo.alexendros.dev`**. DNS/CNAME: operativa #50.
- Operativa (#50): landing estática [`apps/landing/`](./apps/landing/); CNAME `zedazo` → `<HOST_DESTINO>` (operador) y Caddy/Let's Encrypt en [`docs/gui/deploy.md`](./docs/gui/deploy.md). GUI remota en el mismo host → same-origin (ADR-0016), no este Caddyfile de landing.
- TMview UE clases 9 y 42 (#49): pesquisa documental 2026-09-11 en [`docs/brand.md`](./docs/brand.md) — sin coincidencia exacta «Zedazo» en índices públicos; TMview oficial y valoración de similitud (p. ej. EUTM **ZEZARO** 009317348) quedan en checklist humano. No bloquea el rename ni el wordmark.
- Relacionado: issues [#32](https://github.com/Soluciones-Alexendros/zedazo/issues/32), [#35](https://github.com/Soluciones-Alexendros/zedazo/issues/35), [#49](https://github.com/Soluciones-Alexendros/zedazo/issues/49), [#50](https://github.com/Soluciones-Alexendros/zedazo/issues/50); [ROADMAP.md](./ROADMAP.md), [CHANGELOG.md](./CHANGELOG.md).

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

<details>
<summary><strong>ADR-0018</strong> — Red/proveedor CardDAV (precondición v0.4.0)</summary>

- Estado: **aceptada**
- Fecha: 2026-09-11
- Contexto: [#48](https://github.com/Soluciones-Alexendros/zedazo/issues/48) / [ROADMAP.md](./ROADMAP.md) **v0.4.0** exigen un ADR de red/proveedor **antes** de implementar sync. SPECS §3 deja CardDAV, Proton API y Google People como no-objetivo hasta ese hito. El producto ya parsea vCard 3.0/4.0 de **export** Proton / Google / Apple (O1) y opera **single-user self-hosted** (ADR-0015 local, ADR-0016 remoto HTTPS+token). La landing `zedazo.alexendros.dev` (ADR-0014) no es proxy de contactos. El pipeline local sigue **sin red saliente por defecto**.
- Hallazgo de proveedores (2026-09):
  1. **RFC 6352 genérico** (descubrimiento RFC 6764 `/.well-known/carddav`, `addressbook-home-set`, sync RFC 6578): cubre Nextcloud/SabreDAV, Fastmail y hosts DAV ordinarios. Auth típica: HTTP Basic + **contraseña de aplicación**.
  2. **Apple iCloud:** CardDAV nativo (`https://contacts.icloud.com` → shard `pNN-contacts.icloud.com`); Basic + app-specific password; vCard 3.0. Encaja con O1.
  3. **Proton Contacts:** **no** expone CardDAV/CalDAV nativo (Bridge oficial es correo, no contactos; E2E). El flujo vigente sigue siendo export/import VCF 4.0. Un puente local no oficial (p. ej. hydroxide en loopback) podría hablar RFC genérico; Zedazo **no** lo empaqueta ni lo soporta.
  4. **Google Contacts CardDAV:** existe (`https://www.googleapis.com/.well-known/carddav`) pero **solo OAuth 2.0** + registro de cliente en Google Cloud; vCard 3.0. People API es **otro** protocolo (SPECS §3 lo lista aparte).
- Decisión:
  1. **Una cuenta / un principal** por configuración. Sin multi-cuenta simultánea ni perfiles de operador. Cambiar de servidor = cambiar config, no N sesiones. Encaja con ADR-0015/0016; no hay evidencia que fuerce multi-cuenta en v0.4.0.
  2. **Read y write** sobre CardDAV. La lectura (pull) es el default. La escritura (PUT/DELETE) es **opt-in explícito** por invocación: I7 prohíbe sobrescribir el VCF local; el remoto no se muta en `cribar` salvo comando de sync con confirmación. Sin last-write-wins silencioso: `If-Match` / ETag; HTTP 412 → conflicto reportado, no overwrite.
  3. **Cliente RFC primero**, no SDK de marca. Perfiles de descubrimiento documentados: URL explícita, Nextcloud (`/remote.php/dav/`), iCloud (`contacts.icloud.com`). Fastmail u otros hosts RFC = misma impl.
  4. **Auth v0.4.0:** HTTP Basic (usuario + app password / token de aplicación del *proveedor*). Credenciales en env/`zedazo.toml` local (`ZEDAZO_CARDDAV_*`); **nunca** reutilizar `ZEDAZO_AUTH_TOKEN` (eso es acceso a la GUI, ADR-0016). No loguear secretos. Egreso solo desde la máquina del operador (CLI) o, más tarde, desde el host self-hosted; **no** vía `zedazo.alexendros.dev`.
  5. **OAuth/OIDC de proveedor (Google) y Google People API:** fuera del primer slice. SPECS §3 rechaza cuentas OAuth de *producto* Zedazo; un cliente OAuth Google implicaría `client_id`/refresh tokens y un ADR o slice dedicado. **Hunch:** no conviene mezclarlo con el cliente RFC+Basic.
  6. **Proton:** sigue por fichero VCF. CardDAV a Proton solo si el operador apunta el cliente genérico a un puente *suyo*. No hay proveedor «Proton» de primera clase hasta que Proton publique DAV.
  7. **TLS:** HTTPS obligatorio (TLS 1.2+); sin `insecure-skip-verify` por defecto. CA privada (Nextcloud self-signed) solo con flag/config explícita en un PR posterior. HTTP claro solo hacia loopback (puente local).
  8. **Límites:** un request en vuelo por colección de forma conservadora; honrar `Retry-After`; backoff exponencial; sin sync automático al arrancar GUI/CLI.
  9. **Crate:** `zedazo-carddav` (`publish = false`), adaptador de infraestructura. Lo consume `zedazo-cli` en el primer slice. **`zedazo-core` sin HTTP** (ADR-0015: ni Axum inbound ni cliente CardDAV). `zedazo-api` / GUI **no** ganan endpoints CardDAV en el mismo PR que el cliente. Deps HTTP concretas (`reqwest`/equivalente, rustls) + `deny.toml` → PR de implementación (confirmación de dep nueva, AGENTS §4).
  10. **Watch mode y filtros por categoría** (ROADMAP v0.4.0): **fuera del primer slice**. Watch = más adelante, acotado a polling de `CTag`/`sync-token` (y/o watch de ficheros locales) en PRs propios. Filtros = aplicar taxonomía N1/N2 ya existente sobre el set sincronizado; `addressbook-query` RFC 6352 no es requisito del primer slice.
- PRs: todo código CardDAV **separado** de cambios de dominio o UI (AGENTS / #48). No mezclar con features GUI en la misma unidad.
- Alternativas rechazadas (o diferidas):
  - Meter HTTP CardDAV en `zedazo-core`: viola «core sin HTTP».
  - Multi-cuenta / OAuth de producto: contradice single-user ADR-0015/0016.
  - Tratar Proton o Google People como primer proveedor CardDAV: Proton no tiene DAV; People no es CardDAV.
  - Push automático tras `cribar`: viola I7 y el principio de red saliente opt-in.
- Consecuencias: desbloquea implementación de #48 (PRs de código posteriores; este ADR no incluye sync). SPECS §3 / ROADMAP v0.4.0 apuntan aquí. Primer slice verificable: pull RFC + Basic contra fixture/servidor de prueba (Nextcloud o DAV genérico), sin GUI. OTel sigue post-v1.0 (ADR-0017).
- Relacionado: [SPECS.md](./SPECS.md) §3, [ROADMAP.md](./ROADMAP.md) v0.4.0, [ARCHITECTURE.md](./ARCHITECTURE.md), ADR-0014, ADR-0015, ADR-0016, [#48](https://github.com/Soluciones-Alexendros/zedazo/issues/48).
- **Hunches** (etiquetados; no bloquean el ADR si se enmiendan):
  - Nextcloud + iCloud cubren al operador self-hosted y a O1 Apple mejor que perseguir Proton DAV.
  - Un subcomando CLI (`zedazo carddav …`) antes que pantalla GUI.
  - `addressbook-query` y filtros server-side no hacen falta para el MVP de sync.

</details>

<details>
<summary><strong>ADR-0019</strong> — Fuente DTCG de tokens GUI (script Node, sin Style Dictionary)</summary>

- Estado: **aceptada**
- Fecha: 2026-09-11
- Contexto: La GUI (ADR-0015/0016) ya usa custom properties `--zed-*` en OKLCH, escritas a mano. El [plan de design system](./docs/gui/design-system-plan.md) (epic [#59](https://github.com/Soluciones-Alexendros/zedazo/issues/59)) exige un origen DTCG, CSS/TS generados y contraste WCAG 2.2 AA en CI. Style Dictionary v4 + `@tokens-studio/sd-transforms` era la hipótesis; AGENTS §4 pide confirmación para deps nuevas.
- Decisión:
  1. **Fuente DTCG** en [`apps/web/tokens/`](./apps/web/tokens/) (primitivo → semántico; componente = stub en fase 1). Color de origen: objeto `{ colorSpace: "oklch", components: [L, C, H] }`. Sin hex/rgb/hsl en la fuente.
  2. **Build propio en Node 22** ([`apps/web/scripts/design-tokens/`](./apps/web/scripts/design-tokens/)): passthrough OKLCH → `--zed-*` + `src/lib/design-tokens.ts`. **Sin** `style-dictionary` ni otras deps npm. Revisitar SD solo si aparece sync Figma/Tokens Studio o el volumen de transforms lo justifica.
  3. **Artefactos commiteados** (`src/design-system/generated/`, `src/lib/design-tokens.ts`) + `pnpm tokens:check` en `web-ci` / job Web. Política única: no generate-on-CI sin el check de drift.
  4. **Hex solo generado** (`themeColorHex`, `faviconHex`) para `theme-color` del viewport y el favicon (Satori no pinta OKLCH). Los hex huérfanos de `layout.tsx` / `icon.tsx` dejan de ser origen.
  5. **Contraste:** script `pnpm tokens:contrast` convierte OKLCH → sRGB lineal y aplica ratio WCAG 2.2 AA sobre pares semánticos light y dark. APCA informativo queda fuera (fase 4 / catálogo).
  6. **Web Awesome:** se conserva la dependencia; el bridge `--wa-*` se genera desde semánticos Zedazo. La decisión A/B (kit opcional vs retirar) se aplaza a fase 2. Este PR no monta componentes `<wa-*>`.
  7. Nombres públicos `--zed-*` **estables** (sin rename breaking). Tokens aditivos de fase 1: `--zed-bp-md/lg`, `--zed-target-min`, `--zed-z-shell/drawer`, `--zed-badge-*-border`.
- Alternativas rechazadas (o diferidas):
  - Style Dictionary v4 + sd-transforms: dos deps nuevas, riesgo de convertir OKLCH a sRGB, y el set actual cabe en un script de decenas de líneas.
  - Generate-on-CI sin commitear: peor revisión de diffs visuales y CI más opaco.
  - Retirar Web Awesome ahora: es decisión de inventario (fase 2), no de pipeline.
- Consecuencias: `make web-ci` incluye check de artefactos + contraste. `apps/landing/` y `zedazo-api` no consumen estos tokens. PRs de CardDAV (#48) no tocan `tokens/` ni `design-system/`.
- Relacionado: ADR-0015, ADR-0016, [SPECS.md](./SPECS.md) O10, [ROADMAP.md](./ROADMAP.md) v0.5.x, [docs/gui/design-system-plan.md](./docs/gui/design-system-plan.md), [#59](https://github.com/Soluciones-Alexendros/zedazo/issues/59).

</details>

<details>
<summary><strong>ADR-0020</strong> — Alineación P1+P2 (canon de flota) sin aplastar CI Rust</summary>

- Estado: **aceptada**
- Fecha: 2026-09-24
- Contexto: Oleada de alineación a `Soluciones-Alexendros/repo-standard` (main). Este repo ya es gold en governance (dual license, CoC, Renovate, SPECS/ARCHITECTURE/AGENTS, jobs Rust maduros). El canon pide jobs `quality` / `test` / `smoke` y meta-sección **Propósito**. Reescribir `fmt`/`clippy`/`test`/`check`/`parity`/`web` rompería CI.
- Decisión:
  1. Añadir wrappers `quality` (fmt+clippy+docs-validate) y `smoke` (health+check). El job `test` ya es canónico.
  2. Documentar la equivalencia en [`AGENTS.md`](./AGENTS.md). No mover ADRs fuera de este archivo.
  3. Conservar dual license MIT OR Apache-2.0, CoC y Renovate.
- Consecuencias: `make ci` no cambia. Los wrappers solo agregan estado. Sin force-push ni org settings.
- Relacionado: [AGENTS.md](./AGENTS.md), [ARCHITECTURE.md](./ARCHITECTURE.md), [SPECS.md](./SPECS.md), [ROADMAP.md](./ROADMAP.md).

</details>
