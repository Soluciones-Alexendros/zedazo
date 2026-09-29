#!/usr/bin/env node
/**
 * DTCG (apps/web/tokens/) → CSS custom properties + tipos TS.
 * Sin Style Dictionary: passthrough OKLCH, cero deps npm nuevas (ADR-0019).
 *
 *   node scripts/design-tokens/build.mjs
 *   node scripts/design-tokens/build.mjs --check
 */

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import {
  WEB_ROOT,
  formatCssValue,
  loadTokenGraph,
  publicTokensByTheme,
  resolveInTheme,
  resolveToken,
} from "./load.mjs";
import { oklchToHex, parseOklchValue } from "./oklch.mjs";

const GENERATED_DIR = path.join(WEB_ROOT, "src/design-system/generated");
const TS_OUT = path.join(WEB_ROOT, "src/lib/design-tokens.ts");
const LANDING_CSS = path.join(WEB_ROOT, "../landing/tokens.css");
const CHECK = process.argv.includes("--check");

const HEADER = `/**
 * GENERATED — no editar a mano.
 * Fuente: apps/web/tokens/ (DTCG). Regenerar: pnpm tokens:build
 */`;

const COMMON_ORDER = [
  "--zed-font-ui",
  "--zed-font-mono",
  "--zed-text-xs",
  "--zed-text-sm",
  "--zed-text-base",
  "--zed-text-md",
  "--zed-text-lg",
  "--zed-text-xl",
  "--zed-text-2xl",
  "--zed-text-3xl",
  "--zed-leading-tight",
  "--zed-leading-snug",
  "--zed-leading-normal",
  "--zed-leading-relaxed",
  "--zed-weight-regular",
  "--zed-weight-medium",
  "--zed-weight-semibold",
  "--zed-weight-bold",
  "--zed-space-1",
  "--zed-space-2",
  "--zed-space-3",
  "--zed-space-4",
  "--zed-space-5",
  "--zed-space-6",
  "--zed-space-8",
  "--zed-space-10",
  "--zed-space-12",
  "--zed-space-16",
  "--zed-space-20",
  "--zed-space-24",
  "--zed-radius-xs",
  "--zed-radius-sm",
  "--zed-radius-md",
  "--zed-radius-lg",
  "--zed-radius-xl",
  "--zed-radius-pill",
  "--zed-border-width",
  "--zed-focus-width",
  "--zed-focus-offset",
  "--zed-shadow-xs",
  "--zed-shadow-sm",
  "--zed-shadow-md",
  "--zed-content-readable",
  "--zed-content-wide",
  "--zed-sidebar-width",
  "--zed-topbar-height",
  "--zed-statusbar-height",
  "--zed-transition-fast",
  "--zed-transition-base",
  "--zed-transition-slow",
  "--zed-ease-standard",
  "--zed-target-min",
  "--zed-bp-md",
  "--zed-bp-lg",
  "--zed-z-shell",
  "--zed-z-drawer",
  "--zed-z-overlay",
  "--zed-z-modal",
  "--zed-z-skip",
  "--zed-prose-max",
  "--zed-auth-width",
  "--zed-drawer-width",
  "--zed-filter-max",
  "--zed-nav-indicator",
  "--zed-control-height",
  "--zed-disabled-opacity",
  "--zed-textarea-min",
  "--zed-spin-duration",
];

