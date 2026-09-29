import { test, expect } from "@playwright/test";
import {
  applyTheme,
  gotoSettled,
  installApiFixture,
  SYNTHETIC_JOB_ID,
} from "./helpers";

const DESKTOP = { width: 1440, height: 900 } as const;

const ROUTES: { slug: string; path: string }[] = [
  { slug: "home", path: "/" },
  { slug: "procesar", path: "/procesar" },
  { slug: "ejecuciones", path: "/ejecuciones" },
  { slug: "auditar", path: "/auditar" },
  { slug: "reglas", path: "/reglas" },
  { slug: "ajustes", path: "/ajustes" },
  { slug: "acceso", path: "/acceso" },
  { slug: "documentacion", path: "/documentacion" },
  { slug: "documentacion-ds", path: "/documentacion/ds" },
];

async function settleRoute(page: import("@playwright/test").Page, path: string) {
  await gotoSettled(page, path);
  if (path === "/") {
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    await expect(page.getByText("Cargando ejecuciones…")).toHaveCount(0);
  }
  if (path === "/ejecuciones") {
    await expect(page.getByRole("heading", { level: 1, name: "Ejecuciones" })).toBeVisible();
    await expect(page.getByText("Cargando ejecuciones…")).toHaveCount(0);
  }
  if (path === "/ajustes") {
    await expect(page.locator("#ajustes-tema")).toBeVisible();
  }
}

for (const scheme of ["light", "dark"] as const) {
  test.describe(`regresión visual desktop ${scheme}`, () => {
    test.use({
      colorScheme: scheme,
      viewport: DESKTOP,
    });

    for (const route of ROUTES) {
      test(`${route.slug}`, async ({ page }) => {
        await applyTheme(page, scheme);
        await settleRoute(page, route.path);
        await expect(page).toHaveScreenshot(`${route.slug}-${scheme}.png`, {
          animations: "disabled",
          caret: "hide",
        });
      });
    }

    test("ejecucion-running", async ({ page }) => {
      await applyTheme(page, scheme);
      await installApiFixture(page, "running-job");
      await gotoSettled(page, `/ejecuciones/${SYNTHETIC_JOB_ID}`);
      await expect(page.getByText("Cribando").first()).toBeVisible();
      await expect(page).toHaveScreenshot(`ejecucion-running-${scheme}.png`, {
        animations: "disabled",
        caret: "hide",
      });
    });
  });
}

for (const scheme of ["light", "dark"] as const) {
  test.describe(`regresión visual móvil shell ${scheme}`, () => {
    test.use({
      colorScheme: scheme,
      viewport: { width: 390, height: 844 },
      isMobile: true,
      hasTouch: true,
    });

    test("home", async ({ page }) => {
      await applyTheme(page, scheme);
      await settleRoute(page, "/");
      await expect(page).toHaveScreenshot(`shell-mobile-${scheme}.png`, {
        animations: "disabled",
        caret: "hide",
        // Viewport móvil + system-ui: AA tipográfica más volátil entre host/CI
        maxDiffPixelRatio: 0.15,
      });
    });
  });
}
