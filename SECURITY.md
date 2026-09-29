# Política de Seguridad

### Propósito de este documento

- **Objetivos:** Canal privado para vulnerabilidades, versiones soportadas y consideraciones de VCF/API/CardDAV.
- **Estructura:** Propósito → reporte → versiones → consideraciones.
- **Contenido a integrar según contexto:** Conserva la política de `cargo audit` y ADR-0016/0018. No abras issue público para un aviso de seguridad. No copies un SECURITY de otro producto.

## Reportar vulnerabilidades

Si descubres una vulnerabilidad de seguridad, por favor **no abras un issue público**.

Envía un correo a los mantenedores del proyecto con los detalles. Responderemos en un plazo máximo de 48 horas.

## Versiones soportadas

| Versión | Soportada |
|---------|-----------|
| 0.5.x (CLI + API/GUI en `main`) | ✅ Código activo |
| 0.3.x (crates.io) | ✅ Publicada |
| 0.2.x / 0.1.x | ⚠️ Solo histórico |

## Consideraciones de seguridad

- **Archivos VCF**: Zedazo procesa archivos de contactos. No ejecutes la herramienta sobre archivos de fuentes no confiables sin revisarlos previamente.
- **Datos personales**: El VCF de salida, la auditoría TSV y el volumen `ZEDAZO_DATA_DIR` (jobs/uploads) contienen datos personales. Trátalos como el original.
- **API/GUI remota (ADR-0016)**:
  - Nunca `ZEDAZO_AUTH_MODE=disabled` en interfaz pública (fail-closed fuera de loopback).
  - Remoto: `token` + HTTPS (Caddy o túnel); rotar `ZEDAZO_AUTH_TOKEN` si se filtra.
  - No publicar puertos de `api`/`web` al WAN; solo el reverse proxy.
- **CardDAV (ADR-0018):** credenciales del proveedor (`ZEDAZO_CARDDAV_URL`, `ZEDAZO_CARDDAV_USERNAME`, `ZEDAZO_CARDDAV_PASSWORD`, opcional `ZEDAZO_CARDDAV_ADDRESSBOOK`) distintas de `ZEDAZO_AUTH_TOKEN` (GUI). No viajan por `zedazo.alexendros.dev`. Egreso opt-in vía `zedazo carddav`; el pipeline `cribar` sigue sin red por defecto. Write remoto exige `--confirm` + `If-Match`; HTTP 412 no sobrescribe. HTTPS (TLS 1.2+) obligatorio fuera de loopback; sin `insecure-skip-verify`. Ver [`docs/carddav.md`](docs/carddav.md).
- **Dependencias**: `cargo audit` semanal (lunes 08:00 UTC) y en PRs/push que toquen el lockfile o la política (`.github/workflows/audit.yml`).
  - **Falla solo** ante vulnerabilidades RustSec. Avisos informativos (`unmaintained`, `unsound`, `yanked`) no fallan el job.
  - Una excepción temporal exige el mismo `RUSTSEC-*` en [`.cargo/audit.toml`](.cargo/audit.toml) y [`deny.toml`](deny.toml), con motivo y fecha de revisión. No ignores un advisory si hay parche publicable (p. ej. `rustls` ≥ 0.23.45 para [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285)).
  - Local: `make audit` (y `make deny` para cargo-deny, ADR-0013).
