# Tokens de diseño (DTCG)

Fuente de verdad de la GUI browser. Contrato: [ADR-0019](../../../DECISIONS.md), plan en [`docs/gui/design-system-plan.md`](../../../docs/gui/design-system-plan.md).

```
tokens/                    # humano + revisión en PR
  primitive/               # ramps OKLCH, space, type, radius, motion, z
  semantic/                # color.light / color.dark + alias de tipo/elevación
  component/               # control (fase 2) + pattern (fase 3: panel/filtro/nav)
  contrast-pairs.json      # pares WCAG 2.2 AA
        │
        ▼
scripts/design-tokens/     # Node 22, sin deps npm nuevas
        │
        ├─ src/design-system/generated/*.css
        └─ src/lib/design-tokens.ts
```

## Comandos

```bash
pnpm tokens:build      # regenera CSS + tipos
pnpm tokens:check      # falla si el artefacto committed no coincide
pnpm tokens:contrast   # gate WCAG 2.2 AA (sRGB, no L de OKLCH)
```

## Reglas

- Color de origen: **OKLCH estructurado** (`colorSpace: "oklch"`). Sin hex/rgb/hsl en esta carpeta.
- Hex solo como *fallback generado* (`themeColorHex`, `faviconHex`) para `theme-color` y el favicon (Satori no pinta OKLCH).
- Custom properties públicas `--zed-*` se declaran en `$extensions.com.zedazo.cssVar`. No renombrar en fase 1.
- Contrato v1 (`bg` / `text` / `border` / `action` / `feedback`): alias en `contract/` documentados en [CONTRACT.md](./CONTRACT.md). La landing consume `apps/landing/tokens.css` generado por este build.
- `generated/` y `src/lib/design-tokens.ts` se **commitean** y se verifican en `web-ci`.
- Web Awesome no es fuente de verdad; el bridge `--wa-*` se genera desde semánticos Zedazo.
