#!/usr/bin/env node
/**
 * E2E tauri-driver: sample.vcf + invariante I7 (origen intacto).
 * Env: TAURI_DRIVER_PORT (default 4444), TAURI_APPIMAGE, ZEDAZO_SAMPLE_VCF opcional.
 * ZEDAZO_ADDR opcional: si falta, se deriva de browser.getUrl().
 * Imprime ZEDAZO_ADDR=… y ZEDAZO_TOKEN=… en stdout (líneas KEY=) para parity.
 */
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { copyFileSync, readFileSync, writeFileSync, mkdirSync, appendFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { remote } from "webdriverio";
import { fileURLToPath } from "node:url";

const driverPort = Number(process.env.TAURI_DRIVER_PORT || 4444);
const appimage = process.env.TAURI_APPIMAGE;
if (!appimage) {
  console.error("TAURI_APPIMAGE requerido");
  process.exit(1);
}

const sampleSrc = resolve(
  process.env.ZEDAZO_SAMPLE_VCF || join(process.cwd(), "examples", "sample.vcf"),
);
const outEnv = process.env.ZEDAZO_E2E_ENV_OUT || "";

const work = join(tmpdir(), `zedazo-e2e-${process.pid}`);
mkdirSync(work, { recursive: true });
const sample = join(work, "sample.vcf");
copyFileSync(sampleSrc, sample);
const shaBefore = createHash("sha256").update(readFileSync(sample)).digest("hex");

const browser = await remote({
  hostname: "127.0.0.1",
  port: driverPort,
  path: "/",
  // Sin browserName: WDIO v9 inyectaría webSocketUrl si browserName≠safari;
  // tauri-driver no lo elimina y WebKitWebDriver responde "Failed to match capabilities".
  // Canon: docs Tauri WebdriverIO (solo tauri:options).
  capabilities: {
    "tauri:options": {
      application: appimage,
    },
  },
  connectionRetryCount: 3,
  connectionRetryTimeout: 180_000,
  logLevel: "warn",
});

function emit(key, value) {
  const line = `${key}=${value}`;
  console.log(line);
  if (outEnv) {
    appendFileSync(outEnv, `${line}\n`);
  }
}

try {
  await browser.pause(3000);
  const title = await browser.getTitle();
  console.log("window title:", title);

  let addr = process.env.ZEDAZO_ADDR || "";
  if (!addr) {
    const url = await browser.getUrl();
    const m = String(url).match(/https?:\/\/(127\.0\.0\.1:\d+)/);
    if (!m) {
      throw new Error(`no se pudo derivar addr de getUrl(): ${url}`);
    }
    addr = m[1];
  }
  emit("ZEDAZO_ADDR", addr);

  const token = await browser.execute(() => {
    // eslint-disable-next-line no-undef
    return window.__ZEDAZO_SESSION_TOKEN__ || null;
  });
  if (!token || typeof token !== "string" || token.length !== 64) {
    throw new Error("no se leyó __ZEDAZO_SESSION_TOKEN__ vía tauri-driver");
  }
  emit("ZEDAZO_TOKEN", token);

  const base = `http://${addr}`;
  const headers = {
    Authorization: `Bearer ${token}`,
  };

  const form = new FormData();
  const blob = new Blob([readFileSync(sample)], { type: "text/vcard" });
  form.append("file", blob, "sample.vcf");

  const up = await fetch(`${base}/api/v1/uploads`, {
    method: "POST",
    headers,
    body: form,
  });
  if (!up.ok) {
    throw new Error(`upload HTTP ${up.status}: ${await up.text()}`);
  }
  const upload = await up.json();
  const uploadId = upload.upload_id || upload.id;
  if (!uploadId) {
    throw new Error(`upload sin id: ${JSON.stringify(upload)}`);
  }

  const jobRes = await fetch(`${base}/api/v1/jobs`, {
    method: "POST",
    headers: { ...headers, "Content-Type": "application/json" },
    body: JSON.stringify({ upload_id: uploadId }),
  });
  if (!jobRes.ok) {
    throw new Error(`jobs HTTP ${jobRes.status}: ${await jobRes.text()}`);
  }
  const job = await jobRes.json();
  const jobId = job.job_id || job.id;
  if (!jobId) {
    throw new Error(`job sin id: ${JSON.stringify(job)}`);
  }

  let status = job.status || "";
  for (let i = 0; i < 90; i++) {
    const st = await fetch(`${base}/api/v1/jobs/${jobId}`, { headers });
    const body = await st.json();
    status = body.job?.status || body.status || body.state || "";
    console.log("job status:", status);
    if (/completed|failed/i.test(String(status))) break;
    await new Promise((r) => setTimeout(r, 1000));
  }
  if (!/completed/i.test(String(status))) {
    throw new Error(`job no completed: ${status}`);
  }

  const shaAfter = createHash("sha256").update(readFileSync(sample)).digest("hex");
  if (shaBefore !== shaAfter) {
    throw new Error(`I7 violada: sample.vcf mutó (${shaBefore} → ${shaAfter})`);
  }
  console.log("I7 OK: origen intacto", shaBefore);

  const art = await fetch(`${base}/api/v1/jobs/${jobId}/artifacts/vcf`, { headers });
  if (art.ok) {
    const outPath = join(work, "limpio.vcf");
    writeFileSync(outPath, Buffer.from(await art.arrayBuffer()));
    const shaOut = createHash("sha256").update(readFileSync(outPath)).digest("hex");
    if (shaOut === shaBefore) {
      console.warn("aviso: VCF de salida idéntico al origen (posible no-op de reglas)");
    } else {
      console.log("salida distinta del origen:", shaOut);
    }
  } else {
    console.warn(`artefacto vcf HTTP ${art.status} (I7 ya verificada sobre origen)`);
  }

  console.log("OK e2e tauri-driver sample.vcf + I7");

  if (process.env.ZEDAZO_RUN_PARITY === "1") {
    const root = resolve(fileURLToPath(new URL(".", import.meta.url)), "../../..");
    const parity = join(root, "scripts/desktop/parity-cribar.sh");
    console.log("==> parity-cribar.sh");
    const r = spawnSync("bash", [parity], {
      cwd: root,
      env: {
        ...process.env,
        ZEDAZO_ADDR: addr,
        ZEDAZO_TOKEN: token,
        ZEDAZO_SAMPLE_VCF: sampleSrc,
      },
      stdio: "inherit",
    });
    if (r.status !== 0) {
      throw new Error(`parity-cribar falló (exit ${r.status})`);
    }
  }
} finally {
  await browser.deleteSession().catch(() => {});
}
