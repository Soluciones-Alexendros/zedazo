# Despliegue self-hosted

**Traza:** ADR-0014 (dominio de producto), ADR-0015 (local), ADR-0016 (remoto single-user).

## Variables

| Variable | Default | Notas |
|----------|---------|-------|
| `ZEDAZO_API_ENABLED` | true | Operativo |
| `ZEDAZO_WEB_ENABLED` | false | Activar frontend |
| `ZEDAZO_STORAGE_MODE` | ephemeral | FS + manifest |
| `ZEDAZO_DATA_DIR` | `/var/lib/zedazo` | Volumen |
| `ZEDAZO_MAX_UPLOAD_BYTES` | 52428800 | 50 MiB |
| `ZEDAZO_JOB_RETENTION_HOURS` | 24 | TTL |
| `ZEDAZO_AUTH_MODE` | `disabled` | `disabled` solo loopback; `token` en remoto |
| `ZEDAZO_AUTH_ALLOW_DISABLED_NON_LOOPBACK` | `false` | Solo Docker local: API escucha `0.0.0.0` dentro del contenedor mientras el host publica `127.0.0.1` |
| `ZEDAZO_AUTH_TOKEN` | — | Obligatorio si `token`; alta entropía; no commitear |
| `ZEDAZO_AUTH_COOKIE_SECURE` | `true` | Cookie `Secure`; `false` solo pruebas HTTP locales con token |
| `ZEDAZO_CORS_ORIGIN` | vacío | Orígenes explícitos (coma-separados) en modo token; vacío = sin CORS permisivo |
| `ZEDAZO_OTEL_ENABLED` | false | Opt-in |
| `ZEDAZO_BIND` | `127.0.0.1:8080` | Bind API |

## Docker Compose — local (loopback)

```bash
cd deploy
docker compose up --build
```

API: `http://127.0.0.1:8080` · Web: `http://127.0.0.1:3000`  
Auth: `disabled` (puertos solo en loopback del host).

TLS local opcional: `docker compose --profile proxy up --build` → `https://127.0.0.1:8443`.

## Docker Compose — remoto (ADR-0016)

```bash
cd deploy
cp .env.example .env   # editar ZEDAZO_AUTH_TOKEN (openssl rand -hex 32)
docker compose -f docker-compose.remote.yml --env-file .env up --build
```

- Solo **Caddy** publica `127.0.0.1:8443` (HTTPS, `tls internal`).
- API y web solo en red Docker interna (same-origin: `/` → web, `/api/*` → api).
- Frontend con rutas relativas (`NEXT_PUBLIC_API_BASE` vacío).
- Abrir `https://127.0.0.1:8443`, aceptar certificado interno, entrar con el token en `/acceso`.

### Este dispositivo (hoy)

1. Generar token y arrancar `docker-compose.remote.yml` como arriba.
2. **Preferido sin abrir el router:** Tailscale Serve/Funnel o Cloudflare Tunnel hacia `https://127.0.0.1:8443`.
3. Alternativa LAN: IP local + certificado interno (el navegador avisará).

### MiniPC (próximamente)

