# Deploy

Guía canónica: [`docs/gui/deploy.md`](../docs/gui/deploy.md).

| Archivo | Uso |
|---------|-----|
| `docker-compose.yml` | Local loopback (auth disabled) |
| `docker-compose.remote.yml` | Remoto single-user (HTTPS + token, ADR-0016) |
| `docker-compose.landing.yml` | Landing pública `zedazo.alexendros.dev` (ADR-0014 / #50) |
| `.env.example` | Plantilla de `ZEDAZO_AUTH_TOKEN` |
| `Caddyfile` | Proxy remoto (`tls internal`) |
| `Caddyfile.local` | Profile `proxy` del compose local |
| `Caddyfile.public` | GUI/API en dominio + Let's Encrypt (same-origin) |
| `Caddyfile.landing` | Estáticos `apps/landing/` + Let's Encrypt |
