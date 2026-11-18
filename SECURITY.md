# Política de Seguridad

### Propósito de este documento

- **Objetivos:** Canal privado para vulnerabilidades, versiones soportadas y consideraciones de VCF/API/CardDAV.
- **Estructura:** Propósito → reporte → versiones → consideraciones.
- **Contenido a integrar según contexto:** Conserva la política de `cargo audit` y ADR-0016/0018. No abras issue público para un aviso de seguridad. No copies un SECURITY de otro producto.

## Reportar vulnerabilidades

Si descubres una vulnerabilidad de seguridad, por favor **no abras un issue público**.

Abre un aviso privado en GitHub:

<https://github.com/Soluciones-Alexendros/zedazo/security/advisories/new>

**Private vulnerability reporting** está activado en este repositorio (comprobado el 2026-11-18). Responderemos en un plazo máximo de 48 horas. No hay dirección de correo publicada en este documento.

## Versiones soportadas

| Versión | Soportada |
|---------|-----------|
| 0.6.x (tag **v0.6.4**, CLI + API/GUI + AppImage) | ✅ Línea activa |
| 0.5.x | ⚠️ Anterior; sin línea de parches propia |
| 0.3.x y anteriores | ⚠️ Solo histórico |

## Consideraciones de seguridad

- **Archivos VCF**: Zedazo procesa archivos de contactos. No ejecutes la herramienta sobre archivos de fuentes no confiables sin revisarlos previamente.
- **Datos personales**: El VCF de salida, la auditoría TSV y el volumen `ZEDAZO_DATA_DIR` (jobs/uploads) contienen datos personales. Trátalos como el original.
- **API/GUI remota (ADR-0016)**:
  - Nunca `ZEDAZO_AUTH_MODE=disabled` en interfaz pública (fail-closed fuera de loopback).
  - Remoto: `token` + HTTPS (Caddy o túnel); rotar `ZEDAZO_AUTH_TOKEN` si se filtra.
  - **Logout** borra la cookie `zedazo_auth` (`HttpOnly`, `Secure`, `SameSite`); **no** revoca un token estático ya copiado (Bearer/manual).
  - **Rotación:** generar nuevo token (`openssl rand -hex 32`), actualizar `.env` / secreto, recrear el servicio API (y Caddy si aplica). Sesiones con cookie antigua fallan hasta nuevo login.
  - No publicar puertos de `api`/`web` al WAN; solo el reverse proxy.
  - **Backups opt-in** del volumen `zedazo-data` pueden conservar datos que la app ya borró (retención no forense).
- **CardDAV (ADR-0018):** credenciales del proveedor (`ZEDAZO_CARDDAV_URL`, `ZEDAZO_CARDDAV_USERNAME`, `ZEDAZO_CARDDAV_PASSWORD`, opcional `ZEDAZO_CARDDAV_ADDRESSBOOK`) distintas de `ZEDAZO_AUTH_TOKEN` (GUI). No viajan por `zedazo.alexendros.dev`. Egreso opt-in vía `zedazo carddav`; el pipeline `cribar` sigue sin red por defecto. Write remoto exige `--confirm` + `If-Match`; HTTP 412 no sobrescribe. HTTPS (TLS 1.2+) obligatorio fuera de loopback; sin `insecure-skip-verify`. Ver [`docs/carddav.md`](docs/carddav.md).
- **Dependencias**: en cada PR/push, `cargo audit` / OSV van en el job `supply-chain` de `ci.yml` (grafo `ci-gate`). Red semanal: la misma suite SCA (más gitleaks) vía [`.github/workflows/security.yml`](.github/workflows/security.yml) (lunes 05:17 UTC + `workflow_dispatch`).
  - **Falla solo** ante vulnerabilidades RustSec. Avisos informativos (`unmaintained`, `unsound`, `yanked`) no fallan el job.
  - Una excepción temporal exige el mismo `RUSTSEC-*` en [`.cargo/audit.toml`](.cargo/audit.toml) y [`deny.toml`](deny.toml), con motivo y fecha de revisión. No ignores un advisory si hay parche publicable (p. ej. `rustls` ≥ 0.23.45 para [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)).
  - Local: `make audit` (y `make deny` para cargo-deny, ADR-0013).
- **CI Security en PR/push**: jobs `gitleaks` y `supply-chain` de [`.github/workflows/ci.yml`](.github/workflows/ci.yml) (gitleaks binario redactado; `pnpm audit --prod --audit-level=high` en `apps/web`; `cargo audit`; OSV high/critical). [`.github/workflows/security.yml`](.github/workflows/security.yml) es la misma suite en cron semanal + `workflow_dispatch` (no duplica el rollup del PR). El check requerido del ruleset sigue siendo `ci-gate`.
- **SAST**: [`.github/workflows/codeql.yml`](.github/workflows/codeql.yml) analiza Rust con CodeQL cuando cambian `crates/zedazo-api`, `crates/zedazo-carddav` o el grafo Cargo. El foco del informe son esas dos crates; la base de CodeQL es el workspace. **No** forma parte de `ci-gate`.
