# Matriz de paridad funcional CLI ↔ GUI

**Versión:** 0.3.0  
**Fecha:** 2026-10-16  
**Traza:** ADR-0015, ADR-0016, ADR-0018, ADR-0021, SPECS O10/O11, ROADMAP v0.5.0–v0.6.0

La GUI cubre la **paridad de casos de uso**, no sustituye artefactos propios del terminal (`completions`).
O11 (auth remoto) no altera la semántica del pipeline: O10 sigue válido con `ZEDAZO_AUTH_MODE=token`.
El AppImage de escritorio (ADR-0021) reutiliza la misma GUI/API loopback: misma matriz O10; guía en [docs/desktop.md](../desktop.md).

| Capacidad | CLI | Endpoint API | Ruta GUI | Test |
|-----------|-----|--------------|----------|------|
| Procesar / cribado | `zedazo cribar` | `POST /uploads` + `POST /jobs` + `GET /jobs/{id}` | `/procesar` | `equivalence_http::equivalence_cli_http_all_fixtures` |
| Auditar | `zedazo audit` | `POST /audits` · `GET /jobs/{id}/audit` | `/auditar` | harness O10 (artifact `audit_tsv`) |
| Estadísticas | `zedazo stats` | `GET /jobs/{id}/stats` · artifact `stats_json` | ficha ejecución | harness O10 |
| Exportar VCF/CSV/JSON | `cribar -o` / `export` | `GET /jobs/{id}/artifacts/{kind}` | descarga en ejecución | harness O10 + `download_artifacts_after_completed` |
| Reglas + validación | `cribar -c` | `POST /rules/validate` · `POST /rules/preview` | `/reglas` | `adverse_rules_validate_rejects_invalid_toml` |
| Duplicados | dedup en `cribar` | `GET /jobs/{id}/duplicates` | vista Duplicados | fixture `duplicates.vcf` en harness |
| Cancelar | CTRL-C (N/A API) | `POST /jobs/{id}/cancel` | acción en ejecución | `cancel_then_delete_is_idempotent` |
| Descargar | ficheros `-o`/`-a` | `GET .../artifacts/{kind}` | botones descarga | `download_artifacts_after_completed` |
| Borrar | N/A | `DELETE /jobs/{id}` | acción borrar | `cancel_then_delete_is_idempotent` (2.º DELETE 204) |
| Lectura VCF 3.0 / 4.0 | `cribar` / `audit` | Upload + job | `/procesar` | fixtures Google / Proton |
| Encoding ISO-8859-1 | encoding CLI | Upload | `/procesar` | `iso_sample.vcf` |
| Casos adversos (escape, PHOTO, vacío) | exit ≠ 0 / cuarentena | upload 400 / job `failed` | UI error | `edge_cases.vcf` · `adverse_empty_vcf_upload_rejected` |
| Shell completions | `completions` | — | solo docs | N/A |
| CardDAV (ADR-0018) | `zedazo carddav` | — (CLI only) | — | mocks en `crates/zedazo-carddav/tests/` |

## Test de equivalencia (CI)

Para cada fixture en `crates/zedazo-core/tests/fixtures/*.vcf` relevante: ejecutar CLI (`cribar`) y API HTTP con la misma config; normalizar timestamps y orden de categorías; comparar VCF, TSV, CSV, JSON y conteos. Criterio: **misma semántica**, no “ambos funcionan”.

**Cobertura CI:** `make parity` → `cargo test -p zedazo-api --test equivalence_http` (fixtures: sample-contacts, google_*, proton_sample, iso_sample, duplicates, edge_cases). Adversos: vacío, TOML inválido, cancel+delete. Smoke core: `equivalence_cli_api` si aplica.

| Capacidad (filas pipeline) | Cubierta por harness O10 |
|----------------------------|--------------------------|
| Lectura VCF 3/4, encoding ISO | sí (fixtures Google/Proton/iso) |
| Cribado / conteos / export VCF+audit+CSV+JSON+stats | sí |
| Dedup grupos (conteos + artifacts) | sí (`duplicates.vcf`) |
| Cancelación / borrado idempotente / reglas inválidas / upload vacío | sí (tests adversos) |
| Warning `[cribado]` / evidencias D1-D2 en GUI | sí (API+UI V1) |
| Shell completions | N/A (sin pantalla) |
| CardDAV | CLI + mocks (tabla siguiente) |

## CardDAV CLI (mocks, sin red real)

Validación sobre `crates/zedazo-carddav/tests/` (incluido en `cargo test --workspace`):

| Caso | Test |
|------|------|
| Descubrimiento / lectura (well-known, addressbook) | `mock_pull::pull_via_well_known_and_nextcloud_paths` |
| Addressbook explícito | `mock_pull::explicit_addressbook_skips_well_known` |
| Credenciales incorrectas (401) | `mock_pull::unauthorized_is_typed_error` |
| Fallo de red / 429 agotado | `mock_pull::persistent_429_exhausts_retries` |
| Escritura con confirmación (If-Match) | `mock_write::put_if_match_success_sends_quoted_etag` |
| HTTP 412 (conflicto) | `mock_write::put_http_412_is_conflict_not_overwrite` |
| `watch` sin pérdida silenciosa | `mock_watch::watch_detects_sync_token_and_ctag_change` |
| Filtro categorías N1/N2 | `mock_filter::pull_category_filter_keeps_n1_and_n2_matches` |

Sin pantalla GUI ni rutas `/api/v1` CardDAV (ADR-0018).
