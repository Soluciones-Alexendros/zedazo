# Modelo de amenazas — Zedazo web self-hosted

**Versión:** 0.2.0  
**Fecha:** 2026-09-09  
**Traza:** ADR-0015, ADR-0016; egreso CardDAV: ADR-0018 (aceptada; CLI)

## Activos

- Agendas VCF (datos personales; posibles categorías sensibles)
- Artefactos de salida (VCF/TSV/CSV/JSON)
- Reglas TOML del operador
- Metadatos de jobs (hashes, conteos)
- Secreto de acceso (`ZEDAZO_AUTH_TOKEN`) y cookie de sesión
- Credenciales CardDAV (`ZEDAZO_CARDDAV_*`; distintas del token de GUI)

## Supuestos

### V1 local (ADR-0015)

- Operador único en `127.0.0.1`
- `ZEDAZO_AUTH_MODE=disabled` solo en loopback (fail-closed si bind no-loopback)
- Sin red saliente durante procesamiento por defecto (CardDAV es egreso **opt-in**, ADR-0018)
- Volumen de datos bajo control del operador

### V1 remoto single-user (ADR-0016)

- Un operador; sin multi-tenant
- `ZEDAZO_AUTH_MODE=token` + token de alta entropía
- TLS terminado en Caddy (o túnel); API/web solo en red Docker interna
- Same-origin: cookie HttpOnly `zedazo_auth` para SSE
- Preferir túnel (Tailscale / Cloudflare) antes de abrir puertos al router

## Amenazas y controles

| ID | Amenaza | Control |
|----|---------|---------|
| T1 | Path traversal / escritura fuera del job | IDs opacos; rechazar rutas de usuario; denegar symlinks |
| T2 | DoS por upload enorme | `ZEDAZO_MAX_UPLOAD_BYTES`; límites de contactos/tiempo |
| T3 | XSS vía campos vCard | Renderizar como texto; prohibido `dangerouslySetInnerHTML` |
| T4 | Exfiltración accidental “copiar todo” | Sin botón masivo por defecto en multiusuario futuro; V1: cuidado en UX |
| T5 | Logs con PII | Logs sin contenido de contacto por defecto |
| T6 | Contenedor privilegiado | Usuario no-root; FS RO excepto data dir; sin Docker socket |
| T7 | CORS abierto / CSRF | Modo token: CORS acotado o same-origin; cookie `SameSite=Strict`; CSRF residual bajo same-site |
| T8 | Telemetría no deseada | `ZEDAZO_OTEL_ENABLED=false` por defecto |
| T9 | Retención indefinida | TTL `ZEDAZO_JOB_RETENTION_HOURS` + borrado verificable |
| T10 | Sustitución de CLI frágil | No invocar CLI vía Node; core compartido |
| T11 | Exposición sin auth | Fail-closed: `disabled` solo loopback; arranque aborta si no |
| T12 | Robo de token/cookie | HTTPS obligatorio en remoto; cookie `Secure; HttpOnly`; token fuera de git |
| T13 | Publicar API/web al WAN | Solo Caddy (o túnel) publica puertos; profile `remote` |
| T14 | Filtrar app-password CardDAV / reutilizar token GUI | Env `ZEDAZO_CARDDAV_*` distinto de `ZEDAZO_AUTH_TOKEN`; no loguear secretos; egreso solo CLI opt-in |

## Fuera de alcance

- Modelo `team` (ownership, auditoría de accesos, revocación, OAuth/OIDC)
- Amenazas de red multi-tenant / SaaS
