---
title: "Matriz OWASP ASVS L1"
date: "2026-11-18"
status: "Activo"
version: "4.0.3"
---

# Matriz ASVS — objetivo L1

Estándar citado: **OWASP ASVS 4.0.3**. Objetivo de Zedazo: **L1**. Esto no es una certificación.

Un control está `cubierto` solo si la evidencia de esta tabla existe en el árbol. Lo no recorrido en esta pasada queda `no evaluado`. Los requisitos que el estándar marca solo para L2 o L3 no cuentan para el objetivo L1; si se mencionan, llevan el nivel en la columna.

## Controles revisados

| ID | Nivel | Estado | Evidencia |
|---|---|---|---|
| Canal de reporte (informe H-01; no hay un requisito L1 con este texto) | gobernanza | parcial | [`SECURITY.md`](../../SECURITY.md) enlaza el formulario de avisos privados. La activación del ajuste en GitHub no se ha comprobado en esta sesión. |
| Verificación de artefacto (informe INT-1) | gobernanza | cubierto | [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml) sube `SHA256SUMS` y atestación. [`docs/desktop.md`](../desktop.md) documenta `sha256sum -c` y `gh attestation verify`. |
| SAST CodeQL (informe H-02) | gobernanza | parcial | [`.github/workflows/codeql.yml`](../../.github/workflows/codeql.yml) existe y no está en `needs` de `ci-gate`. La primera corrida en GitHub queda pendiente. |
| V9.1.1 TLS en la conectividad del cliente | L1 | parcial | [`SECURITY.md`](../../SECURITY.md): remoto con HTTPS y CardDAV con TLS 1.2+ fuera de loopback. El uso local en loopback no exige TLS. |
| V9.1.2 Suites de cifrado | L1 | no evaluado | No hay inventario de suites en esta revisión. |
| V9.1.3 TLS 1.2 y 1.3 | L1 | parcial | La frase de CardDAV exige TLS 1.2+. No hay prueba de la configuración de Caddy en esta pasada. |
| V3.4.1 cookie `Secure` | L1 | cubierto | [`SECURITY.md`](../../SECURITY.md): cookie `zedazo_auth` con `Secure`. |
| V3.4.2 cookie `HttpOnly` | L1 | cubierto | Misma sección: `HttpOnly`. |
| V3.4.3 cookie `SameSite` | L1 | cubierto | Misma sección: `SameSite`. |
| V3.4.4 prefijo `__Host-` | L1 | parcial | El nombre documentado es `zedazo_auth`, sin prefijo `__Host-`. |
| V3.3.1 logout invalida la sesión | L1 | parcial | El logout borra la cookie y no revoca el token estático ya copiado (`SECURITY.md`). |
| V14.2.1 componentes al día en el build | L1 | parcial | `make audit` y `make deny` en CI. Siguen los ignores de `glib` 0.18.5 y `proc-macro-error` 1.0.4 (revisión 2027-02-18). |
| V14.2.3 SRI de assets en CDN | L1 | no evaluado | No se ha auditado `apps/web` en esta pasada. |
| V14.4.3 CSP | L1 | parcial | [`owasp-review-0.6.0.md`](owasp-review-0.6.0.md) deja S-14 y V-02 en fase C. |
| V14.4.1, V14.4.2, V14.4.4–V14.4.7 cabeceras | L1 | no evaluado | Sin lectura de las cabeceras reales de la API en esta pasada. |
| V2.10.4 secretos fuera del código | L1 | cubierto | Token por entorno (`ZEDAZO_AUTH_TOKEN`) y job `gitleaks` en [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml). |
| V14.2.5 SBOM | L2 | parcial | SBOM de escritorio en `publish.yml` / `release.yml`. El de crates es [`.github/workflows/sbom-crates.yml`](../../.github/workflows/sbom-crates.yml), manual, sin `cargo publish`. No es requisito L1. |
| V1.14.2 firmas de binarios al desplegar | L2 | parcial | `gh attestation verify` en `publish.yml`. No es el modelo de firma a dispositivo del requisito. No es requisito L1. |

## Capítulos L1 no recorridos

Quedan `no evaluado`: V4 control de acceso, V5 validación, V6 criptografía, V7 errores y logs, V8 protección de datos, V10 código malicioso, V11 lógica de negocio, V12 ficheros, V13 API (más allá de lo ya citado), y el resto de V1, V2, V3, V9 y V14 que no aparece en la tabla.
