---
title: "OWASP review v0.6.0"
date: "2026-10-26"
status: "Activo"
traza: "plan v0.6.0 seguridad/escritorio"
---

# OWASP review — zedazo v0.6.0

Revisión de la superficie S/V del plan de cierre (rama `plan/zedazo-v060-seguridad`). Fecha Madrid: 2026-10-26. Fase C (S-11, S-14, S-17, S-19, V-02) **no** ejecutada.

## Tabla S/V

| ID | Tema | PR / SHA | Prueba | Estado |
| --- | --- | --- | --- | --- |
| S-01 | Host/Origin; sin `AllowOrigin::mirror_request` | [#94](https://github.com/Soluciones-Alexendros/zedazo/pull/94) `519f5498` | `cors_host`, `auth_http` | Hecho |
| S-02 | I7 same-file / no sobrescribir entrada | [#93](https://github.com/Soluciones-Alexendros/zedazo/pull/93) `816f7ca6` | `i7_guard`, parity | Hecho |
| S-03 | Dedup indexada D3–D6 + cancel | [#95](https://github.com/Soluciones-Alexendros/zedazo/pull/95) `a272a2aa` | unit + `dedup_perf` | Hecho |
| S-04 | Uploads streaming + tope + `BEGIN:VCARD` | [#94](https://github.com/Soluciones-Alexendros/zedazo/pull/94) `519f5498` | `upload_limits` | Hecho |
| S-05 | Redacción / códigos de error | [#93](https://github.com/Soluciones-Alexendros/zedazo/pull/93), [#94](https://github.com/Soluciones-Alexendros/zedazo/pull/94) | `error_redact` | Hecho |
| S-06 | `secure_fs` 0600/0700 | [#93](https://github.com/Soluciones-Alexendros/zedazo/pull/93), [#94](https://github.com/Soluciones-Alexendros/zedazo/pull/94) | unit + storage | Hecho |
| S-07 | Fórmulas CSV/TSV (+ excepción TEL) | [#99](https://github.com/Soluciones-Alexendros/zedazo/pull/99) `ae7f1dde` | `formula_guard` | Hecho |
| S-08 | Pin Actions a SHA | [#100](https://github.com/Soluciones-Alexendros/zedazo/pull/100) `ff09d58e` | zizmor medium+ = 0 | Hecho (73 refs; informe decía 67) |
| S-09 | Un solo release + attest publish | [#98](https://github.com/Soluciones-Alexendros/zedazo/pull/98) `749c163a` | `publish.yml` | Hecho |
| S-10 | Tools pin SHA256 + repro informativo | [#98](https://github.com/Soluciones-Alexendros/zedazo/pull/98) `749c163a` | prefetch + job repro | Hecho |
| S-11 | Token ≥32, 429 login, sesión | — | — | Fase C |
| S-12 | Retención / wipe escritorio | [#96](https://github.com/Soluciones-Alexendros/zedazo/pull/96) `4155f743` | retention + e2e | Hecho |
| S-13 | `retention_hours` acotado | [#94](https://github.com/Soluciones-Alexendros/zedazo/pull/94) `519f5498` | API tests | Hecho |
| S-14 | CSP hashes / sin unsafe-inline | — | — | Fase C |
| S-15 | Permisos mínimos / digests Docker | [#100](https://github.com/Soluciones-Alexendros/zedazo/pull/100) `ff09d58e` | hadolint + dockerfile-digests | Hecho (Caddy sin `cap_drop:ALL`) |
| S-16 | Renovate pin digests Actions | [#100](https://github.com/Soluciones-Alexendros/zedazo/pull/100) `ff09d58e` | `renovate.json` | Hecho |
| S-17 | cargo-fuzz unfold/parse/… | — | — | Fase C |
| S-18 | glib / RUSTSEC-2024-0429 (Tauri) | ignore OSV | `osv-scanner.toml` | **Riesgo aceptado** — revisión tras upgrade Tauri/gtk-rs (fecha revisión: 2026-10-26; no quitar ignore: OSV aún lo reporta) |
| S-19 | CardDAV tope / redirects privadas | — | — | Fase C |
| V-01 | Descarga artefacto con Bearer | [#96](https://github.com/Soluciones-Alexendros/zedazo/pull/96) `4155f743` | e2e desktop | Hecho |
| V-02 | CSP / cabeceras siempre | — | — | Fase C |
| V-03 | `on_navigation` + xdg-open | [#96](https://github.com/Soluciones-Alexendros/zedazo/pull/96) `4155f743` | e2e | Hecho |
| V-04 | Sin `__TAURI_INTERNALS__` | [#96](https://github.com/Soluciones-Alexendros/zedazo/pull/96) `4155f743` | e2e | Hecho |
| V-05 | Quitar `zod` sin imports | este PR | `pnpm` lock | Hecho |
| P-01 | release-please simple 0.5.1 | [#92](https://github.com/Soluciones-Alexendros/zedazo/pull/92) `919a6602` | docs-validate | Hecho |
| P-02 | e2e sin red | [#101](https://github.com/Soluciones-Alexendros/zedazo/pull/101) `16ba50cf` | `desktop-no-network` | Hecho |
| P-07 | Smoke Ubuntu 26 en CI | [#101](https://github.com/Soluciones-Alexendros/zedazo/pull/101) `16ba50cf` | documentado (EGL/GIO) | No en ci-gate |

## Notas

- Escritorio = **ADR-0021**. Guía operador: [`docs/desktop.md`](../desktop.md).
- Ignore OSV `RUSTSEC-2024-0429` / `GHSA-wrw7-89jp-8q8g`: se mantiene (S-18). Comprobado 2026-10-26 con `osv-scanner` sin config: sigue saliendo en `glib 0.18.5`. No tocar `.cargo/audit.toml` ni `deny.toml` en el mismo gesto.
