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

  // S-04: VCF sintético ~10 MB (generado en tmp; no se commitea).
  const tenMb = 10 * 1024 * 1024;
  let big = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:big10\r\nFN:Big Ten\r\nNOTE:";
  big += "z".repeat(Math.max(0, tenMb - big.length - 20));
  big += "\r\nEND:VCARD\r\n";
  const bigBuf = Buffer.from(big);
  const bigPath = join(work, "synthetic-10mb.vcf");
  writeFileSync(bigPath, bigBuf);
  const bigSha = createHash("sha256").update(bigBuf).digest("hex");
  const form10 = new FormData();
  form10.append("file", new Blob([bigBuf], { type: "text/vcard" }), "synthetic-10mb.vcf");
  const up10 = await fetch(`${base}/api/v1/uploads`, {
    method: "POST",
    headers,
    body: form10,
  });
  if (up10.status !== 201) {
    throw new Error(`upload 10MB HTTP ${up10.status}: ${await up10.text()}`);
  }
  const up10json = await up10.json();
  if (up10json.sha256 !== bigSha) {
    throw new Error(`sha256 10MB mismatch: ${up10json.sha256} vs ${bigSha}`);
  }
  console.log("OK e2e upload sintético 10 MB");

  // V-04: withGlobalTauri=false → sin API global `__TAURI__`.
  // `__TAURI_INTERNALS__` sigue existiendo (IPC de Tauri 2); no es la superficie prohibida.
  const hasGlobalTauri = await browser.execute(() => {
    // eslint-disable-next-line no-undef
    return typeof window.__TAURI__ !== "undefined";
  });
  if (hasGlobalTauri) {
    throw new Error("window.__TAURI__ no debe existir (withGlobalTauri=false)");
  }
  console.log("OK V-04: sin __TAURI__ global");

  // V-03: navegación externa no abandona el loopback.
  const urlBefore = await browser.getUrl();
  await browser.execute(() => {
    // eslint-disable-next-line no-undef
    window.location.href = "https://example.com/";
  });
  await browser.pause(1500);
  const urlAfter = await browser.getUrl();
  if (!/127\.0\.0\.1:\d+/.test(String(urlAfter))) {
    throw new Error(
      `navegación externa escapó del loopback: antes=${urlBefore} después=${urlAfter}`,
    );
  }
  console.log("OK V-03: webview permanece en loopback tras example.com");

  // V-01: clic «Descargar» en GUI; contenido vía fetch+Bearer coincide con API.
  const artApi = await fetch(`${base}/api/v1/jobs/${jobId}/artifacts/vcf`, { headers });
  if (!artApi.ok) {
    throw new Error(`artefacto vcf para V-01 HTTP ${artApi.status}`);
  }
  const artBuf = Buffer.from(await artApi.arrayBuffer());
  const artSha = createHash("sha256").update(artBuf).digest("hex");

  await browser.url(
    `http://${addr}/ejecuciones?job=${encodeURIComponent(jobId)}`,
  );
  // Esperar hidratación del detalle (Descargar VCF en cabecera).
  let clicked = false;
  for (let i = 0; i < 30; i++) {
    const btn = await browser.$("button.zed-button--primary");
    const exists = await btn.isExisting().catch(() => false);
    if (exists) {
      const text = await btn.getText().catch(() => "");
      if (/Descargar/i.test(String(text))) {
        await btn.click();
        clicked = true;
        break;
      }
    }
    // Fallback: cualquier botón con texto Descargar
    const all = await browser.$$("button");
    for (const b of all) {
      const t = await b.getText().catch(() => "");
      if (/Descargar/i.test(String(t))) {
        await b.click();
        clicked = true;
        break;
      }
    }
    if (clicked) break;
    await browser.pause(500);
  }
  if (!clicked) {
    const urlNow = await browser.getUrl();
    const bodySnippet = await browser.execute(() =>
      // eslint-disable-next-line no-undef
      (document.body && document.body.innerText || "").slice(0, 400),
    );
    throw new Error(
      `no se encontró botón Descargar en detalle de job (url=${urlNow}): ${bodySnippet}`,
    );
  }
  await browser.pause(1000);
  const downloadErr = await browser.execute(() => {
    // eslint-disable-next-line no-undef
    const err = document.querySelector(".zed-artifact .zed-muted");
    // Solo fallar si hay mensaje de error de ApiError típico
    const nodes = Array.from(document.querySelectorAll(".zed-artifact p"));
    for (const n of nodes) {
      const t = n.textContent || "";
      if (/ApiError|HTTP\s+[45]|TypeError|Failed to fetch/i.test(t)) return t;
    }
    return null;
  });
  if (downloadErr) {
    throw new Error(`Descargar mostró error: ${downloadErr}`);
  }

  const guiSha = await browser.execute(async (jid) => {
    // eslint-disable-next-line no-undef
    const tok = window.__ZEDAZO_SESSION_TOKEN__;
    // eslint-disable-next-line no-undef
    const res = await fetch(`/api/v1/jobs/${jid}/artifacts/vcf`, {
      headers: { Authorization: `Bearer ${tok}` },
    });
    if (!res.ok) throw new Error(`GUI fetch artifact ${res.status}`);
    const buf = await res.arrayBuffer();
    const hash = await crypto.subtle.digest("SHA-256", buf);
    return Array.from(new Uint8Array(hash))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
  }, jobId);
  if (guiSha !== artSha) {
    throw new Error(`V-01 sha mismatch GUI=${guiSha} API=${artSha}`);
  }
  console.log("OK V-01: Descargar + contenido Bearer coincide");

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