const THEME_ORDER = [
  "--zed-bg-canvas",
  "--zed-bg-subtle",
  "--zed-bg-muted",
  "--zed-bg-raised",
  "--zed-bg-inverse",
  "--zed-overlay",
  "--zed-fg-strong",
  "--zed-fg-default",
  "--zed-fg-muted",
  "--zed-fg-subtle",
  "--zed-fg-inverse",
  "--zed-border-subtle",
  "--zed-border-default",
  "--zed-border-strong",
  "--zed-accent",
  "--zed-accent-hover",
  "--zed-accent-active",
  "--zed-accent-soft",
  "--zed-accent-on",
  "--zed-accent-border",
  "--zed-success",
  "--zed-success-hover",
  "--zed-success-soft",
  "--zed-success-on-soft",
  "--zed-warning",
  "--zed-warning-hover",
  "--zed-warning-soft",
  "--zed-warning-on-soft",
  "--zed-danger",
  "--zed-danger-hover",
  "--zed-danger-soft",
  "--zed-danger-on-soft",
  "--zed-info",
  "--zed-info-hover",
  "--zed-info-soft",
  "--zed-info-on-soft",
  "--zed-focus-ring",
  "--zed-selection-bg",
  "--zed-selection-fg",
  "--zed-code-bg",
  "--zed-code-fg",
  "--zed-badge-success-border",
  "--zed-badge-warning-border",
  "--zed-badge-danger-border",
  "--zed-badge-info-border",
];

const WA_BRIDGE = [
  ["--wa-color-brand-fill-loud", "var(--zed-accent)"],
  ["--wa-color-brand-fill-normal", "var(--zed-accent)"],
  ["--wa-color-brand-fill-quiet", "var(--zed-accent-soft)"],
  ["--wa-color-brand-on-loud", "var(--zed-accent-on)"],
  ["--wa-color-brand-on-normal", "var(--zed-accent-on)"],
  ["--wa-color-brand-on-quiet", "var(--zed-accent-active)"],
  ["--wa-color-success-fill-loud", "var(--zed-success)"],
  ["--wa-color-success-fill-quiet", "var(--zed-success-soft)"],
  ["--wa-color-success-on-quiet", "var(--zed-success-on-soft)"],
  ["--wa-color-warning-fill-loud", "var(--zed-warning)"],
  ["--wa-color-warning-fill-quiet", "var(--zed-warning-soft)"],
  ["--wa-color-warning-on-quiet", "var(--zed-warning-on-soft)"],
  ["--wa-color-danger-fill-loud", "var(--zed-danger)"],
  ["--wa-color-danger-fill-quiet", "var(--zed-danger-soft)"],
  ["--wa-color-danger-on-quiet", "var(--zed-danger-on-soft)"],
  ["--wa-focus-ring-color", "var(--zed-focus-ring)"],
  ["--wa-font-family-body", "var(--zed-font-ui)"],
  ["--wa-font-family-code", "var(--zed-font-mono)"],
];

export async function renderDesignTokens() {
  const graph = await loadTokenGraph();
  const groups = publicTokensByTheme(graph);

  const commonDecls = emitDecls(graph, groups.common, COMMON_ORDER, "common");
  const lightDecls = emitDecls(graph, groups.light, THEME_ORDER, "light");
  const darkDecls = emitDecls(graph, groups.dark, THEME_ORDER, "dark");

  const tokensCss = `${HEADER}

:root {
  color-scheme: light dark;
${commonDecls}
}
`;

  const themesCss = `${HEADER}

:root,
[data-theme="light"] {
${lightDecls}
}

[data-theme="dark"] {
${darkDecls}
}
`;

  const waBridgeCss = `${HEADER}

/* Bridge Web Awesome ← semánticos Zedazo (WA no es fuente de verdad). */
:root,
[data-theme="light"],
[data-theme="dark"] {
${WA_BRIDGE.map(([name, value]) => `  ${name}: ${value};`).join("\n")}
}
`;

  const canvasLight = colorHex(graph, "color.bg.canvas", "light");
  const canvasDark = colorHex(graph, "color.bg.canvas", "dark");
  const accentLight = colorHex(graph, "color.accent.default", "light");
  const accentOnLight = colorHex(graph, "color.accent.on", "light");

  const cssVars = collectCssVars(groups);
  const ts = emitTs(cssVars, {
    light: canvasLight,
    dark: canvasDark,
    faviconBg: accentLight,
    faviconFg: accentOnLight,
  });

  const landingCss = renderLandingCss(graph, groups.common);

  return {
    files: {
      [path.join(GENERATED_DIR, "tokens.css")]: tokensCss,
      [path.join(GENERATED_DIR, "themes.css")]: themesCss,
      [path.join(GENERATED_DIR, "wa-bridge.css")]: waBridgeCss,
      [TS_OUT]: ts,
      [LANDING_CSS]: landingCss,
    },
    hex: {
      themeColor: { light: canvasLight, dark: canvasDark },
      favicon: { background: accentLight, foreground: accentOnLight },
    },
  };
}

