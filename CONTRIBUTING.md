# Contribuir a Zedazo

### Propósito de este documento

- **Objetivos:** Flujo de fork/PR, convenciones Rust y checks locales (`make ci`).
- **Estructura:** Propósito → flujo → convenciones → estructura → antes de enviar → tokens GUI → bugs.
- **Contenido a integrar según contexto:** Conserva hooks, dual license y fixtures sintéticos. CI canónico: `quality` ≡ fmt+clippy+docs-validate; `test` ≡ `cargo test --workspace --all-features`; `smoke` ≡ health+check. No reescribas jobs maduros.

¡Gracias por tu interés en contribuir!

## Flujo de trabajo

1. **Fork** del repositorio
2. Crea una rama: `git checkout -b feat/mi-feature`
3. Haz tus cambios siguiendo las convenciones del proyecto
4. Asegúrate de que pasan los checks: `make ci`
5. Commit con mensaje descriptivo
6. Push y abre un Pull Request

## Convenciones de código

- **Rust 2021 edition**, MSRV 1.80
- `cargo fmt` obligatorio (hook pre-commit incluido)
- `cargo clippy -- -D warnings` sin errores
- Tests unitarios para cada módulo de dominio y aplicación
- Tests de integración con fixtures sintéticos (nunca PII real)
- Español en commits, PRs y mensajes al humano ([AGENTS.md](AGENTS.md))

## Estructura del proyecto

```
crates/zedazo-core/   # domain + application + infrastructure (sin HTTP)
crates/zedazo-cli/    # binario zedazo (Clap)
crates/zedazo-api/    # API HTTP Axum + auth (ADR-0016)
crates/zedazo-carddav/# cliente CardDAV pull (ADR-0018; publish = false)
apps/web/             # GUI Next.js (solo cliente HTTP)
apps/landing/         # ficha pública estática (zedazo.alexendros.dev)
deploy/               # Docker Compose local / remoto / landing + Caddy
docs/gui/             # Paridad, deploy (DNS), threat-model, design-system-plan
tests fixtures:       crates/zedazo-core/tests/fixtures/
```

Ver [`ARCHITECTURE.md`](ARCHITECTURE.md) para la arquitectura completa.

## Antes de enviar un PR

```bash
make hooks       # instalar pre-commit (fmt + clippy); ver `.githooks/install.sh`
make ci          # fmt + clippy + test + check + doc + docs-validate + parity + web-ci
make deny        # opcional: cargo-deny (licencias/advisories)
make audit       # opcional: cargo-audit (falla solo en vulnerabilidades RustSec)
```

Los hooks viven en [`.githooks/`](.githooks/) y se activan con `make hooks`. Documentos canónicos: [AGENTS.md](AGENTS.md), [SPECS.md](SPECS.md), [ROADMAP.md](ROADMAP.md), [DECISIONS.md](DECISIONS.md).

## Tokens de la GUI (ADR-0019)

Fuente DTCG en [`apps/web/tokens/`](apps/web/tokens/). No editar `apps/web/src/design-system/generated/` ni `apps/web/src/lib/design-tokens.ts`.

```bash
pnpm --dir apps/web tokens:build      # regenera CSS + tipos
pnpm --dir apps/web tokens:check      # drift vs artefactos commiteados
pnpm --dir apps/web tokens:contrast   # WCAG 2.2 AA (pares semánticos)
```

Plan: [`docs/gui/design-system-plan.md`](docs/gui/design-system-plan.md) (ejecutado 2026-09-12). Wordmark: [`docs/brand.md`](docs/brand.md) (`zedazo` en minúsculas). Catálogo in-app: `/documentacion/ds`.

Regresión visual (Playwright `toHaveScreenshot`, en `web-ci`):

```bash
pnpm --dir apps/web test:visual -- --update-snapshots
# Capturas README (opt-in; no corre en web-ci):
ZEDAZO_SCREENSHOTS=1 pnpm --dir apps/web test:e2e -- e2e/screenshots.spec.ts
```

## Reportar bugs

Usa la plantilla de [bug report](.github/ISSUE_TEMPLATE/bug_report.md). Incluye:
- Comando exacto ejecutado (o URL/pantalla GUI)
- Archivo VCF de ejemplo (anonimizado si contiene datos reales)
- Salida esperada vs obtenida
- Versión de Zedazo (`zedazo --help` / health API)
