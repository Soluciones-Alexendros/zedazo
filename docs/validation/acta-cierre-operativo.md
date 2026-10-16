# Acta de cierre operativo Zedazo

**Fecha:** 2026-10-16  
**SHA de esta rama (tip):** `fc3a1ecbc27466f942f6c99566ab6744da40f596` (actualizar al merge en `main`)  
**Alcance:** evidencias del plan de cierre (CLI, Docker same-origin, remoto single-user). Sin ampliar producto.

## Lista de aceptación (con evidencia)

| Criterio | Evidencia | Estado |
|----------|-----------|--------|
| Línea base reproducible (versiones, `make ci`, escrituras, logs v0.5.1) | PR-01 · `docs/validation/baseline-2026-10-16.md` · `release-v0.5.1.md` | Hecho |
| Docker same-origin, Caddy entrada, URL única | PR-02 · `deploy/docker-compose.yml` · smoke local | Hecho |
| Retención / reconcile / wipe / pending deletes | PR-03 · `retention.md` · `retention_restart` tests | Hecho |
| Release exige assets + checksums + SBOM | PR-04 · `release.yml` · diagnóstico v0.5.1 | Hecho (reparar assets del tag = seguimiento) |
| Remoto solo proxy, matriz 401, cookie, volúmenes Caddy | PR-05 · `auth_http` · `deploy.md` | Hecho |
| Paridad CLI/API + CardDAV mocks | PR-06 · `equivalence_http` · matriz · carddav tests | Hecho |
| `pnpm test` canónico, E2E API real, Compose smoke | PR-07 · `ci.yml` · `compose-smoke.sh` (local OK) | Hecho |
| A11y + capturas estables Chromium | PR-08 · `a11y.spec` · checklist | Hecho |
| Docs operación + esta acta | PR-09 | Hecho |

## Comandos de verificación locales (sesión cierre)

```bash
# Paridad
cargo test -p zedazo-api --test equivalence_http --all-features

# Web unit
pnpm --dir apps/web test

# Compose smoke
bash deploy/scripts/compose-smoke.sh
# → OK compose-smoke (https://localhost:8443 …)
```

## Limitaciones conocidas (no bloquean el acta; quedan escritas)

1. **Historia lineal en `main`:** auto-merge `--merge` puede rechazarse; decisión humana (ver `branch-protection-gap.md`).
2. **Checks no requeridos:** Web, Parity, Deny, Compose smoke, E2E real-api no están en branch protection.
3. **Ejecución nativa macOS/Windows** de binarios de release: CI valida presencia/checksum; no lanza el binario en esos OS.
4. **Borrado no forense:** aplicación + FS; backups opt-in del operador pueden conservar datos ya borrados.
5. **TTL horario:** retención por barrido, no al segundo exacto.
6. **Logout:** borra cookie; no revoca token estático copiado (rotar `ZEDAZO_AUTH_TOKEN`).
7. **Tag `v0.5.1`:** existe; assets pueden estar incompletos por filtrado histórico de `GITHUB_OUTPUT` (ver `release-v0.5.1.md`). No se mueve el tag desde `main` actual en este cierre.
8. **TLS smoke:** preferir `https://localhost:8443`; `127.0.0.1` puede fallar SNI con `tls internal`.

## PRs del programa

| PR | Título |
|----|--------|
| #73 | línea base |
| #74 | Docker same-origin |
| #75 | retención |
| #76 | integridad release |
| #77 | remoto/seguridad |
| #78 | paridad O10 |
| #79 | CI E2E / Compose |
| #80 | a11y |
| (este) | docs + acta |
