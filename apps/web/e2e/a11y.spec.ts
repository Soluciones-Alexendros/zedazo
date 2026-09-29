import { test, expect } from "@playwright/test";
import {
  analyzeAxe,
  gotoSettled,
  installApiFixture,
  SYNTHETIC_JOB_ID,
} from "./helpers";

test.describe("navegación básica Archivo Vivo", () => {
  test("home muestra marca y CTAs", async ({ page }) => {
    await page.goto("/");
    await expect(page).toHaveTitle(/zedazo/i);
    await expect(page.getByRole("link", { name: /zedazo/i }).first()).toBeVisible();
    await expect(page.getByRole("heading", { level: 1 })).toContainText(
      "Ordena tus contactos",
    );
    await expect(
      page.getByRole("link", { name: /Procesar un archivo VCF/i }).first(),
    ).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Secciones" })).toBeVisible();
  });

  test("procesar carga el flujo", async ({ page }) => {
    await page.goto("/procesar");
    await expect(page.getByRole("heading", { name: "Procesar" })).toBeVisible();
    await expect(
      page.getByRole("navigation", { name: "Progreso del proceso" }),
    ).toBeVisible();
  });

  test("ejecuciones carga listado", async ({ page }) => {
    await page.goto("/ejecuciones");
    await expect(
      page.getByRole("heading", { level: 1, name: "Ejecuciones" }),
    ).toBeVisible();
  });

  test("skip link existe", async ({ page }) => {
    await page.goto("/");
    const skip = page.getByRole("link", { name: /Saltar al contenido/i });
    await expect(skip).toHaveCount(1);
  });
});

const AXE_ROUTES = [
  "/",
  "/procesar",
  "/ejecuciones",
  `/ejecuciones/${SYNTHETIC_JOB_ID}`,
  "/auditar",
  "/reglas",
  "/ajustes",
  "/acceso",
  "/documentacion",
  "/documentacion/ds",
] as const;

for (const path of AXE_ROUTES) {
  test(`${path} cumple criterios axe wcag2a/aa/22aa`, async ({ page }) => {
    await gotoSettled(page, path);
    await analyzeAxe(page);
  });
}

test.describe("estados representativos (fixtures sintéticos)", () => {
  test("/ejecuciones vacío cumple axe", async ({ page }) => {
    await installApiFixture(page, "empty-jobs");
    await gotoSettled(page, "/ejecuciones");
    await expect(page.getByRole("heading", { name: "Sin ejecuciones" })).toBeVisible();
    await analyzeAxe(page);
  });

  test("/ejecuciones error cumple axe", async ({ page }) => {
    await gotoSettled(page, "/ejecuciones?fixture=error");
    await expect(page.getByRole("alert").first()).toBeVisible();
    await analyzeAxe(page);
  });

  test("/ejecuciones loading cumple axe", async ({ page }) => {
    await installApiFixture(page, "hang-jobs");
    await page.goto("/ejecuciones", { waitUntil: "domcontentloaded" });
    await expect(page.getByText("Cargando ejecuciones…")).toBeVisible();
    await analyzeAxe(page);
  });

  test("/ejecuciones/job-sintetico en curso cumple axe", async ({ page }) => {
    await installApiFixture(page, "running-job");
    await gotoSettled(page, `/ejecuciones/${SYNTHETIC_JOB_ID}`);
    await expect(page.getByText("Cribando").first()).toBeVisible();
    await analyzeAxe(page);
  });

  test("/ejecuciones/job-sintetico error cumple axe", async ({ page }) => {
    await gotoSettled(page, `/ejecuciones/${SYNTHETIC_JOB_ID}?fixture=error`);
    await expect(page.getByRole("alert").first()).toBeVisible();
    await analyzeAxe(page);
  });
});

test("prefers-reduced-motion anula el spinner", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await gotoSettled(page, "/documentacion/ds");
  const spinner = page.locator(".zed-spinner").first();
  await expect(spinner).toBeVisible();
  const durationMs = await spinner.evaluate((el) => {
    const raw = getComputedStyle(el).animationDuration;
    const first = raw.split(",")[0]?.trim() ?? "0s";
    if (first.endsWith("ms")) return Number.parseFloat(first);
    return Number.parseFloat(first) * 1000;
  });
  expect(durationMs).toBeLessThan(1);
});
