# Matriz de paridad funcional CLI ↔ GUI

**Versión:** 0.2.0  
**Fecha:** 2026-09-09  
**Traza:** ADR-0015, ADR-0016, SPECS O10/O11, ROADMAP v0.5.0–v0.5.1

La GUI debe cubrir la **paridad de casos de uso**, no sustituir artefactos propios del terminal (`completions`).
O11 (auth remoto) no altera la semántica del pipeline: O10 sigue válido con `ZEDAZO_AUTH_MODE=token`.

| Capacidad | CLI | GUI / API | Criterio de equivalencia |
|-----------|-----|-----------|--------------------------|
| Lectura VCF 3.0 | `cribar` / `audit` | Upload + job / audit | Mismos conteos y decisiones sobre fixtures Google/Apple |
| Lectura VCF 4.0 | idem | idem | Fixture Proton |
| Detección Proton / Google / Apple | `source` auto | Manifest `source_detected` | Igual que `source_detail` |
| Normalización | pipeline cribar | Resultados en ficha contacto | Mismos FN/TEL/ORG normalizados |
| Clasificación | cribar | Categorías en ficha | Mismas N1/N2(/N3) |
| Cribado | cribar | Estados Conserved/Eliminated/NeedsReview/Quarantine | Misma decisión + regla |
| Dedup UID (D1) | cribar | Vista Duplicados | Mismos grupos / merged_uids |
| Dedup teléfono / email fuzzy / nombre | cribar | Vista Duplicados + evidencias | Misma fusión transitiva |
| Cierre transitivo Union-Find | cribar | Grupos | Misma cardinalidad de grupos |
| Export VCF 4.0 | `-o` | Artifact `vcf` | Diff semántico (ignorar timestamps de job) |
| Export audit TSV | `-a` | Artifact `audit_tsv` | Filas equivalentes |
| Export CSV / JSON | `export` | Artifacts | Contenido equivalente |
| Stats text/json/markdown | `stats` | Stats + artifact md/json | Conteos alineados con pipeline completo en jobs |
| Reglas TOML append/replace | `-c` | Pantalla Reglas + job | Mismo hash de reglas → mismo resultado |
| Warning config deprecada `[cribado]` | tracing | Warning en UI | Visible sin fallar |
| Errores parseo / malformado | exit ≠ 0 | Job `failed` + error tipado | Sin PII en logs |
| Codificaciones (UTF-8, ISO-8859-1) | encoding | Upload | Fixture `iso_sample.vcf` |
| Preservación PHOTO/folding | writer | Artifact VCF | Roundtrip como CLI |
| Cancelación | N/A (CTRL-C) | `POST .../cancel` | Artefactos incompletos no descargables como definitivos |
| Shell completions | `completions` | Solo documentación | Sin pantalla equivalente |
| CardDAV pull/list/write/watch (ADR-0018) | `zedazo carddav` | N/A (CLI only) | Sin pantalla ni `/api/v1` CardDAV |

## Test de equivalencia (CI)

Para cada fixture en `crates/zedazo-core/tests/fixtures/*.vcf` relevante: ejecutar CLI (`cribar`) y API HTTP con la misma config; normalizar timestamps y orden de categorías; comparar VCF, TSV, CSV, JSON y conteos. Criterio: **misma semántica**, no “ambos funcionan”.

**Cobertura CI:** `make parity` → `cargo test -p zedazo-api --test equivalence_http` (fixtures: sample-contacts, google_*, proton_sample, iso_sample, duplicates, edge_cases). Smoke core: `equivalence_cli_api`.

| Capacidad (filas pipeline) | Cubierta por harness O10 |
|----------------------------|--------------------------|
| Lectura VCF 3/4, encoding ISO | sí (fixtures Google/Proton/iso) |
| Cribado / conteos / export VCF+audit+CSV+JSON+stats | sí |
| Dedup grupos (conteos + artifacts) | sí (`duplicates.vcf`) |
| Warning `[cribado]` / cancelación UI / evidencias D1-D2 en GUI | sí (API+UI V1 endurecido) |
| Shell completions | N/A (sin pantalla) |
| CardDAV `list`/`pull` | N/A (CLI only; ADR-0018) |