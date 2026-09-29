import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { formatOklchCss, parseOklchValue } from "./oklch.mjs";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
export const WEB_ROOT = path.resolve(SCRIPT_DIR, "../..");
export const TOKENS_ROOT = path.join(WEB_ROOT, "tokens");

const REF = /^\{([a-zA-Z0-9.-]+)\}$/;

/**
 * @typedef {{
 *   path: string,
 *   type: string | undefined,
 *   value: unknown,
 *   description?: string,
 *   cssVar?: string,
 *   aliasOf?: string,
 *   theme?: "light" | "dark" | "common",
 *   file: string,
 * }} Token
 */

export async function loadTokenGraph() {
  const files = await listJsonFiles(TOKENS_ROOT);
  /** @type {Map<string, Token>} */
  const tokens = new Map();

  for (const file of files) {
    if (path.basename(file) === "contrast-pairs.json") continue;
    const raw = JSON.parse(await readFile(file, "utf8"));
    const theme = readTheme(raw) ?? themeFromFilename(file);
    walk(raw, [], undefined, theme, file, tokens);
  }

  return tokens;
}

function lookupKey(tokens, tokenPath, themeHint) {
  if (tokens.has(tokenPath)) return tokenPath;
  if (themeHint && themeHint !== "common") {
    const themed = `${tokenPath}@@${themeHint}`;
    if (tokens.has(themed)) return themed;
  }
  return undefined;
}

/**
 * @param {Map<string, Token>} tokens
 * @param {string} tokenPath
 * @param {Set<string>} [seen]
 * @param {"light" | "dark" | "common"} [themeHint]
 */
export function resolveToken(tokens, tokenPath, seen = new Set(), themeHint = "common") {
  const key = lookupKey(tokens, tokenPath, themeHint);
  if (!key) {
    throw new Error(`Token no encontrado: ${tokenPath} (tema ${themeHint})`);
  }
  const token = tokens.get(key);
  if (seen.has(key)) {
    throw new Error(`Alias circular: ${[...seen, key].join(" → ")}`);
  }
  seen.add(key);

  const nextHint = token.theme === "common" ? themeHint : token.theme;
  const value = token.value;
  if (typeof value === "string") {
    const m = value.match(REF);
    if (m) return resolveToken(tokens, m[1], seen, nextHint);
  }

  return { ...token, value: resolveNestedRefs(tokens, value, seen, nextHint) };
}

function resolveNestedRefs(tokens, value, seen, themeHint) {
  if (typeof value === "string") {
    const m = value.match(REF);
    if (m) return resolveToken(tokens, m[1], new Set(seen), themeHint).value;
    return value;
  }
  if (Array.isArray(value)) {
    return value.map((item) => resolveNestedRefs(tokens, item, seen, themeHint));
  }
  if (value && typeof value === "object") {
    const out = {};
    for (const [k, v] of Object.entries(value)) {
      out[k] = resolveNestedRefs(tokens, v, seen, themeHint);
    }
    return out;
  }
  return value;
}

/**
 * @param {unknown} value
 * @param {string | undefined} type
 */
export function formatCssValue(value, type) {
  if (typeof value === "string") return value;
  if (typeof value === "number") return String(value);

  if (value && typeof value === "object") {
    const oklch = parseOklchValue(value);
    if (oklch) return formatOklchCss(oklch);

    if ("value" in value && "unit" in value) {
      return `${value.value}${value.unit}`;
    }

    if (type === "fontFamily" && Array.isArray(value)) {
      return value.map(quoteFont).join(", ");
    }

    if (type === "cubicBezier" && Array.isArray(value) && value.length === 4) {
      return `cubic-bezier(${value.join(", ")})`;
    }

    if (type === "shadow") {
      return formatShadow(value);
    }
  }

  if (Array.isArray(value) && type === "fontFamily") {
    return value.map(quoteFont).join(", ");
  }

  if (Array.isArray(value) && type === "cubicBezier" && value.length === 4) {
    return `cubic-bezier(${value.join(", ")})`;
  }

  throw new Error(`No se puede serializar CSS para tipo=${type}: ${JSON.stringify(value)}`);
}

function formatShadow(value) {
  const parts = [dim(value.offsetX), dim(value.offsetY), dim(value.blur ?? 0)];
  const spread = dim(value.spread ?? 0);
  if (spread !== "0" && spread !== "0px") parts.push(spread);
  parts.push(formatCssValue(value.color, "color"));
  return `${value.inset ? "inset " : ""}${parts.join(" ")}`;
}

function dim(value) {
  if (typeof value === "string") return value;
  if (value && typeof value === "object" && "value" in value) {
    if (value.value === 0) return "0";
    return `${value.value}${value.unit}`;
  }
  return String(value);
}

function quoteFont(name) {
  return /[\s]/.test(name) ? `"${name}"` : name;
}

async function listJsonFiles(dir) {
  const out = [];
  const entries = await readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...(await listJsonFiles(full)));
    } else if (entry.isFile() && entry.name.endsWith(".json")) {
      out.push(full);
    }
  }
  return out.sort();
}

function readTheme(raw) {
  const ext = raw?.$extensions?.["com.zedazo"]?.theme;
  if (ext === "light" || ext === "dark") return ext;
  return undefined;
}

function themeFromFilename(file) {
  if (file.endsWith(".light.json")) return "light";
  if (file.endsWith(".dark.json")) return "dark";
  return "common";
}

function walk(node, pathParts, inheritedType, theme, file, tokens) {
  if (!node || typeof node !== "object" || Array.isArray(node)) return;

  const type = typeof node.$type === "string" ? node.$type : inheritedType;

  if (Object.prototype.hasOwnProperty.call(node, "$value")) {
    const tokenPath = pathParts.join(".");
    if (!tokenPath) return;
    const key = theme === "common" ? tokenPath : `${tokenPath}@@${theme}`;
    if (tokens.has(key)) {
      const prev = tokens.get(key);
      throw new Error(`Token duplicado: ${tokenPath} (${prev.file} y ${file})`);
    }
    tokens.set(key, {
      path: tokenPath,
      type,
      value: node.$value,
      description: typeof node.$description === "string" ? node.$description : undefined,
      cssVar: node.$extensions?.["com.zedazo"]?.cssVar,
      aliasOf:
        typeof node.$extensions?.["com.zedazo"]?.aliasOf === "string"
          ? node.$extensions["com.zedazo"].aliasOf
          : undefined,
      theme,
      file,
    });
    return;
  }

  for (const [name, child] of Object.entries(node)) {
    if (name.startsWith("$")) continue;
    walk(child, [...pathParts, name], type, theme, file, tokens);
  }
}

/**
 * Tokens con cssVar, indexados por tema.
 * @param {Map<string, Token>} tokens
 */
export function publicTokensByTheme(tokens) {
  /** @type {{ common: Token[], light: Token[], dark: Token[] }} */
  const groups = { common: [], light: [], dark: [] };
  for (const token of tokens.values()) {
    if (!token.cssVar) continue;
    groups[token.theme].push(token);
  }
  for (const list of Object.values(groups)) {
    list.sort((a, b) => a.cssVar.localeCompare(b.cssVar) || a.path.localeCompare(b.path));
  }
  return groups;
}

/**
 * Resuelve un path semántico en un tema (dark cae a common si no hay override).
 * @param {Map<string, Token>} tokens
 * @param {string} tokenPath
 * @param {"light" | "dark" | "common"} theme
 */
export function resolveInTheme(tokens, tokenPath, theme) {
  return resolveToken(tokens, tokenPath, new Set(), theme);
}
