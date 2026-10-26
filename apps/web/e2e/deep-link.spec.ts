import { test, expect } from "@playwright/test";
import { gotoSettled } from "./helpers";

test.describe("recarga y enlaces profundos (N-02)", () => {
  test("/procesar sobrevive a F5 y muestra el encabezado", async ({ page }) => {
    await gotoSettled(page, "/procesar");
    await expect(
      page.getByRole("heading", { level: 1, name: "Procesar" }),
    ).toBeVisible();

    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(
      page.getByRole("heading", { level: 1, name: "Procesar" }),
    ).toBeVisible();
  });
});
