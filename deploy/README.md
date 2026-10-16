# Deploy

Guía canónica: [`docs/gui/deploy.md`](../docs/gui/deploy.md).

| Archivo | Uso |
|---------|-----|
| `docker-compose.yml` | Local same-origin vía Caddy (`https://localhost:8443`, auth disabled) |
| `docker-compose.diag.yml` | Override: publica API `:8080` y web `:3000` en loopback |
| `docker-compose.remote.yml` | Remoto single-user (HTTPS + token, ADR-0016); solo Caddy publica puerto |
| `docker-compose.landing.yml` | Landing pública `zedazo.alexendros.dev` (ADR-0014 / #50) |
| `.env.example` | Plantilla de `ZEDAZO_AUTH_TOKEN` |
| `Caddyfile` | Proxy remoto (`tls internal`) |
| `Caddyfile.local` | Proxy del compose local (entrada canónica) |
| `Caddyfile.public` | GUI/API en dominio + Let's Encrypt (same-origin) |
| `Caddyfile.landing` | Estáticos `apps/landing/` + Let's Encrypt |
| `scripts/compose-smoke.sh` | Smoke: build, health, `examples/sample.vcf`, SSE, descarga, DELETE |

**URL canónica local/remoto (tls internal):** `https://localhost:8443` (también anunciada `https://127.0.0.1:8443`; si TLS falla por SNI, usar `localhost`).

Diagnóstico: `docker compose -f docker-compose.yml -f docker-compose.diag.yml up --build`.
