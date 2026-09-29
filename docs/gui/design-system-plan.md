# Plan de modernización del design system (GUI browser)

**Versión:** 0.2.0  
**Fecha:** 2026-09-12  
**Estado:** **Ejecutado** (2026-09-12). Fases 1–4 aterrizadas (pipeline DTCG + átomos + pantallas `--zed-*` + catálogo/QA, [ADR-0019](../../DECISIONS.md)). Epic [#59](https://github.com/Iniciativas-Alexendros/zedazo/issues/59) cerrable si el DoD de §7 se mantiene en CI.  
**Traza:** ADR-0015 (GUI local), ADR-0016 (remoto HTTPS+token), ADR-0019 (tokens), SPECS O10/O11, identidad «Archivo Vivo», [`docs/brand.md`](../brand.md), epic [#59](https://github.com/Iniciativas-Alexendros/zedazo/issues/59).  
**No mezclar** con CardDAV ([#48](https://github.com/Iniciativas-Alexendros/zedazo/issues/48) / ADR-0018) ni con cambios de dominio.

Este documento es la fuente de verdad del programa de design system (tokens OKLCH `--zed-*` → GUI profesional). Las cuatro fases están **ejecutadas** (2026-09-12). Cambios posteriores son mantenimiento (contraste, snapshots, copy), no un nuevo programa.

---

## 1. Contexto y estado actual

La GUI V1 (`apps/web`, hitos v0.5.0 / v0.5.1) ya no es un prototipo vacío: cubre paridad funcional CLI↔GUI (O10), auth remoto (O11) e identidad documental «Archivo Vivo». El hueco no es de producto; es de **sistema de diseño**: tokens artesanales, recetas CSS duplicadas, cobertura a11y incompleta y ausencia de pipeline que impida regresiones de contraste o de costura visual.

### 1.1 Stack vigente

| Pieza | Estado (2026-09-11) |
|-------|---------------------|
| Next.js 15 App Router + React 19 + TypeScript | `apps/web`; `output: "standalone"` |
| Tokens | Fuente DTCG `apps/web/tokens/` → `generated/` + `src/lib/design-tokens.ts` (ADR-0019). Import path de `layout.tsx` estable (`tokens.css` / `themes.css`) |
| Color | **OKLCH** en claro/oscuro (`themes.css`); sombras también OKLCH |
| Temas | `light` / `dark` / `system` (`data-theme`, `localStorage` `zedazo-theme`, script anti-FOUC) |
| Tipografía | Atkinson Hyperlegible Next + IBM Plex Mono (self-hosted `@fontsource`, sin CDN) |
| Web Awesome `@awesome.me/webawesome` ^3.12 | Dependencia y `transpilePackages`; **ningún componente `<wa-*>` montado**. `WebAwesomeProvider` es un passthrough. Bridge `--wa-*` ← `--zed-*` en `themes.css`; clases `wa-light` / `wa-dark` en `<html>` |
| Playwright + `@axe-core/playwright` | `make web-ci`; tags `wcag2a` / `wcag2aa` / `wcag22aa` en todas las rutas de §1.3 + estados sintéticos |
| Capturas | Regresión visual `e2e/visual.spec.ts` (`toHaveScreenshot`, light/dark, desktop + un viewport móvil) en `web-ci`. README: `e2e/screenshots.spec.ts` opt-in `ZEDAZO_SCREENSHOTS=1` → `docs/screenshots/` |

### 1.2 Capas CSS actuales

Orden de carga en `layout.tsx`: `reset` → `tokens` → `themes` → `typography` → `motion` → `utilities` → `components` → `globals.css`.

- **Primitivos + semánticos** viven en JSON DTCG. `tokens.css` / `themes.css` son wrappers que importan `generated/` (fase 1). Recetas de componente siguen siendo CSS humano.
- **Recetas de componente** en `components.css` (`.zed-button`, `.zed-badge`, `.zed-card`, `.zed-input`, …).
- **CSS Modules** paralelos: `styles/shell.module.css`, `forms.module.css`, `tables.module.css`, `states.module.css`. Fases 2–3 cablean colores, z-index, overlay, spacing y patrones de pantalla a `--zed-*`. Breakpoints siguen como literales equivalentes (`56.25rem` / `60rem` = `--zed-bp-md/lg`; custom props no aplican en `@media`).
- **Residuos aceptados:** anchos dinámicos (barra de stats, skeleton) siguen en inline porque son datos, no receta. APCA informativo en el catálogo; el gate de CI es WCAG 2.2 AA.

Fase 1 cubre JSON DTCG, build Node, tipos TS y check de contraste. Fase 2 añadió el catálogo mínimo; fase 4 completa [`/documentacion/ds`](../../apps/web/src/app/documentacion/ds/page.tsx) (átomos de producto + tokens + estados de ejecución).

### 1.3 Superficie de producto (rutas y bloques)

Rutas App Router: `/`, `/procesar`, `/ejecuciones`, `/ejecuciones/[jobId]`, `/auditar`, `/reglas`, `/acceso`, `/ajustes`, `/documentacion`, `/documentacion/ds` (catálogo de átomos, fase 4).

Bloques React (inventario de partida para la fase 2):

| Área | Módulos |
|------|---------|
| Shell | `app-shell`, `topbar`, `app-sidebar`, `mobile-navigation`, `statusbar`, `page-header`, `auth-gate` |
| UI atómica | `button`, `icon-button`, `badge`, `card`, `callout`, `icon`, `empty-state`, `error-state`, `loading-state`, `filter-bar`, `stat-card`, `section-heading`, `metadata-list`, `progress-stepper`, `artifact-download`, `visually-hidden` |
| Jobs | `job-status`, `job-progress`, `job-summary`, `job-timeline`, `retention-notice` |
| Contactos | `contact-table`, `contact-drawer`, `contact-field`, `contact-result-badge`, `duplicate-group`, `duplicate-evidence`, `field-provenance` |
| Auditoría | `audit-timeline`, `audit-event`, `audit-reason` |
| Reglas | `toml-editor`, `rules-source-selector`, `rules-validation`, `rules-hash` |
| Marca | `zedazo-wordmark`, `zedazo-mark`, `product-lockup` |

### 1.4 A11y y QA hoy

`e2e/a11y.spec.ts` cubre las rutas de §1.3 (incluido `/documentacion/ds`) y estados sintéticos: listado vacío, error, loading y job `screening` (`job-sintetico`). `e2e/atoms.spec.ts` fija el catálogo. `e2e/visual.spec.ts` es la regresión visual en CI.

Motion: `prefers-reduced-motion` anula animaciones/transiciones en `motion.css`; e2e comprueba que `.zed-spinner` no anima bajo `reduce`.

Contraste: `pnpm tokens:contrast` cubre pares semánticos light/dark en CI. Axe WCAG 2.2 A/AA cubre el DOM de producto.

### 1.5 Restricciones de producto que el DS no puede romper

- Single-user self-hosted; UI en **español**; wordmark lowercase **`zedazo`** (ADR-0014).
- Identidad **documental/precisa**, no dashboard SaaS genérico.
- Sin CDN de terceros por defecto (ADR-0015); fuentes e iconos locales.
- CLI = canal oficial de automatización; la GUI no duplica `completions`.
- Sin PWA, sin notificaciones de job, sin extras de GUI sin ancla SPECS/ADR.
- Paridad O10 intacta: este plan **no** cambia semántica de cribado, jobs ni artefactos.

---

## 2. Objetivo

Una GUI de browser **acabada y profesional** sobre ADR-0015/0016: misma paridad funcional, misma amenaza/modelo de auth, pero con un sistema de diseño que se siente de una pieza.

**Acabada** significa:

1. Un solo origen de tokens (DTCG) genera CSS y tipos; no hay hex/rgb/hsl de producción ni magics de spacing en componentes.
2. Claro, oscuro y sistema sin FOUC ni puentes rotos con Web Awesome (si se conserva).
3. Componentes atómicos + patrones de pantalla alineados: mismo radio, foco, densidad, copy y movimiento.
4. WCAG 2.2 AA verificable en **todas** las rutas y en los pares de color del tema.
5. Catálogo consultable (`/documentacion/ds` o Storybook) y DoD de QA (axe + contraste + regresión visual) en CI.
6. Cero costuras: ni un control «de librería» y otro «hecho a mano» con tipografía, foco o hue distintos.

Fuera de este objetivo: nuevas features de dominio, CardDAV, multi-usuario, OAuth de producto, OTel.

---

## 3. Estándares

| Norma | Cómo aplica en Zedazo |
|-------|------------------------|
| [W3C Design Tokens (DTCG)](https://tr.designtokens.org/format/) | Fuente JSON (`$value`, `$type`, `$description`). Grupos por capa. Sin inventar un schema paralelo. |
| OKLCH en producción | Color, sombras, `color-mix(in oklch, …)`. Hex solo como *fallback* generado o `theme-color` derivado del token, nunca como origen. |
| [WCAG 2.2 AA](https://www.w3.org/TR/WCAG22/) | Texto 4.5:1 (3:1 si ≥18 pt / 14 pt bold); UI no-texto y foco 3:1; target size 2.2 (24 px CSS o equivalente con spacing); focus visible; reduced motion. |
| [ARIA APG](https://www.w3.org/WAI/ARIA/apg/) | Patrones: disclosure (drawer), tabs si aparecen, combobox/listbox (filtros), grid/table (contactos), dialog (confirmaciones), switch/radio (tema). Preferir HTML nativo. |
| Tokens en capas | **Primitivo → semántico → componente**. Los componentes no apuntan a ramps crudas (`color.blue.600`); apuntan a `--zed-accent`, `--zed-fg-default`, `--zed-space-3`. |
| Motion | Duraciones/easing en tokens. Animación solo bajo `prefers-reduced-motion: no-preference`. Spinners y transiciones de shell incluidos. |
| Tipografía | Escala y pesos tokenizados; Atkinson para UI, Plex Mono para hashes/TOML/código. Sin toggle de densidad (el zoom del sistema basta; ya documentado en Ajustes). |
| i18n de UI | Copy en español; `lang="es"` en `<html>`; nombres accesibles del lockup = `zedazo`. |

**Contraste y OKLCH.** WCAG 2.2 sigue midiendo luminancia relativa sRGB, no L de OKLCH. El pipeline debe convertir a sRGB lineal antes del ratio. APCA puede usarse como métrica *informativa* en el catálogo; el gate de CI es WCAG 2.2 AA.

**No-objetivos de estándar (esta modernización):** WCAG 2.2 AAA, APCA como gate, modo alto contraste propio del SO (se respeta `prefers-contrast` si es barato; no es fase 1).

---

## 4. Arquitectura

### 4.1 Pipeline de tokens

```
apps/web/tokens/                    # fuente DTCG (humano + revisión en PR)
  primitive/                        # color ramps, space, type, radius, motion, z
  semantic/
    color.light.json
    color.dark.json
    typography.json
    elevation.json
  component/                        # button, badge, input, table, shell, …
        │
        ▼
script Node (apps/web/scripts/design-tokens/) — ADR-0019
        │  (Style Dictionary diferido; sin deps npm nuevas)
        ├─► src/design-system/generated/tokens.css     # :root primitivos + layout
        ├─► src/design-system/generated/themes.css     # [data-theme=light|dark]
        ├─► src/design-system/generated/wa-bridge.css  # --wa-* ← semánticos Zedazo
        └─► src/lib/design-tokens.ts                   # union types + hex generado
        │
        ▼
CSS humano (no generado)
  reset.css · typography.css · motion.css · utilities.css · components.css
  styles/*.module.css  →  solo var(--zed-*) / clases .zed-*
```

Reglas:

1. **`generated/` no se edita a mano.** Artefactos **commiteados** + `pnpm tokens:check` en `web-ci` (ADR-0019). No mezclar con generate-on-CI sin el check.
2. Recetas (`.zed-button`, etc.) siguen siendo CSS revisable por humanos; consumen semánticos/componente, no primitivos.
3. Nombres de custom property: `--zed-{layer}-{name}` ya usado (`--zed-accent`, `--zed-space-4`). El build debe **preservar** esos nombres públicos para no romper módulos existentes en el primer slice.
4. Temas: `data-theme="light|dark"` como hoy; `system` resuelve en boot script. El JSON semántico tiene dos sets; no hay tercer tema «system» en tokens.
5. Dep nueva (`style-dictionary` u otra) = **confirmación** AGENTS §4 en el PR de fase 1. Alternativa aceptable si el ADR corto lo justifica: script Node propio que lea DTCG y emita CSS/TS, sin Style Dictionary.

### 4.2 Capas de tokens (contrato)

| Capa | Ejemplo | Quién consume |
|------|---------|----------------|
| Primitivo | `color.accent.600` = OKLCH 0.53 0.16 257; `space.4` = 1rem | Solo semantic / build |
| Semántico | `color.fg.default`, `color.bg.canvas`, `color.accent`, `focus.ring` | CSS de receta, módulos de pantalla |
| Componente | `button.primary.bg`, `table.header.fg`, `shell.sidebar.width` | Receta de ese componente |

Alias semánticos actuales a conservar (mapeo, no rename breaking en fase 1):

`--zed-bg-{canvas,subtle,muted,raised,inverse}`, `--zed-fg-{strong,default,muted,subtle,inverse}`, `--zed-border-{subtle,default,strong}`, `--zed-accent` + hover/active/soft/on, `--zed-{success,warning,danger,info}` + hover/soft/on-soft, `--zed-focus-ring`, `--zed-selection-*`, `--zed-code-*`, escala `--zed-text-*`, `--zed-space-*`, `--zed-radius-*`, `--zed-shadow-*`, `--zed-transition-*`, `--zed-ease-standard`, layout `--zed-content-*`, `--zed-sidebar-width`, `--zed-topbar-height`, `--zed-statusbar-height`.

Añadidos en fase 1 (aditivos, sin cablear recetas): `--zed-bp-md` / `--zed-bp-lg`, `--zed-target-min`, `--zed-z-shell` / `--zed-z-drawer`, `--zed-badge-*-border`. `theme-color` y favicon usan hex **generado** desde canvas / accent (`themeColorHex`, `faviconHex`).

Añadidos en fase 2 (cableados a recetas): `--zed-overlay`, `--zed-accent-border`, `--zed-disabled-opacity`, `--zed-control-height`, `--zed-textarea-min`, `--zed-spin-duration`, `--zed-focus-offset`, `--zed-z-overlay`, `--zed-z-modal`.

### 4.3 Frontera con Web Awesome

Web Awesome **no** es la fuente de verdad visual. **Decisión fase 2 (opción A acotada):** se conserva la dependencia y el bridge `--wa-*` generado; clases `wa-light` / `wa-dark` en `<html>`. Iconos: SVG local (`Icon`). **No** se montan componentes compuestos `<wa-*>` salvo que un PR de pantalla posterior lo justifique y replique contraste OKLCH. Sin CDN. Sin CSS de tema WA que pise `--zed-*`.

No se retira `@awesome.me/webawesome` en este slice (opción B aplazada: menos superficie, pero el inventario no exige el corte).

### 4.4 Catálogo

Preferencia: **`/documentacion/ds`** dentro de `apps/web` (misma app, mismo tema, cero SaaS, copy en español). Storybook solo si el catálogo desborda una página Next (dep nueva → confirmación). El catálogo no es PWA ni sustituye `docs/gui/` canónico.

### 4.5 Lo que no entra en este pipeline

- `apps/landing/` era ficha estática fuera del pipeline (fase 1–4). Desde el contrato v1 ([`apps/web/tokens/CONTRACT.md`](../../apps/web/tokens/CONTRACT.md)) consume `tokens.css` generado: alias semánticos, mismos tonos `--zed-*`, sin paleta paralela.
- Estilos de `zedazo-api` (no hay UI).
- Tokens de marca verbal (`docs/brand.md`): el wordmark no dicta la paleta; la paleta no cambia el wordmark.

---

## 5. Entrega por fases

Cada fase = uno o más PRs **pequeños**, CI verde, **sin** CardDAV, **sin** cambios de `zedazo-core` salvo que un test de contraste viva en otro crate (no hace falta). Implementación **después** de aterrizar este plan.

### Fase 1 — Fundación

**Objetivo:** tokens como producto, no como CSS copiado.

- [x] Árbol `apps/web/tokens/` DTCG que reproduce 1:1 los `--zed-*` actuales (migración sin cambio visual de recetas).
- [x] Build Node (equivalente a Style Dictionary; ADR-0019) → CSS generado + `design-tokens.ts`.
- [x] Política de artefacto: **commit + `tokens:check`** (no generate-on-CI suelto).
- [x] Check de contraste WCAG 2.2 AA sobre pares semánticos light **y** dark; enganchado a `make web-ci` y al job Web.
- [x] `theme-color` / favicon derivados del token (hex generado; fin del hex huérfano en `layout.tsx` / `icon.tsx`).
- [x] **ADR-0019 aceptada:** script Node sin deps nuevas; WA aplazado a fase 2; `--zed-*` estables.
- [x] Tests: snapshot de OKLCH públicos + contrast checker con pares sintéticos (no PII).

**Criterio de salida:** `pnpm` build + contrast check verdes; GUI pixel-compatible con main en recetas/módulos (sin rediseño). Diff de UI accidental = fallo de la fase. *Hecho (2026-09-11).*

### Fase 2 — Componentes atómicos

**Objetivo:** un inventario y una receta por control.

- [x] Inventario (apéndice A): cada `components/ui/*` + clases `.zed-*` + CSS modules que estilan átomos.
- [x] Unificar: magics → tokens; inline de átomos → clases; `buttonClassName()` + `<Button>` como API React (`.zed-button` sigue siendo la receta para `<Link>` / `<label>`).
- [x] Contratos APG: foco visible (`--zed-focus-ring` / `--zed-focus-offset`), `disabled` (`--zed-disabled-opacity`), `aria-busy` en loading, diana ≥ 24 px (`--zed-target-min`).
- [x] Decisión A/B Web Awesome (apartado 4.3): **A acotada** — bridge generado + `wa-light`/`wa-dark`; sin `<wa-*>` compuestos.
- [x] Sin rediseño de pantallas enteras (fase 3). Átomos ajustados de forma uniforme.

**Criterio de salida:** ningún color/spacing hardcodeado en `components/ui`; axe de `/` y `/procesar` sigue a cero violaciones; catálogo mínimo (botón, badge, input, callout, card) en `/documentacion/ds`. *Hecho (2026-09-12).* Epic [#59](https://github.com/Iniciativas-Alexendros/zedazo/issues/59) — no se cierra.

### Fase 3 — Patrones y pantallas

**Objetivo:** shell y flujos sin costuras.

- [x] **Shell:** topbar, sidebar, statusbar, skip link, nav móvil, page header. `--zed-nav-indicator`, `--zed-z-skip`, `min-height` de chrome, `prefers-reduced-motion` en el drawer móvil.
- [x] **Formularios:** `/procesar`, `/reglas`, `/acceso`, `/ajustes`. `FileDropzone` compartido; fieldset/checkbox/radio con el mismo foco; `ErrorState` en login y validación.
- [x] **Tablas y fichas:** contactos, duplicados, drawer (`.zed-drawer` + `--zed-drawer-width`). Densidad `--zed-text-sm` / space tokens; `EmptyState` cuando no hay filas.
- [x] **Jobs y auditoría:** stepper, `JobProgress`, timeline, `JobRowActions`, retención; copy «ejecución» (no «job» en UI).
- [x] **Estados:** empty / loading / error / privacy callouts con la misma receta tipográfica (`states.module.css` + `compact`).
- [x] Axe ampliado a las rutas de §1.3. Catálogo `/documentacion/ds` muestra stepper y estados. Fixture running → fase 4.

Copy: español, tono archivo (preciso, no marketing). Wordmark intocable (`zedazo` en chrome; prosa «Zedazo»).

**Criterio de salida:** las nueve rutas se sienten del mismo producto en light y dark; O10 y O11 sin cambios de comportamiento. *Hecho (2026-09-12).* Epic [#59](https://github.com/Iniciativas-Alexendros/zedazo/issues/59) — no se cierra.

### Fase 4 — Acabado y QA

**Objetivo:** que no se deshaga.

- [x] Catálogo `/documentacion/ds` (sin Storybook): tokens, botón, badge, card, callout, formulario, icono, filtro, datos, stepper, ejecución, empty/error/loading.
- [x] Axe **todas** las rutas de §1.3 + estados representativos (vacío, error, loading, job running con fixture sintético).
- [x] Regresión visual: `toHaveScreenshot` en CI (chromium, light+dark, desktop; un viewport móvil del shell). `docs/screenshots/` se regenera con `ZEDAZO_SCREENSHOTS=1` (opt-in, fuera de `web-ci`).
- [x] DoD de §7 cumplido; este plan marcado **ejecutado** (2026-09-12).
- [x] README / `/documentacion` enlazan el catálogo. Sin PWA.

**Criterio de salida:** `make ci` verde con la matriz de §6; epic [#59](https://github.com/Iniciativas-Alexendros/zedazo/issues/59) cerrable. *Hecho (2026-09-12).*

---

## 6. Validación y matriz de pruebas

| Capa | Qué | Dónde | Gate |
|------|-----|-------|------|
| Tokens | JSON DTCG válido; nombres `--zed-*` públicos estables | `tokens/` + build | `web-ci` |
| Contraste | Pares semánticos light/dark (fg/bg, accent-on/accent, estado soft/on-soft, foco 3:1) | script Node en `apps/web` | `web-ci` |
| Unidad | Contrast checker; resolución de tema `system` | `node --test` o Vitest *si* se añade (dep → confirmar) | `web-ci` |
| e2e funcional | Marca, CTAs, skip link, Procesar, Ejecuciones (ya existe) | `e2e/a11y.spec.ts` parte nav | `web-ci` |
| axe 2.2 AA | Cada ruta de §1.3 + vacío/error/loading/running | `e2e/a11y.spec.ts` | `web-ci` |
| Visual | Shell + pantallas clave light/dark + móvil | `e2e/visual.spec.ts` (`toHaveScreenshot`) | `web-ci` |
| Reduced motion | Spinner no anima bajo `prefers-reduced-motion: reduce` | `e2e/a11y.spec.ts` | `web-ci` |
| Paridad O10 | Equivalencia CLI↔API | `make parity` | **no** se toca |
| Auth O11 | Login cookie / Bearer | `auth_http` | **no** se toca |
| Docs | Enlaces internos | `make docs-validate` | siempre |

Fixtures: 100 % sintéticos. Jobs de axe contra API local pueden usar el sample de `examples/sample.vcf`, nunca agendas reales.

**Rutas axe (fase 4, hechas):** `/`, `/procesar`, `/ejecuciones`, `/ejecuciones/[jobId]` (error + running sintético), `/auditar`, `/reglas`, `/acceso`, `/ajustes`, `/documentacion`, `/documentacion/ds`. Estados extra: vacío, error, loading.

**Pares de contraste mínimos (fase 1):**

- `fg.strong` / `fg.default` sobre `bg.canvas` y `bg.raised`
- `fg.muted` sobre `bg.canvas` (si no llega a 4.5:1, o se oscurece el token o se restringe a texto no esencial / 3:1 UI)
- `accent-on` sobre `accent` / `accent-hover` / `accent-active`
- `{success,warning,danger,info}-on-soft` sobre el `*-soft` correspondiente
- `focus-ring` vs `bg.canvas` ≥ 3:1
- Inverso: `fg.inverse` sobre `bg.inverse` (skip link)

---

## 7. Definition of Done

Checklist del **programa** (no de este PR de docs). Cada fase tiene su propio DoD local; esto cierra [#59](https://github.com/Iniciativas-Alexendros/zedazo/issues/59).

- [x] Fuente DTCG única; CSS/TS generados; `--zed-*` públicos documentados. *(fase 1)*
- [x] OKLCH en origen de tokens; hex solo como fallback generado (`themeColorHex` / `faviconHex`). Los modules aún pueden tener magics (fase 2).
- [x] Temas claro / oscuro / sistema sin FOUC; `theme-color` alineado al canvas (hex generado).
- [x] Contraste WCAG 2.2 AA en CI para pares semánticos. *(fase 1)*
- [x] Átomos sin color/spacing hardcodeado; un API React por control (`buttonClassName`). Modules de átomos/chrome cableados; magics de pantalla → fase 3. *(fase 2)*
- [x] Frontera WA resuelta: bridge generado + kit opcional acotado (sin `<wa-*>`). *(fase 2)*
- [x] Pantallas de §5 fase 3 sin costura light/dark; copy ES; wordmark lowercase. *(fase 3)*
- [x] Catálogo `/documentacion/ds` completo (fase 4). Sin Storybook.
- [x] Axe 2.2 AA en todas las rutas de la matriz; reduced-motion cubierto.
- [x] Regresión visual light+dark en CI; capturas README regeneradas (`docs/screenshots/`).
- [x] `make ci` verde; O10/O11 intactos; sin secretos en el diff.
- [x] PRs de implementación **no** mezclan CardDAV, dominio ni majors de parser.
- [x] Docs canónicos: este plan marcado ejecutado (2026-09-12); `docs/brand.md` sigue siendo wordmark, no paleta.

El PR de fase 4 deja el epic [#59](https://github.com/Iniciativas-Alexendros/zedazo/issues/59) **cerrable** si CI permanece verde. Residuos explícitos (no bloquean): APCA informativo, job `screenshots` opt-in separado, anchos de barra/skeleton como datos inline.

---

## 8. Fuera de alcance

| Ítem | Por qué |
|------|---------|
| GUI CardDAV / sync en pantallas | [#48](https://github.com/Iniciativas-Alexendros/zedazo/issues/48), ADR-0018; PRs de red **separados** de UI |
| Multi-usuario, colaboración, edición manual de contactos | SPECS §3; no cubierto por ADR-0016 |
| Cuentas OAuth/OIDC de *producto* Zedazo | SPECS §3; OAuth de *proveedor* Google es otro slice CardDAV, no DS |
| OpenTelemetry / OTLP | ADR-0017, post-v1.0 |
| PWA, push, toasts de job, toggle de densidad | Sin ancla SPECS/ADR |
| Rediseño de `apps/landing/` o del logomark | Marca ≠ DS de GUI |
| Cambiar CLI, OpenAPI, semántica de jobs | O10/O11 no se renegocian aquí |
| Nom 8, majors de toml/chardetng | AGENTS: no mezclar con UI |
| Publicar Storybook/Chromatic SaaS | Preferir catálogo in-app + Playwright; SaaS = confirmación |

---

## 9. Issues y slicing de PRs

Epic: [#59](https://github.com/Iniciativas-Alexendros/zedazo/issues/59) — *GUI: modernizar design system (OKLCH tokenizado → GUI profesional)*.

Los títulos siguientes son **sugeridos** para issues/PRs de implementación (no se abren en esta unidad salvo el epic). Cada PR: una fase o un slice de pantalla; base `main`; **sin** archivos CardDAV ni `crates/zedazo-core` de dominio.

| Orden | Título sugerido | Notas |
|-------|-----------------|-------|
| 1 | `GUI DS fase 1: pipeline DTCG → CSS custom properties + tipos TS` | Dep Style Dictionary (o script) + confirmación AGENTS §4 |
| 1b | `GUI DS: check CI de contraste WCAG 2.2 sobre tokens OKLCH` | Puede ir en el mismo PR que 1 si el diff cabe |
| 1c | `docs: ADR-0019 fuente de tokens GUI (DTCG + frontera Web Awesome)` | Solo si hay dep nueva o se retira WA |
| 2 | `GUI DS fase 2: inventario atómico y unificar CSS modules a --zed-*` | Hecho (2026-09-12). Sin rediseño de rutas |
| 2b | `GUI DS: retirar Web Awesome` **o** `GUI DS: bridge --wa-* generado` | Hecho como **A acotada**: bridge ya generado (fase 1); no se retira WA; no se montan `<wa-*>` |
| 3a | `GUI DS fase 3: shell (topbar, nav, statusbar, skip link)` | Hecho en el PR de fase 3 (mismo slice) |
| 3b | `GUI DS fase 3: formularios (procesar, reglas, acceso, ajustes)` | Hecho en el PR de fase 3 |
| 3c | `GUI DS fase 3: tablas, drawer de contacto y duplicados` | Hecho en el PR de fase 3 |
| 3d | `GUI DS fase 3: jobs, auditoría y estados vacíos/error` | Hecho en el PR de fase 3 |
| 4a | `GUI DS fase 4: catálogo /documentacion/ds` | Hecho (2026-09-12). Sin Storybook |
| 4b | `GUI DS fase 4: axe en todas las rutas + visual regression Playwright` | Hecho (2026-09-12). `docs/screenshots/` vía spec opt-in |

Reglas de slicing:

- Un PR de DS **no** toca `zedazo-carddav`, OpenAPI, ni reglas de cribado.
- Un PR de CardDAV **no** retoca `design-system/` ni CSS modules de chrome.
- Si un slice necesita dep nueva (Storybook, Vitest, Style Dictionary): un ADR o un párrafo en DECISIONS + confirmación humana.
- Español en título de issue/PR y commits.

---

## Referencias

- ADR-0014 marca · ADR-0015 GUI local · ADR-0016 remoto · ADR-0017 OTel (fuera) · ADR-0018 CardDAV (fuera) · ADR-0019 tokens GUI
- [`docs/brand.md`](../brand.md) · [`docs/gui/functional-parity-matrix.md`](./functional-parity-matrix.md) · [`docs/gui/threat-model.md`](./threat-model.md)
- [W3C Design Tokens Format Module](https://tr.designtokens.org/format/)
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) · [ARIA APG](https://www.w3.org/WAI/ARIA/apg/)
- Código de partida: `apps/web/src/design-system/`, `apps/web/src/components/`, `apps/web/e2e/a11y.spec.ts`

---

## Apéndice A — Inventario atómico (fase 2)

Receta visual = clase `.zed-*` en `components.css`. API React = un componente en `components/ui` (o helper `buttonClassName` para `<Link>` / `<label>`). Shell y módulos solo se listan cuando estilan el átomo o el chrome compartido.

| Átomo / primitivo | React | Receta CSS | Variantes / estados | Módulo colindante |
|-------------------|-------|------------|---------------------|-------------------|
| Botón | `Button`, `buttonClassName()` | `.zed-button`, `--primary/secondary/tertiary/danger/icon`, `--sm` | hover, active, `:focus-visible`, `:disabled`, `aria-busy` + `.zed-spinner` | — |
| Icon button | `IconButton` | `.zed-button--icon` (+ `--sm`) | `aria-label` obligatorio; mismos estados que botón | chips en `states.module.css` |
| Badge | `Badge` | `.zed-badge--neutral/info/success/warning/danger/technical` | bordes `--zed-badge-*-border` | — |
| Card | `Card` | `.zed-card--document/action/metric/outlined/interactive` | hover/foco en interactive | — |
| Callout | `Callout` | `.zed-callout--info/success/warning/danger/privacy/verification` | `role="note"` | — |
| Input / textarea | nativo + receta | `.zed-input`, `.zed-textarea`, `.zed-field` | disabled, `:focus-visible` | `forms.module.css` |
| Dropzone | nativo | `.zed-dropzone` | `data-active`, `:focus-within` | `forms.module.css` |
| Icono | `Icon` | `.zed-icon` (SVG local; no `<wa-icon>`) | `currentColor` | — |
| Empty / error / loading | `EmptyState`, `ErrorState`, `LoadingState` | `.zed-spinner` + `states.module.css` | `role="status"` / `role="alert"` | `states.module.css` |
| Filter chips | `FilterBar` | chip en `states` + `IconButton` sm | live region | `states.module.css` |
| Stat / heading / metadata / artifact | `StatCard`, `SectionHeading`, `MetadataList`, `ArtifactDownload` | `.zed-stat-card*`, `.zed-section-heading`, `.zed-metadata`, `.zed-artifact` | — | — |
| Stepper | `ProgressStepper` | `.zed-stepper__step` | `todo/current/done`, `aria-current` | — |
| Visually hidden | `VisuallyHidden` | `.zed-sr-only` | — | — |
| Page header | `PageHeader` | `.zed-page-header` | — | shell |
| Scrim / overlay | clase | `.zed-scrim`, `.zed-scrim--modal` | `--zed-overlay`, `--zed-z-overlay/modal` | `shell.module.css`, drawer |
| Nav / chrome | `AppShell`, `Topbar`, `AppSidebar` | `.navLink`, tokens de z/sidebar | `:focus-visible` global | `shell.module.css` |
| Tabla (no átomo; colindante) | `contact-table` | — | hover de fila | `tables.module.css` |

Tokens aditivos de fase 2: `--zed-overlay`, `--zed-accent-border`, `--zed-disabled-opacity`, `--zed-control-height`, `--zed-textarea-min`, `--zed-spin-duration`, `--zed-focus-offset`, `--zed-z-overlay`, `--zed-z-modal`.

Tokens aditivos de fase 3: `--zed-z-skip`, `--zed-prose-max`, `--zed-auth-width`, `--zed-drawer-width`, `--zed-filter-max`, `--zed-nav-indicator`. Recetas nuevas: `.zed-auth-layout`, `.zed-drawer`, `.zed-lockup*`, `.zed-dropzone__*`, `FileDropzone`, `JobRowActions`.
