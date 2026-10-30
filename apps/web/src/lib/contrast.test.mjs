import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const CSS_PATH = process.env.DS_REFERENCES_CSS;
if (!CSS_PATH) {
  throw new Error(
    "DS_REFERENCES_CSS no está definido. Pásalo como DS_REFERENCES_CSS=ruta/al/design-references.css",
  );
}

function hexToRgb(hex) {
  const value = hex.replace("#", "");
  return {
    r: parseInt(value.slice(0, 2), 16),
    g: parseInt(value.slice(2, 4), 16),
    b: parseInt(value.slice(4, 6), 16),
  };
}

function channel(c) {
  const s = c / 255;
  return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
}

function relativeLuminance(hex) {
  const { r, g, b } = hexToRgb(hex);
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function contrastRatio(a, b) {
  const la = relativeLuminance(a);
  const lb = relativeLuminance(b);
  const [hi, lo] = la >= lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}

function extractVars(css, blockMarker) {
  const start = css.indexOf(blockMarker);
  if (start === -1) throw new Error(`bloque no encontrado: ${blockMarker}`);
  const open = css.indexOf("{", start);
  const close = css.indexOf("}", open);
  const body = css.slice(open + 1, close);
  const vars = {};
  for (const line of body.split("\n")) {
    const m = line.match(/--ds-([a-z0-9-]+):\s*(#[0-9a-fA-F]{6})\s*;/);
    if (m) vars[m[1]] = m[2];
  }
  return vars;
}

test("contraste sintético: negro sobre blanco ≈ 21:1", () => {
  assert.ok(Math.abs(contrastRatio("#000000", "#ffffff") - 21) < 0.05);
});

test("contraste sintético: mismo gris no alcanza AA texto", () => {
  assert.ok(contrastRatio("#808080", "#808080") < 1.05);
});

test("contraste sintético: par que falla 4.5:1", () => {
  assert.ok(contrastRatio("#999999", "#bbbbbb") < 4.5);
});

const css = await readFile(CSS_PATH, "utf8");
const light = extractVars(css, '[data-theme="light"]');
const dark = extractVars(css, '[data-theme="dark"]');

test("texto sobre canvas cumple AA (≥ 4.5) en claro", () => {
  assert.ok(contrastRatio(light.text, light["bg-canvas"]) >= 4.5);
});

test("texto sobre canvas cumple AA (≥ 4.5) en oscuro", () => {
  assert.ok(contrastRatio(dark.text, dark["bg-canvas"]) >= 4.5);
});

test("texto/raised y texto-muted/canvas cumplen AA (≥ 4.5)", () => {
  for (const theme of [light, dark]) {
    assert.ok(contrastRatio(theme.text, theme["bg-raised"]) >= 4.5);
    assert.ok(contrastRatio(theme["text-muted"], theme["bg-canvas"]) >= 4.5);
  }
});

test("acento/canvas y on-accent/accent cumplen AA (≥ 4.5)", () => {
  for (const theme of [light, dark]) {
    assert.ok(contrastRatio(theme.accent, theme["bg-canvas"]) >= 4.5);
    assert.ok(contrastRatio(theme["on-accent"], theme.accent) >= 4.5);
  }
});

test("success, warning, danger e info sobre canvas ≥ 4.5", () => {
  for (const theme of [light, dark]) {
    for (const role of ["success", "warning", "danger", "info"]) {
      assert.ok(
        contrastRatio(theme[role], theme["bg-canvas"]) >= 4.5,
        `${role} en el tema no alcanza 4.5`,
      );
    }
  }
});

test("foco/canvas, foco/raised y borde/canvas ≥ 3", () => {
  for (const theme of [light, dark]) {
    assert.ok(contrastRatio(theme.focus, theme["bg-canvas"]) >= 3);
    assert.ok(contrastRatio(theme.focus, theme["bg-raised"]) >= 3);
    assert.ok(contrastRatio(theme.border, theme["bg-canvas"]) >= 3);
  }
});

test("design-references solo declara --ds-* y no contiene colores de tema", () => {
  const begin = css.indexOf("/* design-references v1.0.0 BEGIN */");
  const endMarker = "/* design-references v1.0.0 END */";
  const end = css.indexOf(endMarker);
  assert.ok(begin !== -1, "falta marcador BEGIN");
  assert.ok(end !== -1, "falta marcador END");
  const block = css.slice(begin, end + endMarker.length);
  assert.equal(/#[0-9a-fA-F]{3,8}\b/.test(block), false, "el núcleo contiene hex");
  const customProps = [...block.matchAll(/(--[a-z0-9-]+)\s*:/g)].map((m) => m[1]);
  for (const prop of customProps) {
    assert.ok(prop.startsWith("--ds-"), `propiedad fuera de --ds-*: ${prop}`);
  }
});