function emitDecls(graph, tokens, preferredOrder, theme) {
  const byVar = new Map();
  for (const token of tokens) {
    const resolved = resolveToken(graph, token.path, new Set(), theme);
    const css = formatCssValue(resolved.value, resolved.type ?? token.type);
    byVar.set(token.cssVar, css);
  }

  const ordered = [
    ...preferredOrder.filter((name) => byVar.has(name)),
    ...[...byVar.keys()].filter((name) => !preferredOrder.includes(name)).sort(),
  ];

  return ordered.map((name) => `  ${name}: ${byVar.get(name)};`).join("\n");
}

function colorHex(graph, tokenPath, theme) {
  const resolved = resolveInTheme(graph, tokenPath, theme);
  const oklch = parseOklchValue(resolved.value);
  if (!oklch) {
    throw new Error(`Token ${tokenPath} (${theme}) no es OKLCH estructurado`);
  }
  return oklchToHex(oklch);
}

function collectCssVars(groups) {
  const names = new Set();
  for (const list of Object.values(groups)) {
    for (const token of list) names.add(token.cssVar);
  }
  return [...names].sort();
}

function toCamel(cssVar) {
  const body = cssVar.startsWith("--zed-")
    ? cssVar.slice("--zed-".length)
    : cssVar.replace(/^--+/, "");
  const camel = body.replace(/-([a-z0-9])/g, (_, ch) => ch.toUpperCase());
  if (!/^[A-Za-z_]/.test(camel)) {
    throw new Error(`identificador TS inválido para ${cssVar}`);
  }
  return camel;
}

function oklchFunctionsToHex(css) {
  return css.replace(
    /oklch\(\s*([0-9.]+)\s+([0-9.]+)\s+([0-9.]+)(?:\s*\/\s*([0-9.]+))?\s*\)/g,
    (_, l, c, h, alpha) => {
      const hex = oklchToHex({ l: Number(l), c: Number(c), h: Number(h) });
      if (alpha == null) return hex;
      const r = Number.parseInt(hex.slice(1, 3), 16);
      const g = Number.parseInt(hex.slice(3, 5), 16);
      const b = Number.parseInt(hex.slice(5, 7), 16);
      return `rgb(${r} ${g} ${b} / ${alpha})`;
    },
  );
}

function decls(pairs) {
  return pairs.map(([name, value]) => `  ${name}: ${value};`).join("\n");
}

/**
 * CSS autónomo para apps/landing. Los alias de contrato se resuelven a los
 * mismos OKLCH de marca (claro/oscuro). Hex solo dentro de @supports not.
 */
