# Capturas de la GUI (README)

PNG de producto para el README. **No** son el baseline de CI.

- Regresión visual en CI: `apps/web/e2e/visual.spec.ts` (`toHaveScreenshot`).
- Regenerar estas capturas (opt-in, fuera de `make web-ci`):

```bash
cd apps/web
pnpm build
ZEDAZO_SCREENSHOTS=1 pnpm test:e2e -- e2e/screenshots.spec.ts
```

Temas: `light` y `dark`. Viewport: 1440×900. Fixtures 100 % sintéticos.
