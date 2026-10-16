# Checklist de accesibilidad (PR-08)

**Fecha:** 2026-10-16  
**Automatizado:** `apps/web/e2e/a11y.spec.ts` (axe wcag2a/aa/22aa + teclado).  
**Motor declarado:** Chromium en CI (`playwright.config.ts`). No se añaden Firefox/WebKit sin ancla en SPECS.

## Revisión manual registrada

| Ítem | Resultado | Evidencia |
|------|-----------|-----------|
| Skip link visible al Tab | OK | test teclado en a11y.spec |
| Foco visible en enlaces de nav | OK | focus + Enter → `/procesar` |
| Heading h1 en rutas principales | OK | specs de navegación básica |
| Alertas en estado error | OK | fixtures `?fixture=error` + axe |
| `prefers-reduced-motion` | OK | spinner sin animación |
| Mensajes en español | OK | copy UI Archivo Vivo |

## Capturas visuales

- `e2e/visual.spec.ts` + snapshots Linux Chromium.
- Tolerancia `maxDiffPixelRatio: 0.06` por AA tipográfica del stack del SO (comentado en config).
- Estabilización: `document.fonts.ready` + `waitForShellSettled` antes del screenshot.
- No bajar tolerancias hasta que CI y host local usen el mismo navegador/fuentes.
