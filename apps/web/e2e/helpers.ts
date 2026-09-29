import { expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

export const API_ORIGIN = "http://127.0.0.1:8080";
export const SYNTHETIC_JOB_ID = "job-sintetico";

export const SYNTHETIC_RUNNING_JOB = {
  job: {
    job_id: SYNTHETIC_JOB_ID,
    status: "screening",
    display_name: "agenda-sintetica.vcf",
    created_at: "2026-09-12T00:00:00Z",
    artifacts: ["vcf", "audit_tsv"],
    retention_hours: 24,
    core_version: "0.5.1",
    input: {
      original_name: "agenda-sintetica.vcf",
      sha256:
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      bytes: 128,
      upload_id: "upload-sintetico",
      source_detected: "generic",
      vcard_version: "3.0",
    },
    summary: {
      input_contacts: 12,
      retained: 9,
      needs_review: 2,
      eliminated: 1,
      quarantine: 0,
      duplicate_groups: 1,
    },
    rules: {
      mode: "builtin",
      sha256:
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    },
  },
  warnings: [],
};

export type ApiFixture =
  | "disconnected"
  | "empty-jobs"
  | "running-job"
  | "hang-jobs";

const HEALTH = {
  status: "ok",
  api_version: "0.5.1",
  core_version: "0.5.1",
  storage_mode: "local",
};

export async function analyzeAxe(page: Page) {
  const result = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag22aa"])
    .analyze();
  expect(result.violations).toEqual([]);
}

export async function waitForShellSettled(page: Page) {
  await page.evaluate(() => document.fonts.ready);
  const footer = page.getByRole("contentinfo");
  if ((await footer.count()) > 0) {
    await expect(footer).not.toContainText("Comprobando");
  }
}

export async function gotoSettled(page: Page, path: string) {
  await page.goto(path, { waitUntil: "domcontentloaded" });
  await waitForShellSettled(page);
}

export async function applyTheme(page: Page, scheme: "light" | "dark") {
  await page.addInitScript((pref) => {
    localStorage.setItem("zedazo-theme", pref);
  }, scheme);
  await page.emulateMedia({
    colorScheme: scheme,
    reducedMotion: "reduce",
  });
}

export async function installApiFixture(page: Page, fixture: ApiFixture) {
  if (fixture === "disconnected") {
    return;
  }

  await page.route(`${API_ORIGIN}/api/v1/**`, async (route) => {
    const url = new URL(route.request().url());
    const path = url.pathname;
    const method = route.request().method();

    if (path === "/api/v1/health") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(HEALTH),
      });
      return;
    }

    if (path === "/api/v1/jobs" && method === "GET") {
      if (fixture === "hang-jobs") {
        await new Promise(() => undefined);
        return;
      }
      const items =
        fixture === "running-job" ? [SYNTHETIC_RUNNING_JOB.job] : [];
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ items }),
      });
      return;
    }

    if (path === `/api/v1/jobs/${SYNTHETIC_JOB_ID}` && method === "GET") {
      if (fixture === "running-job") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(SYNTHETIC_RUNNING_JOB),
        });
        return;
      }
    }

    await route.fulfill({
      status: 404,
      contentType: "application/json",
      body: JSON.stringify({ error: "fixture-sintetico" }),
    });
  });
}
