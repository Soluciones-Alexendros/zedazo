---
version: "0.3.2"
date: "2026-09-09"
status: "Activo"
canonical: true
supersedes: "v0.3.1"
---

# Zedazo — Arquitectura

### Propósito de este documento

- **Objetivos:** Describir capas Clean/DDD, CI/CD y límites de crates de Zedazo.
- **Estructura:** Propósito → principios → CI/CD → capas → workspace → invariantes.
- **Contenido a integrar según contexto:** Conserva la arquitectura Rust/GUI de este repo. No copies un diagrama de landing/SaaS. Los jobs canónicos `quality` / `test` / `smoke` son wrappers; no reescribas `fmt`, `clippy`, `check`, `parity` ni `web`.

**Versión:** 0.3.2
**Fecha:** 2026-09-09
**Canónico:** este archivo. [`docs/architecture.md`](docs/architecture.md) redirige aquí.

---

## Principios

- **Clean Architecture:** dependencias apuntan hacia el centro. `domain` no depende de nada externo. `application` orquesta `domain`. `infrastructure` implementa adaptadores.
- **DDD:** bounded contexts independientes con lenguaje ubicuo compartido.
- **Spec-Driven:** criterios de aceptación e invariantes guían el desarrollo y los tests.
- **Event Storming:** comandos, eventos y excepciones documentan el flujo.

---

## CI/CD

El proyecto usa **GitHub Actions** en **`ubuntu-latest`**. ADR-0008 prefiere runners `[self-hosted, ts]`; enmienda 2026-08-15: con 0 runners registrados, los workflows usan `ubuntu-latest` hasta restaurar el runner `ts`.

| Workflow          | Trigger                    | Jobs                                     |
| ----------------- | -------------------------- | ---------------------------------------- |
| `ci.yml`          | push/PR a `main`           | Jobs maduros (Check, MSRV, Format, Clippy, Test, Doc, Coverage, Docs validate, Parity O10, Web, Deny) + wrappers canónicos `quality` (fmt+clippy+docs-validate) y `smoke` (health+check). `test` ya es el nombre canónico. |
| `audit.yml`       | Lunes 08:00 UTC; PR/push a lockfile/política | cargo audit (falla solo en vulns RustSec) |
| `release.yml`     | Tag `v*`                   | Build + Package + Publish to crates.io   |
| Renovate          | Schedule + PRs             | `.github/renovate.json` (no Dependabot)  |

**Branch protection:** `main` requiere:
- Todos los checks CI verdes
- 1 approving review + code owner
- Linear history (no merge commits)
- Sin pushes directos

→ Ver [ROADMAP.md](./ROADMAP.md) y [DECISIONS.md](./DECISIONS.md) (ADR-0007 Renovate, ADR-0008 runners, ADR-0011 coverage).

---

## Capas

```
┌─────────────────────────────────────────────────────────────┐
│  interfaces: zedazo-cli (Clap)  │  zedazo-api (Axum /api/v1) │
│  apps/web (Next.js) → solo HTTP; sin lógica de dominio       │
├─────────────────────────────────────────────────────────────┤
│           application/ (zedazo-core)                         │
│           Cribar, Audit, Stats, Export + ProcessRequest      │
├─────────────────────────────────────────────────────────────┤
│  domain/ (puro)          │  infrastructure/ (I/O reutilizable)│
└─────────────────────────────────────────────────────────────┘

Regla de dependencia:
  domain ← application ← interfaces (cli | http)
  domain ← infrastructure
  zedazo-core NO depende de Axum, Tokio HTTP, cookies ni DB
```