export function renderLandingCss(graph, commonTokens) {
  const plain = [];
  const colorish = [];
  /** @type {{ cssVar: string, aliasOf: string }[]} */
  const aliases = [];

  for (const token of commonTokens) {
    if (token.aliasOf) {
      aliases.push({ cssVar: token.cssVar, aliasOf: token.aliasOf });
      continue;
    }
    const resolved = resolveToken(graph, token.path, new Set(), "common");
    const css = formatCssValue(resolved.value, resolved.type ?? token.type);
    if (css.includes("oklch(")) {
      colorish.push([token.cssVar, css, oklchFunctionsToHex(css)]);
    } else {
      plain.push([token.cssVar, css]);
    }
  }

  const orderedPlain = [
    ...COMMON_ORDER.filter((name) => plain.some(([n]) => n === name)).map((name) =>
      plain.find(([n]) => n === name),
    ),
    ...plain.filter(([name]) => !COMMON_ORDER.includes(name)).sort((a, b) => a[0].localeCompare(b[0])),
  ];

  const light = aliases.map((alias) => {
    const resolved = resolveInTheme(graph, alias.aliasOf, "light");
    return {
      name: alias.cssVar,
      oklch: formatCssValue(resolved.value, resolved.type),
      hex: colorHex(graph, alias.aliasOf, "light"),
    };
  });
  const dark = aliases.map((alias) => {
    const resolved = resolveInTheme(graph, alias.aliasOf, "dark");
    return {
      name: alias.cssVar,
      oklch: formatCssValue(resolved.value, resolved.type),
      hex: colorHex(graph, alias.aliasOf, "dark"),
    };
  });

  const lightOklch = [
    ...light.map((row) => [row.name, row.oklch]),
    ...colorish.map(([name, oklch]) => [name, oklch]),
  ];
  const lightHex = [
    ...light.map((row) => [row.name, row.hex]),
    ...colorish.map(([name, , hex]) => [name, hex]),
  ];
  const darkOklch = dark.map((row) => [row.name, row.oklch]);
  const darkHex = dark.map((row) => [row.name, row.hex]);

  const block = (pairs) => (pairs.length ? `${decls(pairs)}\n` : "");

  return `${HEADER}

/* Landing estática. Alias del contrato v1 → paleta zedazo. No editar a mano. */

:root {
  color-scheme: light dark;
${decls(orderedPlain)}
}

@supports not (color: oklch(0 0 0)) {
  :root {
${block(lightHex)}  }
  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) {
      color-scheme: dark;
${block(darkHex)}    }
  }
  :root[data-theme="dark"] {
    color-scheme: dark;
${block(darkHex)}  }
  :root[data-theme="light"] {
    color-scheme: light;
${block(lightHex)}  }
}

@supports (color: oklch(0 0 0)) {
  :root {
${block(lightOklch)}  }
  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) {
      color-scheme: dark;
${block(darkOklch)}    }
  }
  :root[data-theme="dark"] {
    color-scheme: dark;
${block(darkOklch)}  }
  :root[data-theme="light"] {
    color-scheme: light;
${block(lightOklch)}  }
}
`;
}

function emitTs(cssVars, hex) {
  const entries = cssVars.map((name) => `  ${toCamel(name)}: "${name}",`).join("\n");
  return `${HEADER}

export const zedCssVars = {
${entries}
} as const;

export type ZedCssVar = (typeof zedCssVars)[keyof typeof zedCssVars];
export type ZedCssVarName = keyof typeof zedCssVars;

/** Hex generado desde OKLCH (theme-color / favicon). No es origen. */
export const themeColorHex = {
  light: "${hex.light}",
  dark: "${hex.dark}",
} as const;

/** Hex generado desde accent / accent-on (tema claro) para el favicon. */
export const faviconHex = {
  background: "${hex.faviconBg}",
  foreground: "${hex.faviconFg}",
} as const;
`;
}

async function writeOrCheck(files) {
  let dirty = false;
  for (const [file, contents] of Object.entries(files)) {
    if (CHECK) {
      let current = null;
      try {
        current = await readFile(file, "utf8");
      } catch {
        current = null;
      }
      if (current !== contents) {
        dirty = true;
        console.error(`desactualizado: ${path.relative(WEB_ROOT, file)}`);
      }
    } else {
      await mkdir(path.dirname(file), { recursive: true });
      await writeFile(file, contents, "utf8");
      console.log(`wrote ${path.relative(WEB_ROOT, file)}`);
    }
  }
  if (CHECK && dirty) {
    console.error("Ejecuta: pnpm tokens:build");
    process.exitCode = 1;
  } else if (CHECK) {
    console.log("tokens generados al día");
  }
}

const isMain = Boolean(process.argv[1]) && import.meta.url === pathToFileURL(process.argv[1]).href;

if (isMain) {
  const { files } = await renderDesignTokens();
  await writeOrCheck(files);
}
