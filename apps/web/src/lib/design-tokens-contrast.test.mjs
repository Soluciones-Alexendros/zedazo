import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import {
  contrastRatio,
  mapToSrgbGamut,
  oklchToHex,
  parseOklchValue,
} from "../../scripts/design-tokens/oklch.mjs";
import { evaluateContrastPairs } from "../../scripts/design-tokens/contrast.mjs";

const WEB_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("contraste sintético: negro sobre blanco ≈ 21:1", () => {
  const white = { l: 1, c: 0, h: 0 };
  const black = { l: 0, c: 0, h: 0 };
  assert.ok(Math.abs(contrastRatio(white, black) - 21) < 0.05);
});

test("contraste sintético: mismo gris no alcanza AA texto", () => {
  const gray = { l: 0.5, c: 0, h: 0 };
  assert.ok(contrastRatio(gray, gray) < 1.05);
});

test("contraste sintético: par que falla 4.5:1", () => {
  const fg = { l: 0.7, c: 0, h: 0 };
  const bg = { l: 0.85, c: 0, h: 0 };
  assert.ok(contrastRatio(fg, bg) < 4.5);
});

test("hex generado no es origen: canvas claro es OKLCH→sRGB", () => {
  const hex = oklchToHex({ l: 0.985, c: 0.008, h: 257 });
  assert.match(hex, /^#[0-9a-f]{6}$/);
  assert.equal(hex, "#f7faff");
});

test("pares semánticos light/dark cumplen el umbral WCAG 2.2", async () => {
  const results = await evaluateContrastPairs();
  assert.ok(results.length >= 30);
  const failed = results.filter((row) => !row.pass);
  assert.deepEqual(
    failed.map((row) => `${row.theme}:${row.id}`),
    [],
  );
});

test("themes generados conservan OKLCH públicos 1:1", async () => {
  const css = await readFile(
    path.join(WEB_ROOT, "src/design-system/generated/themes.css"),
    "utf8",
  );
  for (const needle of [
    "--zed-bg-canvas: oklch(0.985 0.008 257);",
    "--zed-accent: oklch(0.53 0.16 257);",
    "--zed-bg-canvas: oklch(0.16 0.025 258);",
    "--zed-accent: oklch(0.76 0.13 257);",
    "--zed-fg-default: oklch(0.34 0.03 258);",
    "--zed-fg-default: oklch(0.86 0.018 250);",
  ]) {
    assert.ok(css.includes(needle), `falta ${needle}`);
  }
});

test("fuente DTCG no contiene hex de producción", async () => {
  const files = await listJson(path.join(WEB_ROOT, "tokens"));
  const hex = /#[0-9a-fA-F]{3,8}\b/;
  for (const file of files) {
    const raw = await readFile(file, "utf8");
    assert.equal(hex.test(raw), false, `hex en ${path.relative(WEB_ROOT, file)}`);
  }
});

test("parseOklchValue rechaza no-OKLCH", () => {
  assert.equal(parseOklchValue({ colorSpace: "srgb", components: [1, 0, 0] }), null);
  assert.equal(parseOklchValue("oklch(1 0 0)"), null);
});

test("contrato v1: alias bg/text/border/action/feedback y landing", async () => {
  const gui = await readFile(
    path.join(WEB_ROOT, "src/design-system/generated/tokens.css"),
    "utf8",
  );
  for (const needle of [
    "--color-bg-base: var(--zed-bg-canvas);",
    "--color-text-base: var(--zed-fg-default);",
    "--color-border-base: var(--zed-border-default);",
    "--color-action-primary-bg: var(--zed-accent);",
    "--color-feedback-success-text: var(--zed-success-on-soft);",
    "--color-button-primary-bg: var(--color-action-primary-bg);",
    "--color-card-bg: var(--color-bg-surface);",
    "--color-input-focus: var(--color-action-focus-ring);",
  ]) {
    assert.ok(gui.includes(needle), `falta ${needle}`);
  }

  const landingCss = await readFile(
    path.join(WEB_ROOT, "../landing/tokens.css"),
    "utf8",
  );
  assert.match(landingCss, /--color-text-base: oklch\(0\.34 0\.03 258\);/);
  assert.match(landingCss, /--color-text-base: oklch\(0\.86 0\.018 250\);/);
  assert.match(landingCss, /@supports not \(color: oklch\(0 0 0\)\)/);

  const html = await readFile(path.join(WEB_ROOT, "../landing/index.html"), "utf8");
  const layout = await readFile(path.join(WEB_ROOT, "../landing/landing.css"), "utf8");
  const painted = html.replace(/<meta name="theme-color"[^>]*>/g, "");
  assert.equal(/oklch\(/.test(painted), false);
  assert.equal(/#[0-9a-fA-F]{3,8}\b/.test(painted), false);
  assert.equal(/oklch\(/.test(layout), false);
  assert.equal(/#[0-9a-fA-F]{3,8}\b/.test(layout), false);
  assert.match(html, /href="\.\/tokens\.css"/);
  assert.match(html, /<h1 class="wordmark">zedazo<\/h1>/);
});

test("mapToSrgbGamut reduce C si el color está fuera de sRGB", () => {
  const loud = { l: 0.7, c: 0.4, h: 30, alpha: 1 };
  const mapped = mapToSrgbGamut(loud);
  assert.ok(mapped.c < loud.c);
});

async function listJson(dir) {
  const out = [];
  const entries = await readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...(await listJson(full)));
    else if (entry.name.endsWith(".json")) out.push(full);
  }
  return out;
}
