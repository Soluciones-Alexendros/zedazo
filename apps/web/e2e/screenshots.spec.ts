import { test } from "@playwright/test";
import path from "path";
import { applyTheme, gotoSettled } from "./helpers";

const OUT = path.resolve(process.cwd(), "../../docs/screenshots");

test.beforeEach(() => {
  test.skip(
    !process.env.ZEDAZO_SCREENSHOTS,
    "Exportar capturas README: ZEDAZO_SCREENSHOTS=1 pnpm test:e2e -- e2e/screenshots.spec.ts",
  );
});

const routes: { slug: string; path: string }[] = [
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

for (const scheme of ["light", "dark"] as const) {
  test.describe(`capturas README ${scheme}`, () => {
    test.use({
      colorScheme: scheme,
      viewport: { width: 1440, height: 900 },
    });

    for (const route of routes) {
      test(`${route.slug}`, async ({ page }) => {
        await applyTheme(page, scheme);
        await gotoSettled(page, route.path);
        await page.screenshot({
          path: path.join(OUT, `${route.slug}-${scheme}.png`),
          fullPage: false,
          animations: "disabled",
          caret: "hide",
        });
      });
    }
  });
}