1. Docker + clonar repo o copiar `deploy/`.
2. Copiar `.env` (rotar token si circuló inseguro) y opcionalmente el volumen `zedazo-data`.
3. Misma orden con `docker-compose.remote.yml`.
4. Con **dominio público de la GUI:** montar [`Caddyfile.public`](../../deploy/Caddyfile.public) (Let's Encrypt) y publicar `80`/`443` solo de Caddy. Si el hostname es `zedazo.alexendros.dev`, ver [Dominio de producto](#dominio-de-producto) (landing vs GUI; same-origin).

## Rollback

1. Detener (`docker compose -f docker-compose.remote.yml down` sin `-v`).
2. Conservar volumen `zedazo-data`.
3. Usar CLI: `zedazo cribar entrada.vcf -o salida.vcf -a audit.tsv`.
4. No borrar artefactos hasta verificar recuperación.

## Coolify / VPS

- **Landing de producto** (`zedazo.alexendros.dev` sin GUI): estáticos `apps/landing/` + TLS en el proxy; ver [Dominio de producto](#dominio-de-producto).
- **GUI remota:** exponer solo tras HTTPS + `ZEDAZO_AUTH_MODE=token`. Preferir bind interno + reverse proxy / túnel. Nunca `disabled` en interfaz pública. Same-origin obligatorio (`/` → web, `/api/*` → api).

## Dominio de producto

Wordmark lowercase **`https://zedazo.alexendros.dev`** (ADR-0014). Convención de marca y pesquisa TMview: [`docs/brand.md`](../brand.md). Esta URL es la **ficha pública** del producto, no un SaaS ni la GUI de cribado.

Hoy el repo sirve una landing estática en [`apps/landing/`](../../apps/landing/index.html) (pitch, crate, GitHub, docs.rs). El registro DNS lo crea el **operador** en la zona `alexendros.dev`; el destino del CNAME no está fijado en el código.

### DNS (operador)

En el panel DNS de `alexendros.dev`:

| Tipo | Nombre (host) | Valor | TTL sugerido |
|------|----------------|-------|----------------|
| **CNAME** | `zedazo` | `<HOST_DESTINO>.` | 300–3600 s |

Sustituir `<HOST_DESTINO>` por el hostname público **real** del VPS, miniPC o proxy (Coolify / Caddy) que va a servir el sitio. Ejemplos de forma, no de valor:

- `minipc.example.net.`
- `algo.coolify.example.`
- hostname del túnel o del reverse proxy

Si el host solo tiene IP pública (sin nombre estable), usar **A** (y **AAAA** si hay IPv6) en lugar de CNAME:

| Tipo | Nombre | Valor |
|------|--------|-------|
| A | `zedazo` | `<IP_PUBLICA>` |
| AAAA | `zedazo` | `<IPV6_PUBLICA>` (si aplica) |

No crear CNAME en el apex `alexendros.dev`. No apuntar a Vercel salvo decisión explícita.

Comprobar resolución **antes** de pedir el certificado:

```bash
dig +short CNAME zedazo.alexendros.dev
# o: dig +short A zedazo.alexendros.dev
```

Debe devolver `<HOST_DESTINO>` (CNAME) o la IP del host que ejecuta Caddy.

### TLS (Let's Encrypt)

Caddy en [`Caddyfile.landing`](../../deploy/Caddyfile.landing) obtiene el certificado solo. Requisitos:

1. El CNAME/A ya apunta a este host.
2. Solo Caddy publica **80** y **443** (HTTP-01).
3. El hostname del sitio en el Caddyfile es exactamente `zedazo.alexendros.dev`.

### Arranque — solo landing

En el host destino:

```bash
cd deploy
docker compose -f docker-compose.landing.yml up -d
```

Vista local sin DNS ni TLS:

```bash
python3 -m http.server 4173 --directory apps/landing
# http://127.0.0.1:4173
```

Coolify / VPS: aplicación estática con raíz `apps/landing/`, dominio `zedazo.alexendros.dev`, TLS en el proxy. El compose anterior es la receta Docker; Coolify puede sustituir el contenedor Caddy si ya termina TLS.

### Same-origin si más adelante se sirve la GUI remota

Si `zedazo.alexendros.dev` pasa a ser la GUI/API (ADR-0016):

1. Detener `docker-compose.landing.yml` (o dejar de servir `apps/landing` en `/`).
2. Usar [`Caddyfile.public`](../../deploy/Caddyfile.public) + [`docker-compose.remote.yml`](../../deploy/docker-compose.remote.yml): `/` → web, `/api/*` → api, `ZEDAZO_AUTH_MODE=token`.
3. Publicar **solo** Caddy en 80/443. API y web quedan en la red interna.
4. El CNAME **no cambia**; cambia el Caddyfile. La landing deja de ocupar `/` (la GUI usa esas rutas). Conservar la ficha: otro host, o enlaces a GitHub / [`docs/product-card.md`](../product-card.md).

No servir landing en un origen y API en otro sin CORS explícito: el modo token espera same-origin.

### Alternativa: redirección (si la landing no se hospeda)

Si no hay host todavía, **no** crear un CNAME a un destino inventado. Opciones:

- Esperar al VPS/miniPC y seguir la tabla CNAME de arriba.
- Redirección HTTP 302/301 desde el proxy hacia `https://github.com/Iniciativas-Alexendros/zedazo` (pierde la ficha propia).
- GitHub Pages con el mismo `apps/landing/` como puente temporal; al tener host, mover el CNAME al VPS.
