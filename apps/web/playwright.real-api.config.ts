import { defineConfig, devices } from "@playwright/test";

/** Config sin webServer: API + next los arranca scripts/e2e-real-api.sh */
export default defineConfig({
  testDir: "./e2e",
  testMatch: /real-api\.spec\.ts/,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  reporter: "list",
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:3000",
    trace: "on-first-retry",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
});
