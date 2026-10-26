import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const WEB_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

async function readCss(rel) {
  return readFile(path.join(WEB_ROOT, rel), "utf8");
}

test("recetas de átomos no usan color-mix; bordes van a roles", async () => {
  const css = await readCss("src/design-system/components.css");
  assert.equal(
    /color-mix\s*\(/.test(css),
    false,
    "components.css aún tiene color-mix",
  );
  for (const needle of [
    "var(--ds-success)",
    "var(--ds-warning)",
    "var(--ds-danger)",
    "var(--ds-info)",
    "var(--ds-border)",
    "var(--ds-disabled-opacity)",
    "var(--ds-focus)",
    "var(--ds-control-height)",
    "var(--ds-target-min)",
    "var(--ds-overlay)",
  ]) {
    assert.ok(css.includes(needle), `falta ${needle} en components.css`);
  }
});

test("spinner y foco usan roles de motion/foco", async () => {
  const motion = await readCss("src/design-system/motion.css");
  const utilities = await readCss("src/design-system/utilities.css");
  assert.ok(motion.includes("var(--ds-spin-duration)"));
  assert.ok(motion.includes("var(--ds-accent)"));
  assert.ok(utilities.includes("var(--ds-focus)"));
  assert.ok(utilities.includes("var(--ds-focus-offset)"));
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
      `oklch literal en ${file} (usar var(--ds-*))`,
    );
  }
});

test("shell overlay y z-index apuntan a roles", async () => {
  const css = await readCss("src/styles/shell.module.css");
  assert.ok(css.includes("var(--ds-overlay)"));
  assert.ok(css.includes("var(--ds-z-shell)"));
  assert.ok(css.includes("var(--ds-z-drawer)"));
  assert.ok(css.includes("var(--ds-z-overlay)"));
  assert.ok(css.includes("var(--ds-sidebar-width)"));
  assert.ok(css.includes("var(--ds-nav-indicator)"));
  assert.ok(css.includes("prefers-reduced-motion"));
});

test("patrones de pantalla usan roles de fase 3", async () => {
  const utilities = await readCss("src/design-system/utilities.css");
  const forms = await readCss("src/styles/forms.module.css");
  const states = await readCss("src/styles/states.module.css");
  assert.ok(utilities.includes("var(--ds-z-skip)"));
  assert.ok(utilities.includes("var(--ds-auth-width)"));
  assert.ok(utilities.includes("var(--ds-drawer-width)"));
  assert.ok(utilities.includes("var(--ds-filter-max)"));
  assert.ok(forms.includes("var(--ds-control-height)"));
  assert.ok(states.includes("var(--ds-prose-max)"));
});
