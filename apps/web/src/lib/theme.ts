export type ThemePreference = "system" | "light" | "dark";

const STORAGE_KEY = "ds-theme";
const LEGACY_STORAGE_KEY = "zedazo-theme";

export function readThemePreference(): ThemePreference {
  if (typeof window === "undefined") return "system";
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (raw === "light" || raw === "dark" || raw === "system") return raw;
  return "system";
}

function resolveEffectiveTheme(
  preference: ThemePreference,
): "light" | "dark" {
  if (preference === "light" || preference === "dark") return preference;
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

export function applyTheme(preference: ThemePreference) {
  if (typeof document === "undefined") return;
  const effective = resolveEffectiveTheme(preference);
  document.documentElement.setAttribute("data-theme", effective);
  document.documentElement.style.colorScheme = effective;
}

export function persistTheme(preference: ThemePreference) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(STORAGE_KEY, preference);
  applyTheme(preference);
}

/** Migra la clave anterior (zedazo-theme) a ds-theme, una sola vez, al arrancar. */
export function migrateLegacyTheme(): ThemePreference | null {
  if (typeof window === "undefined") return null;
  try {
    const legacy = window.localStorage.getItem(LEGACY_STORAGE_KEY);
    if (legacy === null) return null;
    let mapped: ThemePreference;
    switch (legacy) {
      case "claro":
      case "light":
        mapped = "light";
        break;
      case "oscuro":
      case "dark":
        mapped = "dark";
        break;
      case "auto":
      case "sistema":
      case "system":
        mapped = "system";
        break;
      default:
        mapped = "system";
    }
    window.localStorage.setItem(STORAGE_KEY, mapped);
    window.localStorage.removeItem(LEGACY_STORAGE_KEY);
    return mapped;
  } catch {
    return null;
  }
}

/** Script de arranque C20 (design-references v1.0.0) — texto exacto, sin nada antes ni después de los marcadores. */
export const THEME_BOOT_SCRIPT = `/* design-references v1.0.0 theme-boot BEGIN */
(function () {
  var t = "system";
  try { t = localStorage.getItem("ds-theme") || "system"; } catch (e) {}
  if (t !== "light" && t !== "dark") {
    t = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  document.documentElement.setAttribute("data-theme", t);
})();
/* design-references v1.0.0 theme-boot END */`;
