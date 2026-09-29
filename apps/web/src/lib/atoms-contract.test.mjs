import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const WEB_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

async function readCss(rel) {
  return readFile(path.join(WEB_ROOT, rel), "utf8");
}

test("recetas de átomos no usan color-mix; bordes van a tokens", async () => {
  const css = await readCss("src/design-system/components.css");
  assert.equal(
    /color-mix\s*\(/.test(css),
    false,
    "components.css aún tiene color-mix",
  );
  for (const needle of [
    "var(--zed-badge-success-border)",
    "var(--zed-badge-warning-border)",
    "var(--zed-badge-danger-border)",
    "var(--zed-badge-info-border)",
    "var(--zed-accent-border)",
    "var(--zed-disabled-opacity)",
    "var(--zed-focus-ring)",
    "var(--zed-control-height)",
    "var(--zed-target-min)",
    "var(--zed-overlay)",
  ]) {
    assert.ok(css.includes(needle), `falta ${needle} en components.css`);
  }
});

test("spinner y foco usan tokens de motion/foco", async () => {
  const motion = await readCss("src/design-system/motion.css");
  const utilities = await readCss("src/design-system/utilities.css");
  assert.ok(motion.includes("var(--zed-spin-duration)"));
  assert.ok(motion.includes("var(--zed-accent)"));
  assert.ok(utilities.includes("var(--zed-focus-ring)"));
  assert.ok(utilities.includes("var(--zed-focus-offset)"));
});

test("módulos compartidos no pintan oklch/hex huérfanos en átomos", async () => {
  const files = [
    "src/styles/shell.module.css",
    "src/styles/forms.module.css",
    "src/styles/tables.module.css",
    "src/styles/states.module.css",
  ];
  const forbidden = /(?:oklch|#[0-9a-fA-F]{3,8}|rgb\(|hsl\()\s*\(/;
  const hex = /#[0-9a-fA-F]{3,8}\b/;
  for (const file of files) {
    const css = await readCss(file);
    assert.equal(forbidden.test(css), false, `color hardcodeado en ${file}`);
    assert.equal(hex.test(css), false, `hex en ${file}`);
    assert.ok(
      !css.includes("oklch("),
      `oklch literal en ${file} (usar var(--zed-*))`,
    );
  }
});

test("shell overlay y z-index apuntan a tokens", async () => {
  const css = await readCss("src/styles/shell.module.css");
  assert.ok(css.includes("var(--zed-overlay)"));
  assert.ok(css.includes("var(--zed-z-shell)"));
  assert.ok(css.includes("var(--zed-z-drawer)"));
  assert.ok(css.includes("var(--zed-z-overlay)"));
  assert.ok(css.includes("var(--zed-sidebar-width)"));
  assert.ok(css.includes("var(--zed-nav-indicator)"));
  assert.ok(css.includes("prefers-reduced-motion"));
});

test("patrones de pantalla usan tokens de fase 3", async () => {
  const utilities = await readCss("src/design-system/utilities.css");
  const forms = await readCss("src/styles/forms.module.css");
  const states = await readCss("src/styles/states.module.css");
  assert.ok(utilities.includes("var(--zed-z-skip)"));
  assert.ok(utilities.includes("var(--zed-auth-width)"));
  assert.ok(utilities.includes("var(--zed-drawer-width)"));
  assert.ok(utilities.includes("var(--zed-filter-max)"));
  assert.ok(forms.includes("var(--zed-control-height)"));
  assert.ok(states.includes("var(--zed-prose-max)"));
});
