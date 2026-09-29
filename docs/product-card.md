# Zedazo — Ficha de producto

---

## Portada / Metadata

| Campo | Valor |
|-------|-------|
| **Nombre** | **zedazo** (wordmark lowercase; prosa: Zedazo). Convención: [brand.md](./brand.md) |
| **Versión publicada** | Workspace **0.5.1**; crates.io tras tag humano `v0.5.1` |
| **Código `main`** | Hitos **v0.5.0** (GUI) + **v0.5.1** (remoto) |
| **Estado** | Publicado (CLI) + GUI/API self-hosted en código |
| **Licencia** | MIT OR Apache-2.0 (dual) |
| **MSRV** | Rust 1.80+ |
| **Lenguaje** | Rust (edition 2021) + TypeScript (Next.js) |
| **Tipo** | CLI + API HTTP + GUI web self-hosted |
| **Repositorio** | https://github.com/Soluciones-Alexendros/zedazo |
| **Dominio producto** | https://zedazo.alexendros.dev (cero coste; ADR-0014; landing `apps/landing/`; DNS en [deploy.md](./gui/deploy.md); marca: [brand.md](./brand.md)) |
| **crates.io** | https://crates.io/crates/zedazo |
| **Documentación** | https://docs.rs/zedazo · [docs/gui/](./gui/) |
| **Workspace** | `zedazo-core` · `zedazo-cli` (binario `zedazo`) · `zedazo-api` · `zedazo-carddav` · `apps/web` |

## Descripción

Herramienta para cribar, normalizar, clasificar y deduplicar archivos de contactos **VCF vCard 4.0/3.0** exportados desde **ProtonMail**, **Google Contacts** y **Apple iCloud**.

El núcleo de dominio (`zedazo-core`) lo consumen la **CLI** (`zedazo`) y, desde v0.5.0 (ADR-0015), una **API HTTP** (`zedazo-api`) + **GUI** Next.js (`apps/web`). Procesamiento local, sin telemetría remota por defecto. Remoto single-user vía HTTPS + token (ADR-0016).

Aplica un pipeline determinista de 6 etapas con reglas configurables vía TOML, generando salida en VCF 4.0 limpio + auditoría completa en TSV, CSV y JSON.

## Funcionalidades

### Pipeline completo (`cribar`)

```
VCF → Parse → Normalize → Classify → Screen → Dedup → Write
```

| Etapa | Qué hace |
|-------|----------|
| **Parse** | RFC 6350 §3.2 unfold, §3.4 escape, propiedades agrupadas, v3→v4 |
| **Normalize** | N1-N7 FN (títulos, cargos, partículas), T1-T4 TEL (E.164 +34), ORG (siglas, formas jurídicas) |
| **Classify** | 16 categorías N2 en jerarquía de 3 niveles (PROF-JUD, INST-AUT, FIN-CRYPTO, TECH-SW, SALUD-SOC...) |
| **Screen** | Conservación por categoría (incl. C1/C5/C7) + eliminación (incl. E4/E6) |
| **Dedup** | Union-Find con cierre transitivo D1-D2 (UID exacto, TEL exacto, EMAIL fuzzy, FN fuzzy) |
| **Write** | VCF 4.0 folding 75 octetos + TSV auditoría + CSV/JSON export |

### Comandos CLI adicionales

| Comando | Función |
|---------|---------|
| `audit` | Solo screening + TSV, sin modificar VCF |
| `stats` | Estadísticas en texto, JSON o Markdown |
| `export` | Export CSV o JSON desde pipeline |
| `completions` | Autocompletado bash, zsh, fish (sin pantalla GUI) |

### GUI / API (v0.5.0+)

- API `/api/v1`: uploads, jobs, SSE, artefactos, contactos, duplicados, auditoría, reglas
- GUI: Procesar, Ejecuciones, Auditar, Reglas, Ajustes, Acceso (token), Documentación
- Paridad funcional CLI↔GUI (O10); remoto O11 (ADR-0016)

## Configuración

Archivo TOML opcional con soporte `replace` (reemplazar defaults) / `append` (añadir):

```toml
[zedazo]
replace = false
prefijo_pais = "+34"
conservar_dominios = ["@example.org"]
e2_keywords = ["pharma", "jackpot"]
```

## Integraciones y exportación

