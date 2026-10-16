# Hueco de branch protection (cierre operativo)

**Fecha:** 2026-10-16  
**Fuente:** [`docs/branch-protection.json`](../branch-protection.json)

## Checks requeridos hoy en `main`

Los jobs Rust clásicos (p. ej. Format, Clippy, Test, Check, Docs validate, Health). **No** están en la lista requerida:

- Web (apps/web)
- Parity CLI↔API (O10)
- Deny (cargo-deny)
- Web E2E API real
- Compose smoke (Docker)

Cambiar la protección de `main` queda **fuera** de este cierre: hace falta confirmación humana porque `required_linear_history` y el code owner no se tocan aquí.

## Historia lineal

`required_linear_history: true` impide merge commits. El cierre operativo abre PRs con auto-merge `--merge`; si GitHub rechaza el merge por historia lineal, hay que decidir (rebase/squash con confirmación, o relajar la protección).
