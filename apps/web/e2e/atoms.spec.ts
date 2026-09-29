import { test, expect } from "@playwright/test";
import { analyzeAxe, gotoSettled } from "./helpers";

test.describe("catálogo de átomos", () => {
  test("muestra variantes de botón, badge, input, callout y patrones", async ({
    page,
  }) => {
    await gotoSettled(page, "/documentacion/ds");
    await expect(
      page.getByRole("heading", { level: 1, name: "Sistema de diseño" }),
    ).toBeVisible();
    await expect(
      page.getByRole("navigation", { name: "Secciones del catálogo" }),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "Primario" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Secundario" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Peligro" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Deshabilitado" })).toBeDisabled();
    await expect(page.getByRole("button", { name: "Cargando" })).toHaveAttribute(
      "aria-busy",
      "true",
    );
    await expect(page.getByText("Neutral", { exact: true })).toBeVisible();
    await expect(page.getByLabel("Campo de ejemplo")).toBeVisible();
    await expect(page.getByRole("note").first()).toBeVisible();
    await expect(page.getByRole("heading", { name: "Estados" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Vacío" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Ejecución" })).toBeVisible();
    await expect(page.getByText("Cribando").first()).toBeVisible();
    await expect(page.getByText("Zona de archivo de ejemplo")).toBeVisible();
  });

  test("/documentacion/ds cumple axe wcag2a/aa/22aa", async ({ page }) => {
    await gotoSettled(page, "/documentacion/ds");
    await analyzeAxe(page);
  });
});