| Formato | Dirección | Detalle |
|---------|-----------|---------|
| VCF 4.0 | Salida | FN, N, ORG, TEL, EMAIL, NOTE, X-ZEDAZO-*, PRODID |
| VCF 3.0/4.0 | Entrada | Auto-detección Proton/Google/Apple |
| TSV | Auditoría | Columnas de decisión y evidencia |
| CSV / JSON | Export | Contactos del pipeline |
| TOML | Config | Reglas personalizadas de cribado |
| ISO-8859-1 | Entrada | Transcodificación automática → UTF-8 |

## Tecnologías

| Categoría | Dependencia | Notas |
|-----------|-------------|-------|
| CLI | clap (derive) | 4.5 |
| Parser | nom | 7 (congelado < 8) |
| Config | toml | < 1 (congelado) |
| Encoding | chardetng + encoding_rs | chardetng < 1 |
| API | Axum + Tokio | Solo `zedazo-api` |
| GUI | Next.js App Router | `apps/web`; Web Awesome |
| Dist | cargo-dist | 5 targets + SBOM |

## Arquitectura

```
crates/zedazo-core/   Dominio + application + infra I/O (sin HTTP)
crates/zedazo-cli/    Binario `zedazo` (Clap)
crates/zedazo-api/    Axum `/api/v1` (publish = false)
crates/zedazo-carddav/ Cliente CardDAV (publish = false; ADR-0018)
apps/web/             Next.js — identidad «Archivo Vivo»
apps/landing/         Ficha pública estática (zedazo.alexendros.dev)
deploy/               Compose local/remoto/landing + Caddy
```

**Patrón:** Clean Architecture; `domain` puro sin I/O. Ver [ARCHITECTURE.md](../ARCHITECTURE.md) y ADR-0015/0016.

## Calidad y CI/CD

| Aspecto | Herramienta |
|---------|-------------|
| CI | GitHub Actions en `ubuntu-latest` (`make ci`: fmt, clippy, test, check, doc, docs-validate, parity, web-ci) |
| Release | Tag semver → cargo-dist + SBOM + crates.io (`zedazo-core` luego `zedazo`) |
| Security | `cargo audit` semanal |
| Dependencias | **Renovate** (no Dependabot); pins en ADR-0009 |
| Pre-commit | `make hooks` → fmt + clippy |
| Paridad | O10: `make parity` + matriz en `docs/gui/` |

## Roadmap

Alineado con [ROADMAP.md](../ROADMAP.md) (canónico):

| Versión | Features |
|---------|----------|
| **v0.1.x–v0.2.0** ✅ | Pipeline CLI, rename Zedazo, crates.io |
| **v0.3.0** ✅ | Calidad: reglas C1/C5/C7/E4/E6, invariantes I1–I7 |
| **v0.5.0** ✅ código | GUI + API self-hosted (ADR-0015); tag pendiente |
| **v0.5.1** ✅ código | Remoto HTTPS + token (ADR-0016); tag/bump crates.io pendiente |
| **v0.4.0** | CardDAV CLI: pull, write opt-in, watch, filtros N1/N2 ([docs/carddav.md](./carddav.md), ADR-0018/#48) |
| **v1.0.0** | API de crate estable, benchmarks, corpus grande |
| **Post-v1.0** | OpenTelemetry opt-in (ADR-0017) |

## Seguridad y privacidad

- Procesamiento local, sin telemetría remota por defecto
- Fixtures de tests 100 % sintéticos (sin PII real)
- Remoto: auth fail-closed, cookie HttpOnly, same-origin (ADR-0016)
- Código abierto bajo MIT OR Apache-2.0
- Política de vulnerabilidades en SECURITY.md

## Uso rápido

```bash
cargo install zedazo
zedazo cribar contactos.vcf -o limpio.vcf -a audit.tsv

# GUI local (tras v0.5.0 en código)
cd deploy && docker compose up --build
```

## Enlaces

- GitHub: https://github.com/Soluciones-Alexendros/zedazo
- crates.io: https://crates.io/crates/zedazo
- Documentación: https://docs.rs/zedazo
- Dependencias: Renovate (no Dependabot)
- Cobertura: cargo-llvm-cov → Coveralls
- Deploy: [docs/gui/deploy.md](./gui/deploy.md) (DNS CNAME + landing + GUI remota)
- Marca / TMview: [docs/brand.md](./brand.md)
