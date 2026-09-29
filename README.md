# Zedazo

[![CI](https://github.com/Soluciones-Alexendros/zedazo/actions/workflows/ci.yml/badge.svg)](https://github.com/Soluciones-Alexendros/zedazo/actions/workflows/ci.yml)
[![Security Audit](https://github.com/Soluciones-Alexendros/zedazo/actions/workflows/audit.yml/badge.svg)](https://github.com/Soluciones-Alexendros/zedazo/actions/workflows/audit.yml)
[![Coverage Status](https://coveralls.io/repos/github/Soluciones-Alexendros/zedazo/badge.svg?branch=main)](https://coveralls.io/github/Soluciones-Alexendros/zedazo?branch=main)
[![Crates.io](https://img.shields.io/crates/v/zedazo?color=orange)](https://crates.io/crates/zedazo)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/rustc-1.80+-blue.svg)](https://blog.rust-lang.org/2024/07/25/Rust-1.80.0.html)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**Tu agenda, pasada por el zedazo fino.**

Criba, normaliza, clasifica y deduplica contactos VCF exportados desde **ProtonMail**, **Google Contacts** o **Apple iCloud**, con reglas deterministas, deduplicación transitiva y normalización de nombres y teléfonos.

Desde **v0.5.0** incluye **GUI web + API HTTP** self-hosted sobre el mismo core (`zedazo-core`): local en loopback (ADR-0015) y remoto single-user con HTTPS + token (ADR-0016). La CLI sigue siendo el canal oficial de automatización.

> Antes: `vcf-cribador`. Migración: ver [CHANGELOG 0.2.0](CHANGELOG.md#020---2026-08-15) y [ADR-0014](DECISIONS.md).

## Quick start

```bash
# Instalar
cargo install zedazo

# Cribar un archivo (conservados → limpio.vcf, trazabilidad → auditoría.tsv)
zedazo cribar mis-contactos.vcf -o limpio.vcf -a auditoria.tsv

# Solo auditar sin modificar
zedazo audit mis-contactos.vcf -o auditoria.tsv

# Estadísticas
zedazo stats limpio.vcf
zedazo stats limpio.vcf -f json
zedazo stats limpio.vcf -f markdown

# Exportar a CSV o JSON
zedazo export limpio.vcf -o contactos.csv
zedazo export limpio.vcf -o contactos.json -f json
```

## GUI / API self-hosted (ADR-0015 / ADR-0016)

Workspace: `crates/zedazo-core` · `crates/zedazo-cli` · `crates/zedazo-api` · `apps/web`.

```bash
# API local (auth disabled solo en loopback)
ZEDAZO_DATA_DIR=./data ZEDAZO_BIND=127.0.0.1:8080 cargo run -p zedazo-api

# Frontend
pnpm --dir apps/web install
pnpm --dir apps/web dev

# Docker local
docker compose -f deploy/docker-compose.yml up --build

# Docker remoto single-user (HTTPS + token)
cp deploy/.env.example deploy/.env   # editar token
docker compose -f deploy/docker-compose.remote.yml --env-file deploy/.env up --build
# → https://127.0.0.1:8443
```

Docs: [docs/gui/](docs/gui/) · OpenAPI [docs/api/openapi.yaml](docs/api/openapi.yaml) · [deploy.md](docs/gui/deploy.md).

## Capturas

| Procesar | Ejecuciones | Ajustes |
| --- | --- | --- |
| ![Procesar VCF en Zedazo](docs/screenshots/procesar-light.png) | ![Lista de ejecuciones](docs/screenshots/ejecuciones-light.png) | ![Ajustes de apariencia y privacidad](docs/screenshots/ajustes-light.png) |

<details>
<summary>Tema oscuro</summary>

| Procesar | Ejecuciones | Ajustes |
| --- | --- | --- |
| ![Procesar VCF (oscuro)](docs/screenshots/procesar-dark.png) | ![Ejecuciones (oscuro)](docs/screenshots/ejecuciones-dark.png) | ![Ajustes (oscuro)](docs/screenshots/ajustes-dark.png) |

</details>

## Demo en 60 segundos

```bash
# Fixture sintético de ejemplo
cp examples/sample.vcf /tmp/sample.vcf

# Stack local (API :8080, web :3000)
docker compose -f deploy/docker-compose.yml up --build
# Abrir http://127.0.0.1:3000 → Procesar → subir examples/sample.vcf

# O solo CLI
cargo install zedazo   # o binario de release tras tag v0.5.1
zedazo cribar examples/sample.vcf -o /tmp/out.vcf -a /tmp/audit.tsv
```

## Configuración

Opcional: crea un archivo TOML para personalizar el cribado.

```toml
# zedazo.toml
[zedazo]
prefijo_pais = "+34"         # prefijo telefónico por defecto
replace = false              # false = añade a los defaults, true = reemplaza
conservar_dominios = [       # dominios de email que NUNCA se eliminan
    "@example.org",
    "@admin.gob.es"
]
e2_keywords = [              # palabras clave adicionales para detección de spam
    "pharma",
    "jackpot"
]

[clasificacion]
replace = false              # false = añade a las reglas por defecto, true = reemplaza
[[clasificacion.reglas]]
regex = "(?i)rob[óo]tica|maker"
n1 = "TEC"
n2 = "TEC-HW"
n3 = "HW-MAKER"
```

```bash
zedazo cribar contactos.vcf --config zedazo.toml
```

La sección `[cribado]` sigue aceptándose con un warning de deprecación.

## Pipeline

```
  VCF  ──→  Parse   ──→  Normalize  ──→  Classify  ──→  Screen  ──→  Dedup  ──→  Write
 4.0/3.0    unfold      FN · TEL · ORG    16 categorías     C2-C6          Union-Find     VCF
            unescape     E.164 · N7        N1 + N2          E1-E3          cierre         TSV
            grouped                                                     transitivo      CSV/JSON
```

| Etapa         | Descripción                                                                                           |
| ------------- | ----------------------------------------------------------------------------------------------------- |
| **Parse**     | RFC 6350 §3.2 (unfold), §3.4 (escape). Propiedades agrupadas (`ITEM1.EMAIL`). Compatibilidad v3 → v4. |
| **Normalize** | N1-N7: capitalización de nombres, extracción de títulos, cargos, partículas. T1-T4: E.164 + tipos T4.  |
| **Classify**  | Taxonomía N1/N2/N3: PROF, INST, FIN, FORM, TEC, HOST, TRAN, INMO, SERV, ASOC, SALUD, etc.                |
| **Screen**    | C2-C6: conservar por categoría. E1-E3: eliminar huérfanos, spam, email-only.                          |
| **Dedup**     | Union-Find con cierre transitivo. Coincidencia por TEL exacto, EMAIL fuzzy, FN fuzzy.                 |
| **Write**     | VCF 4.0 con folding 75 octetos. ADR y tipos T4 preservados. TSV de trazabilidad. CSV/JSON export.    |

## Ejemplo real

```
$ zedazo cribar contacts-2025.vcf -o limpio.vcf -a audit.tsv

=== Estadísticas de cribado ===
Total entrada:   475
  Conservados:   221
  Eliminados:    254
  Fusionados:    0
  Cuarentena:    0
  Needs Review:  1

Por categoría:
  FIN-CRYPTO:  3    FIN-FINTEC:  5    INST-AUT:  7
  PROF-JUD:    3    PROF-NOT:    2    PROF-COL:  4
  TEC-COM:     3    SALUD-SOC:   2    ...
```

## Arquitectura

```
crates/zedazo-core/     Dominio + application + infra I/O (sin HTTP)
crates/zedazo-cli/      Binario `zedazo` (Clap)
crates/zedazo-api/      API Axum `/api/v1` (+ auth ADR-0016)
apps/web/               GUI Next.js (solo HTTP; sin lógica de cribado)
deploy/                 Docker Compose local + remoto (Caddy)
```

→ [`ARCHITECTURE.md`](ARCHITECTURE.md) · ADR-0015 · ADR-0016

## Documentación

| Documento | Contenido |
| -------------------------------------------------------------- | ---------------------------------------------------- |
| [`SPECS.md`](SPECS.md)                                         | Especificación, invariantes, criterios de aceptación |
| [`ARCHITECTURE.md`](ARCHITECTURE.md)                           | Clean Architecture, CI/CD, capas                     |
| [`ROADMAP.md`](ROADMAP.md)                                     | Hitos y criterios de salida                          |
| [`DECISIONS.md`](DECISIONS.md)                                 | ADR con IDs estables                                 |
| [`AGENTS.md`](AGENTS.md)                                       | Contrato para agentes de código                      |
| [`docs/gui/`](docs/gui/)                                       | Paridad O10, deploy, threat-model, retención         |
| [`docs/api/openapi.yaml`](docs/api/openapi.yaml)               | Contrato HTTP `/api/v1`                              |
| [`docs/domain.md`](docs/domain.md)                             | Lenguaje ubicuo, entidades, rules                    |
| [`docs/implementation-guide.md`](docs/implementation-guide.md) | Guía de implementación (histórico MVP)               |
| [`docs/test-plan.md`](docs/test-plan.md)                       | Estrategia de testing, fixtures                      |
| [`docs/events.md`](docs/events.md)                             | Comandos, eventos                                    |

## Desarrollo

```bash
git clone https://github.com/Soluciones-Alexendros/zedazo.git
cd zedazo

make hooks     # instalar pre-commit hooks
make ci        # fmt + clippy + test + check + doc + docs-validate + parity + web-ci
make release   # build release
```

Ver [`CONTRIBUTING.md`](CONTRIBUTING.md) para la guía de contribución.

## Seguridad

Reporta vulnerabilidades de forma privada. Ver [`SECURITY.md`](SECURITY.md).

Ejecutamos `cargo audit` semanalmente vía GitHub Actions.

## Licencia

MIT OR Apache-2.0 · Ver [`LICENSE-MIT`](LICENSE-MIT) y [`LICENSE-APACHE`](LICENSE-APACHE).
