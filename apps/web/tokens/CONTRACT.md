# Contrato de tokens DTCG OKLCH v1

**Versión:** 1.0  
**Fecha:** 2026-09-24  
**Traza:** ADR-0019, [`docs/gui/design-system-plan.md`](../../../docs/gui/design-system-plan.md), [`docs/brand.md`](../../../docs/brand.md)

La paleta de marca no cambia. Este documento mapea el contrato semántico v1 (`bg`, `text`, `border`, `action`, `feedback`) al árbol DTCG que ya alimenta `--zed-*`. No hay un segundo sistema visual.

## Forma canónica y archivos reales

| Contrato v1 | Dónde vive en zedazo | Notas |
| --- | --- | --- |
| `primitive.color` brand | `primitive/color.json` → `color.accent.*` | Acento azul de marca. No se renombra la rampa. |
| `primitive.color` neutral | `primitive/color.json` → `color.paper.*`, `color.ink.*` | Papel (claro) y tinta (oscuro). |
| `primitive.color` success / warning / danger | `primitive/color.json` → `color.success`, `color.warning`, `color.danger` | |
| `primitive.color` focus | `primitive/color.json` → `color.accent.focus-light` / `focus-dark` | Semántico: `color.focus.ring`. |
| `semantic.color` light / dark | `semantic/color.light.json`, `semantic/color.dark.json` | Nombres públicos históricos: `bg`, `fg`, `border`, `accent`, estados. |
| `semantic.color` bg / text / border / action / feedback | `contract/semantic.alias.json` | Alias `var(--zed-*)`. Claro y oscuro salen del tema, no de un JSON duplicado. |
| `primitive.dimension` space | `primitive/space.json` → `--zed-space-*` | |
| `primitive.dimension` radius | `primitive/radius.json` → `--zed-radius-*` | |
| `primitive.dimension` shadow | `primitive/elevation.json` → `--zed-shadow-*` | |
| `primitive.dimension` motion | `primitive/motion.json` → `--zed-transition-*`, `--zed-ease-standard` | |
| `primitive.dimension` z | `primitive/layout.json` → `--zed-z-*` | |
| `primitive.dimension` breakpoint | `primitive/layout.json` → `--zed-bp-md`, `--zed-bp-lg` | |
| `semantic.typography` | `semantic/typography.json` + `primitive/typography.json` | `--zed-font-*`, `--zed-text-*`, pesos y leading. |
| Componente button / card / input | `contract/component.alias.json` | Apuntan a `--color-*`, no a rampas. Las recetas `.zed-*` de la GUI siguen en CSS humano. |

El valor de color en origen sigue siendo OKLCH estructurado (`colorSpace: "oklch"`, `components: [L, C, H]`). Los alias de contrato no copian componentes: referencian la custom property de marca.

## Variables CSS del contrato

Declaradas en `:root` de `src/design-system/generated/tokens.css` como `var(--zed-*)`. Cambian con `[data-theme]` porque el destino ya es temático.

| Grupo | Variable | Destino de marca |
| --- | --- | --- |
| bg | `--color-bg-base` | `--zed-bg-canvas` |
| bg | `--color-bg-surface` | `--zed-bg-raised` |
| bg | `--color-bg-muted` | `--zed-bg-muted` |
| bg | `--color-bg-subtle` | `--zed-bg-subtle` |
| bg | `--color-bg-code` | `--zed-code-bg` |
| text | `--color-text-base` | `--zed-fg-default` |
| text | `--color-text-strong` | `--zed-fg-strong` |
| text | `--color-text-muted` | `--zed-fg-muted` |
| text | `--color-text-link` | `--zed-accent` |
| text | `--color-text-on-code` | `--zed-code-fg` |
| border | `--color-border-base` | `--zed-border-default` |
| border | `--color-border-strong` | `--zed-border-strong` |
| border | `--color-border-subtle` | `--zed-border-subtle` |
| action | `--color-action-primary-bg` | `--zed-accent` |
| action | `--color-action-primary-bg-hover` | `--zed-accent-hover` |
| action | `--color-action-on-primary` | `--zed-accent-on` |
| action | `--color-action-secondary-bg` | `--zed-bg-raised` |
| action | `--color-action-on-secondary` | `--zed-fg-strong` |
| action | `--color-action-focus-ring` | `--zed-focus-ring` |
| feedback | `--color-feedback-success-bg` / `--color-feedback-success-text` | `--zed-success-soft` / `--zed-success-on-soft` |
| feedback | `--color-feedback-warning-bg` / `--color-feedback-warning-text` | `--zed-warning-soft` / `--zed-warning-on-soft` |
| feedback | `--color-feedback-danger-bg` / `--color-feedback-danger-text` | `--zed-danger-soft` / `--zed-danger-on-soft` |
| button | `--color-button-primary-*`, `--color-button-secondary-*` | action / border |
| card | `--color-card-bg`, `--color-card-fg`, `--color-card-border` | bg / text / border |
| input | `--color-input-bg`, `--color-input-fg`, `--color-input-border`, `--color-input-focus` | bg / text / border / action |

`fg` de la GUI equivale a `text` del contrato. `accent` equivale a `action` primario. El feedback usa el par soft / on-soft, que ya cumple 4.5:1 en el gate.

## Landing

`pnpm tokens:build` escribe [`apps/landing/tokens.css`](../../landing/tokens.css). Ahí los alias se resuelven al OKLCH de marca en claro, en `prefers-color-scheme: dark` y en `[data-theme]`. El hex solo aparece en `@supports not (color: oklch(0 0 0))`. `apps/landing/index.html` no pinta con hex ni `oklch()` sueltos.

## Contraste

`pnpm tokens:contrast` no se relaja. Los pares de `contrast-pairs.json` siguen midiendo los semánticos de marca (texto ≥ 4.5:1, UI ≥ 3:1, claro y oscuro). Los alias no introducen combinaciones nuevas.
