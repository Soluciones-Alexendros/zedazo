/**
 * OKLCH → sRGB lineal → sRGB / hex, y contraste WCAG 2.2.
 * WCAG mide luminancia relativa sRGB, no el canal L de OKLCH.
 *
 * Matrices: Björn Ottosson, OKLab.
 * https://bottosson.github.io/posts/oklab/
 */

const DEG = Math.PI / 180;

/**
 * @typedef {{ l: number, c: number, h: number, alpha?: number }} Oklch
 */

/**
 * @param {Oklch} color
 * @returns {{ r: number, g: number, b: number, alpha: number }} sRGB lineal (no recortado)
 */
export function oklchToLinearSrgb(color) {
  const { l, c, h } = color;
  const hr = h * DEG;
  const a = c * Math.cos(hr);
  const b = c * Math.sin(hr);

  const l_ = l + 0.3963377774 * a + 0.2158037573 * b;
  const m_ = l - 0.1055613458 * a - 0.0638541728 * b;
  const s_ = l - 0.0894841775 * a - 1.291485548 * b;

  const l3 = l_ * l_ * l_;
  const m3 = m_ * m_ * m_;
  const s3 = s_ * s_ * s_;

  return {
    r: +4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3,
    g: -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3,
    b: -0.0041960863 * l3 - 0.7034186147 * m3 + 1.707614701 * s3,
    alpha: color.alpha ?? 1,
  };
}

function srgbTransfer(channel) {
  const abs = Math.abs(channel);
  const encoded =
    abs <= 0.0031308 ? 12.92 * channel : Math.sign(channel) * (1.055 * abs ** (1 / 2.4) - 0.055);
  return encoded;
}

function inSrgbGamut(lin) {
  return lin.r >= 0 && lin.r <= 1 && lin.g >= 0 && lin.g <= 1 && lin.b >= 0 && lin.b <= 1;
}

/**
 * Aproxima el gamut mapping CSS (reducir C, conservar L/H) para que
 * hex y contraste coincidan con lo que pinta el motor, no con un clip RGB.
 * @param {Oklch} color
 */
export function mapToSrgbGamut(color) {
  if (inSrgbGamut(oklchToLinearSrgb(color))) return color;
  let lo = 0;
  let hi = color.c;
  let mapped = { ...color, c: 0 };
  for (let i = 0; i < 32; i++) {
    const mid = (lo + hi) / 2;
    const candidate = { ...color, c: mid };
    if (inSrgbGamut(oklchToLinearSrgb(candidate))) {
      mapped = candidate;
      lo = mid;
    } else {
      hi = mid;
    }
  }
  return mapped;
}

/**
 * @param {Oklch} color
 * @returns {{ r: number, g: number, b: number, alpha: number }} sRGB 0–1
 */
export function oklchToSrgb(color) {
  const mapped = mapToSrgbGamut(color);
  const lin = oklchToLinearSrgb(mapped);
  return {
    r: clamp01(srgbTransfer(lin.r)),
    g: clamp01(srgbTransfer(lin.g)),
    b: clamp01(srgbTransfer(lin.b)),
    alpha: mapped.alpha ?? 1,
  };
}

function clamp01(n) {
  if (n < 0) return 0;
  if (n > 1) return 1;
  return n;
}

function toByte(channel) {
  return Math.round(clamp01(channel) * 255);
}

function byteHex(n) {
  return n.toString(16).padStart(2, "0");
}

/**
 * Hex #rrggbb generado (fallback theme-color / favicon). Nunca es origen.
 * @param {Oklch} color
 */
export function oklchToHex(color) {
  const srgb = oklchToSrgb(color);
  return `#${byteHex(toByte(srgb.r))}${byteHex(toByte(srgb.g))}${byteHex(toByte(srgb.b))}`;
}

/**
 * Luminancia relativa WCAG 2.x (sRGB).
 * @param {Oklch} color
 */
export function relativeLuminance(color) {
  const lin = oklchToLinearSrgb(mapToSrgbGamut(color));
  const r = clamp01(lin.r);
  const g = clamp01(lin.g);
  const b = clamp01(lin.b);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/**
 * Ratio de contraste WCAG (más claro + 0.05) / (más oscuro + 0.05).
 * @param {Oklch} a
 * @param {Oklch} b
 */
export function contrastRatio(a, b) {
  const l1 = relativeLuminance(a);
  const l2 = relativeLuminance(b);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * @param {unknown} value
 * @returns {Oklch | null}
 */
export function parseOklchValue(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  if (value.colorSpace !== "oklch" || !Array.isArray(value.components)) return null;
  const [l, c, h] = value.components;
  if (![l, c, h].every((n) => typeof n === "number" && Number.isFinite(n))) return null;
  const alpha = typeof value.alpha === "number" ? value.alpha : 1;
  return { l, c, h, alpha };
}

/**
 * Serializa un color DTCG OKLCH a función CSS (sin hex).
 * @param {Oklch} color
 */
export function formatOklchCss(color) {
  const { l, c, h, alpha = 1 } = color;
  const core = `oklch(${formatNum(l)} ${formatNum(c)} ${formatNum(h)})`;
  if (alpha >= 1) return core;
  return `oklch(${formatNum(l)} ${formatNum(c)} ${formatNum(h)} / ${formatNum(alpha)})`;
}

function formatNum(n) {
  if (Number.isInteger(n)) return String(n);
  const trimmed = n.toFixed(6).replace(/\.?0+$/, "");
  return trimmed;
}
