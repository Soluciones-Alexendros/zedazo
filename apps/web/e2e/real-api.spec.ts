/**
 * E2E contra API Rust real (capa 2).
 * Activar con ZEDAZO_E2E_REAL_API=1 y API en 127.0.0.1:8080.
 * Las specs con fixtures mock siguen siendo la capa 1 (componentes).
 */
import { test, expect } from "@playwright/test";
import path from "path";
import fs from "fs";

const enabled = process.env.ZEDAZO_E2E_REAL_API === "1";
const apiBase =
  process.env.NEXT_PUBLIC_API_BASE?.trim() || "http://127.0.0.1:8080";

test.describe("API Rust real", () => {
  test.skip(!enabled, "Requiere ZEDAZO_E2E_REAL_API=1");

  test("health de la API responde", async ({ request }) => {
    const res = await request.get(`${apiBase}/api/v1/health`);
    expect(res.ok()).toBeTruthy();
    const body = await res.json();
    expect(body.status).toBe("ok");
  });

  test("subir sample.vcf, completar job y ver en UI", async ({
    page,
    request,
  }) => {
    const sample = path.resolve(__dirname, "../../../examples/sample.vcf");
    expect(fs.existsSync(sample)).toBeTruthy();

    const upload = await request.post(`${apiBase}/api/v1/uploads`, {
      multipart: {
        file: {
          name: "sample.vcf",
          mimeType: "text/vcard",
          buffer: fs.readFileSync(sample),
        },
      },
    });
    expect(upload.status()).toBe(201);
    const { upload_id } = await upload.json();

    const create = await request.post(`${apiBase}/api/v1/jobs`, {
      data: {
        upload_id,
        artifacts: ["vcf", "audit_tsv", "stats_json"],
      },
    });
    expect(create.status()).toBe(201);
    const { job_id } = await create.json();

    let status = "";
    for (let i = 0; i < 60; i++) {
      const get = await request.get(`${apiBase}/api/v1/jobs/${job_id}`);
      expect(get.ok()).toBeTruthy();
      const payload = await get.json();
      status = payload.job.status;
      if (status === "completed" || status === "failed") break;
      await page.waitForTimeout(500);
    }
    expect(status).toBe("completed");

    await page.goto(`/ejecuciones/${job_id}`, {
      waitUntil: "domcontentloaded",
    });
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible({
      timeout: 15_000,
    });
  });
});
