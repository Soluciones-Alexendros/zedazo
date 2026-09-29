#!/usr/bin/env node
/**
 * Gate WCAG 2.2 AA sobre pares semánticos light y dark.
 * Convierte OKLCH → sRGB lineal antes del ratio (no usa L de OKLCH).
 */

import { readFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { contrastRatio, parseOklchValue } from "./oklch.mjs";
import { TOKENS_ROOT, loadTokenGraph, resolveInTheme } from "./load.mjs";

const THEMES = ["light", "dark"];

export async function evaluateContrastPairs() {
  const graph = await loadTokenGraph();
  const spec = JSON.parse(await readFile(path.join(TOKENS_ROOT, "contrast-pairs.json"), "utf8"));
  const results = [];

  for (const pair of spec.pairs) {
    for (const theme of THEMES) {
      const fgTok = resolveInTheme(graph, pair.fg, theme);
      const bgTok = resolveInTheme(graph, pair.bg, theme);
      const fg = parseOklchValue(fgTok.value);
      const bg = parseOklchValue(bgTok.value);
      if (!fg || !bg) {
        throw new Error(`Par ${pair.id} (${theme}): fg/bg no son OKLCH estructurado`);
      }
      const ratio = contrastRatio(fg, bg);
      results.push({
        id: pair.id,
        theme,
        fg: pair.fg,
        bg: pair.bg,
        minRatio: pair.minRatio,
        role: pair.role ?? "text",
        ratio,
        pass: ratio + 1e-9 >= pair.minRatio,
      });
    }
  }

  return results;
}

function formatRatio(n) {
  return n.toFixed(2);
}

const isMain = Boolean(process.argv[1]) && import.meta.url === pathToFileURL(process.argv[1]).href;

if (isMain) {
  const results = await evaluateContrastPairs();
  const failed = results.filter((r) => !r.pass);
  for (const row of results) {
    const mark = row.pass ? "ok" : "FAIL";
    console.log(
      `${mark} [${row.theme}] ${row.id} ${formatRatio(row.ratio)}:1 (min ${row.minRatio})`,
    );
  }
  if (failed.length) {
    console.error(`\n${failed.length} pares por debajo de WCAG 2.2 AA`);
    process.exitCode = 1;
  } else {
    console.log(`\n${results.length} pares OK (WCAG 2.2 AA)`);
  }
}