Workspace (ADR-0015): `crates/zedazo-core`, `crates/zedazo-cli`, `crates/zedazo-api`, `crates/zedazo-carddav`, `apps/web`.
CardDAV (v0.4.0, **ADR-0018** aceptada): crate `zedazo-carddav` (`publish = false`), cliente HTTP consumido por la CLI (`zedazo carddav list|pull|put|delete|watch`); **no** HTTP CardDAV en `zedazo-core` ni endpoints en `zedazo-api`/GUI. Guía: [`docs/carddav.md`](docs/carddav.md). Código de red en PRs distintos de dominio/UI.
Ficha pública (ADR-0014 / #50): `apps/landing/` en `zedazo.alexendros.dev`; DNS y Caddy en [`docs/gui/deploy.md`](docs/gui/deploy.md).
Jobs web: directorio aislado por ULID bajo `$ZEDAZO_DATA_DIR` con `manifest.json` y `events.ndjson`.
Tokens GUI (ADR-0019): fuente DTCG [`apps/web/tokens/`](apps/web/tokens/) → `pnpm tokens:build` → `--zed-*` OKLCH + tipos TS; contraste WCAG 2.2 AA en `web-ci`. El contrato v1 ([`apps/web/tokens/CONTRACT.md`](apps/web/tokens/CONTRACT.md)) añade alias `bg` / `text` / `border` / `action` / `feedback` hacia esa paleta. `apps/landing/tokens.css` sale del mismo build.

### Auth y despliegue (ADR-0016)

- Middleware `require_auth` en `/api/v1/*` salvo `GET /health`, `POST /auth/login`, `POST /auth/logout`
- Modos: `disabled` (solo loopback, fail-closed; Docker local puede usar `ZEDAZO_AUTH_ALLOW_DISABLED_NON_LOOPBACK`) | `token` (Bearer + cookie `zedazo_auth` para SSE)
- Remoto: [`deploy/docker-compose.remote.yml`](deploy/docker-compose.remote.yml) + Caddy same-origin; guía en [`docs/gui/deploy.md`](docs/gui/deploy.md)

→ Ver [ROADMAP.md](./ROADMAP.md) v0.5.0–v0.5.1 y [DECISIONS.md](./DECISIONS.md) (ADR-0015, ADR-0016).

## Capas (histórico v0.2 — CLI única)

```
┌──────────────────────────────────────┐
│           interfaces/cli.rs           │  ← Clap (punto de entrada CLI)
├──────────────────────────────────────┤
│           application/                │  ← Casos de uso (Cribar, Audit, Stats, Export)
├──────────────────────────────────────┤
│  ┌────────────┐  ┌────────────────┐  │
│  │  domain/   │  │ infrastructure/ │  │
│  │            │  │                 │  │
│  │ Contact    │  │ parser.rs       │  │
│  │ Decision-  │  │ writer.rs       │  │
│  │ Trace      │  │ encoding.rs     │  │
│  │ classify() │  │ v3_compat.rs    │  │
│  │ dedup()    │  │ source.rs       │  │
│  │ decide()   │  │ config.rs       │  │
│  │ rules      │  │ csv/json/tsv    │  │
│  └────────────┘  └────────────────┘  │
└──────────────────────────────────────┘

Regla de dependencia:
  domain ← application ← interfaces
  domain ← infrastructure
  application → infrastructure (vía traits)
```

---

## Bounded Contexts

### 1. IngestionContext

**Responsabilidad:** Leer bytes, detectar codificación, transcodificar a UTF-8, desplegar líneas, parsear vCard 3.0/4.0 → `ParsedVCard`, detectar fuente y versión.

**Módulos:**
- `infrastructure/encoding.rs` — chardetng + encoding_rs
- `infrastructure/parser.rs` — unfold, unescape, nom parsers
- `infrastructure/v3_compat.rs` — adaptación vCard 3.0 → 4.0
- `infrastructure/source.rs` — detección de fuente (PRODID, UID)

**Entrada:** `Vec<u8>`
**Salida:** `Vec<ParsedVCard>`

### 2. QualityContext

**Responsabilidad:** Cribado (E1-E6, C1-C7) con precedencia determinista.

**Módulos:**
- `domain/screening.rs` — `decide()`, `DecisionTrace`, `ScreeningDecision`
- `domain/contact.rs` — `Contact`, `StructuredName`, `Tel`

**Entrada:** `Vec<ParsedVCard>`
**Salida:** `Vec<Contact>` con `decision` (sin normalizar)

### 3. NormalizationContext

**Responsabilidad:** Normalización de campos (N1-N7 para FN, T1-T4 para TEL, normalización de ORG, ADR).

**Módulos:**
- `domain/normalization.rs` — `normalize_fn()`, `normalize_tel()`, `normalize_org()`
- `application/cribar.rs` — llamada a funciones de normalización

**Entrada:** `Vec<Contact>` (tras cribado, solo activos)
**Salida:** `Vec<Contact>` con campos normalizados (FN, TEL, ORG, title, role)

### 4. TaxonomyContext

**Responsabilidad:** Asignar categorías N1/N2/N3 mediante reglas regex. Cargar reglas desde TOML (herencia: añadir o reemplazar).

**Módulos:**
- `domain/rules.rs` — `ClassificationRule`, reglas estándar
- `domain/classification.rs` — `classify()`, `CategorySet`
- `infrastructure/config.rs` — carga de `zedazo.toml` ([zedazo]; alias [cribado] deprecado)

**Entrada:** `Vec<Contact>` + `Option<Config>`
**Salida:** `Vec<Contact>` con `categories` poblado

### 5. IdentityContext

**Responsabilidad:** Detectar duplicados D1-D2 con Union-Find (cierre transitivo), fusionar contactos, registrar propuestas D3-D6.

**Módulos:**
- `domain/identity.rs` — `deduplicate()`, `DuplicateCluster`, `merge_contacts()`

**Entrada:** `Vec<Contact>`
**Salida:** `Vec<Contact>` fusionado + `merged_uids` + propuestas en NOTE

### 6. OutputContext

**Responsabilidad:** Serializar a VCF 4.0, CSV, JSON, TSV. Folding 75 octetos. Preservación de binarios.

**Módulos:**
- `infrastructure/writer.rs` — `write_vcf()`, `fold_line()`
- `infrastructure/csv_writer.rs` — export CSV
- `infrastructure/json_writer.rs` — export JSON
- `infrastructure/tsv_writer.rs` — `write_audit_tsv()`

**Entrada:** `Vec<Contact>` + `Vec<AuditEntry>`
**Salida:** archivos en disco (VCF, CSV, JSON, TSV)

---

## Modelos y su tránsito

```
┌─────────────┐     into_contact()     ┌──────────┐
│ ParsedVCard │ ──────────────────────→ │ Contact  │
│ (infra)     │                        │ (domain) │
└─────────────┘                        └──────────┘
                                               │
                     ┌─────────────────────────┤
                     │                         │
                decide()                  classify()
                     │                         │
                     ▼                         ▼
             ScreeningDecision            CategorySet
                     │                         │
                     └─────────┬───────────────┘
                               │
                        normalize_fn/tel/org()
                               │
                               ▼
                     Vec<Contact> (normalizado)
                               │
                          deduplicate()
                               │
                               ▼
                       Vec<Contact> (final)
                               │
               ┌───────────────┼───────────────┐
               │               │               │
          write_vcf()    write_csv()    write_json()
               │               │               │
               ▼               ▼               ▼
          .vcf (4.0)      .csv            .json

  En paralelo: AuditEntry → write_tsv() → .tsv
```

---

## Árbol de dependencias entre módulos

```
main.rs
  → cli.rs
  → tracing_subscriber

cli.rs
  → application::cribar
  → application::audit
  → application::stats
  → application::export

application::cribar
  → infrastructure::encoding
  → infrastructure::parser
  → infrastructure::v3_compat
  → infrastructure::source
  → infrastructure::config
  → domain::screening
  → domain::contact
  → domain::normalization
  → domain::classification
  → domain::identity
  → infrastructure::writer
  → infrastructure::tsv_writer

domain::screening → domain::contact, domain::rules
domain::normalization → domain::contact
domain::classification → domain::rules
domain::identity → domain::contact
domain::contact → (sin dependencias)
domain::rules → regex (crate externa, sin lógica de negocio)

infrastructure::parser → nom (crate externa)
infrastructure::encoding → chardetng, encoding_rs
infrastructure::config → toml, domain::rules
```

---

## Traits de infraestructura (dependency inversion)

```rust
// domain/screening.rs — el dominio no conoce la infraestructura
pub trait ScreeningConfigProvider {
    fn conservar_dominios(&self) -> &[String];
    fn e2_keywords(&self) -> &[String];
}
```

La implementación concreta vive en `infrastructure/config.rs` y se inyecta en `application/cribar.rs`.
