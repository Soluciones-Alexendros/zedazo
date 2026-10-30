#!/usr/bin/env node
/**
 * E2E tauri-driver: sample.vcf + I7 + recorrido GUI + paridad opcional.
 * Env: TAURI_DRIVER_PORT (default 4444), TAURI_APPIMAGE, ZEDAZO_SAMPLE_VCF opcional.
 * ZEDAZO_ADDR opcional: si falta, se deriva de browser.getUrl().
 * Token: solo a ZEDAZO_E2E_ENV_OUT (nunca en stdout en claro).
 */
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  readFileSync,
  writeFileSync,
  mkdirSync,
  appendFileSync,
} from "node:fs";
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
  if (key === "ZEDAZO_TOKEN") {
    console.log(`ZEDAZO_TOKEN=<redacted len=${String(value).length}>`);
  } else {
    console.log(`${key}=${value}`);
  }
  if (outEnv) {
    appendFileSync(outEnv, `${key}=${value}\n`);
  }
}

/** Readiness: React hidratado (no basta textContent: los `<script>` __next_f cuentan). */
async function waitAppReady(label, timeoutMs = 45_000) {
  const deadline = Date.now() + timeoutMs;
  let last = "";
  while (Date.now() < deadline) {
    const snap = await browser.execute(() => {
      // eslint-disable-next-line no-undef
      const body = document.body;
      const inner = body ? body.innerText || "" : "";
      const hydrated = Boolean(
        document.querySelector(
          ".zed-gate, .zed-skip-link, button.zed-button, a[href='/ejecuciones'], a[href='/procesar']",
        ),
      );
      return {
        ready: document.readyState,
        url: location.href,
        textLen: inner.length,
        htmlLen: body ? body.innerHTML.length : 0,
        text: inner.slice(0, 200),
        hydrated,
        // eslint-disable-next-line no-undef
        hasToken: typeof window.__ZEDAZO_SESSION_TOKEN__ === "string",
        title: document.title || "",
      };
    });
    last = JSON.stringify(snap);
    if (
      snap.ready === "complete" &&
      snap.hasToken &&
      snap.hydrated &&
      /127\.0\.0\.1:\d+/.test(String(snap.url))
    ) {
      console.log(`ready:${label} hydrated textLen=${snap.textLen}`);
      return snap;
    }
    await browser.pause(250);
  }
  throw new Error(`timeout readiness (${label}): ${last}`);
}

async function gotoLoopback(pathAndQuery) {
  const path = pathAndQuery.startsWith("/") ? pathAndQuery : `/${pathAndQuery}`;
  // browser.url (navegación WebDriver) es más fiable que location.assign en wry/WebKit:
  // assign dejaba readyState=complete + token pero DOM vacío (CI e78df058).
  const baseUrl = await browser.getUrl();
  const origin = String(baseUrl).match(/^(https?:\/\/127\.0\.0\.1:\d+)/)?.[1];
  if (!origin) {
    throw new Error(`no se pudo derivar origin de getUrl(): ${baseUrl}`);
  }
  await browser.url(`${origin}${path}`);
  await waitAppReady(`nav:${path}`, 60_000);
}

/**
 * `FileDropzone` usa `.zed-sr-only` (1×1 clipped). WebKit/WebDriver rechaza
 * clear/setValue sobre ese input («element not interactable», CI fd2781e5).
 */
async function setFileInputPath(absPath) {
  const input = await browser.$('input[type="file"]');
  if (!(await input.isExisting())) {
    throw new Error("no hay input[type=file]");
  }
  await browser.execute((el) => {
    el.classList.remove("zed-sr-only");
    el.removeAttribute("hidden");
    el.style.cssText =
      "position:fixed;left:8px;top:8px;width:240px;height:40px;opacity:1;z-index:99999;clip:auto;overflow:visible;";
  }, input);
  await input.waitForDisplayed({ timeout: 5_000 });
  await input.setValue(absPath);
  return input;
}

async function clickDescargar(timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const btn = await browser.$("button.zed-button--primary");
    const exists = await btn.isExisting().catch(() => false);
    if (exists) {
      const text = await btn.getText().catch(() => "");
      if (/Descargar/i.test(String(text))) {
        await btn.click();
        return;
      }
    }
    const all = await browser.$$("button");
    for (const b of all) {
      const t = await b.getText().catch(() => "");
      if (/Descargar/i.test(String(t))) {
        await b.click();
        return;
      }
    }
    await browser.pause(400);
  }
  const diag = await browser.execute(() => {
    // eslint-disable-next-line no-undef
    const body = document.body;
    const buttons = Array.from(document.querySelectorAll("button")).map((b) =>
      (b.textContent || "").trim().slice(0, 40),
    );
    return {
      url: location.href,
      // eslint-disable-next-line no-undef
      hasToken: typeof window.__ZEDAZO_SESSION_TOKEN__ === "string",
      text: body ? (body.innerText || body.textContent || "").slice(0, 600) : "",
      htmlLen: body ? body.innerHTML.length : 0,
      buttons: buttons.slice(0, 20),
      gate: Boolean(document.querySelector(".zed-gate")),
    };
  });
  throw new Error(
    `no se encontró botón Descargar en detalle de job: ${JSON.stringify(diag)}`,
  );
}

/** Navegación in-app (Next Link) para no perder hidratación tras browser.url. */
async function openJobDetailViaGui(jobId) {
  const side = await browser.$('a[href="/ejecuciones"]');
  if (await side.isExisting().catch(() => false)) {
    await side.click();
    await waitAppReady("ejecuciones-list", 45_000);
  } else {
    await gotoLoopback("/ejecuciones");
  }

  const href = `/ejecuciones?job=${encodeURIComponent(jobId)}`;
  const deadline = Date.now() + 45_000;
  while (Date.now() < deadline) {
    const link = await browser.$(`a[href="${href}"]`);
    if (await link.isExisting().catch(() => false)) {
      await link.click();
      await waitAppReady(`job-detail-soft:${jobId}`, 45_000);
      return;
    }
    await browser.pause(500);
  }
  const origin = String(await browser.getUrl()).match(
    /^(https?:\/\/127\.0\.0\.1:\d+)/,
  )?.[1];
  if (!origin) {
    throw new Error("no origin para fallback job-detail");
  }
  await browser.url(`${origin}${href}`);
  await waitAppReady(`job-detail-hard:${jobId}`, 60_000);
}

try {
  // Offline/unshare: WebKit puede tardar más en hidratar; 90s.
  await waitAppReady("boot", 90_000);
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

  // --- Identidad del artefacto (sin Bearer → 401; con Bearer → sha ≠ unknown en release) ---
  const verAnon = await fetch(`${base}/api/v1/version`);
  if (verAnon.status !== 401) {
    throw new Error(`version sin Bearer: esperado 401, got ${verAnon.status}`);
  }
  const verAuth = await fetch(`${base}/api/v1/version`, { headers });
  if (!verAuth.ok) {
    throw new Error(`version con Bearer HTTP ${verAuth.status}`);
  }
  const verBody = await verAuth.json();
  console.log(
    `version ok: version=${verBody.version} git_sha=${verBody.git_sha} build_date=${verBody.build_date}`,
  );

  // --- Flujo API (I7 + upload grande) ---
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

  const tenMb = 10 * 1024 * 1024;
  let big = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:big10\r\nFN:Big Ten\r\nNOTE:";
  big += "z".repeat(Math.max(0, tenMb - big.length - 20));
  big += "\r\nEND:VCARD\r\n";
  const bigBuf = Buffer.from(big);
  writeFileSync(join(work, "synthetic-10mb.vcf"), bigBuf);
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

  const hasGlobalTauri = await browser.execute(() => {
    // eslint-disable-next-line no-undef
    return typeof window.__TAURI__ !== "undefined";
  });
  if (hasGlobalTauri) {
    throw new Error("window.__TAURI__ no debe existir (withGlobalTauri=false)");
  }
  console.log("OK V-04: sin __TAURI__ global");

  // V-01 ANTES de V-03: la navegación externa bloqueada puede dejar el WebView
  // en un documento vacío (run 40279090494); no envenenar antes del detalle.
  const artApi = await fetch(`${base}/api/v1/jobs/${jobId}/artifacts/vcf`, { headers });
  if (!artApi.ok) {
    throw new Error(`artefacto vcf para V-01 HTTP ${artApi.status}`);
  }
  const artBuf = Buffer.from(await artApi.arrayBuffer());
  const artSha = createHash("sha256").update(artBuf).digest("hex");

  // Detalle vía GUI (Link cliente): evita documento «muerto» tras browser.url
  // (htmlLen>0 pero sin botones — CI c119b360).
  await openJobDetailViaGui(jobId);
  await clickDescargar();
  await browser.pause(800);
  const downloadErr = await browser.execute(() => {
    // eslint-disable-next-line no-undef
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

  // Auditoría TSV (detalle → pestaña exportar / botón Descargar TSV si existe).
  const auditApi = await fetch(`${base}/api/v1/jobs/${jobId}/artifacts/audit_tsv`, {
    headers,
  });
  if (auditApi.ok) {
    const auditText = await auditApi.text();
    if (!auditText || auditText.length < 8) {
      throw new Error("audit_tsv vacío");
    }
    console.log("OK artefacto auditoría TSV");
  } else {
    console.warn(`audit_tsv HTTP ${auditApi.status}`);
  }

  // --- Recorrido GUI: Procesar con input file ---
  await gotoLoopback("/procesar");
  await setFileInputPath(sample);
  await browser.pause(500);
  // Continuar (paso reglas) si aparece.
  for (let i = 0; i < 20; i++) {
    const cont = await browser.$$("button");
    let hit = false;
    for (const b of cont) {
      const t = await b.getText().catch(() => "");
      if (/^Continuar$/i.test(String(t).trim())) {
        await b.click();
        hit = true;
        break;
      }
    }
    if (hit) break;
    await browser.pause(300);
  }
  // Lanzar / Procesar job si el wizard lo pide.
  for (let i = 0; i < 40; i++) {
    const buttons = await browser.$$("button");
    for (const b of buttons) {
      const t = await b.getText().catch(() => "");
      if (/^(Procesar|Ejecutar|Lanzar|Iniciar)/i.test(String(t).trim())) {
        await b.click();
        console.log("OK GUI: clic procesar/lanzar");
      } else if (/Continuar/i.test(String(t))) {
        await b.click();
      }
    }
    const text = await browser.execute(() =>
      // eslint-disable-next-line no-undef
      (document.body && document.body.innerText) || "",
    );
    if (/complet|expediente|Descargar|ejecuci/i.test(String(text))) {
      console.log("OK GUI: estado post-proceso visible");
      break;
    }
    await browser.pause(500);
  }

  // Entrada inválida: VCF basura → mensaje útil.
  const badPath = join(work, "bad.vcf");
  writeFileSync(badPath, "NOT-A-VCARD\n");
  await gotoLoopback("/procesar");
  if (await browser.$('input[type="file"]').isExisting()) {
    await setFileInputPath(badPath);
    await browser.pause(400);
    for (let i = 0; i < 15; i++) {
      const buttons = await browser.$$("button");
      for (const b of buttons) {
        const t = await b.getText().catch(() => "");
        if (/Continuar|Procesar|Ejecutar|Lanzar/i.test(String(t))) {
          await b.click();
        }
      }
      const text = await browser.execute(() =>
        // eslint-disable-next-line no-undef
        (document.body && document.body.innerText) || "",
      );
      if (/error|inválid|fail|no se pudo|ApiError|HTTP/i.test(String(text))) {
        console.log("OK GUI: error útil ante entrada inválida");
        break;
      }
      await browser.pause(400);
    }
  }

  // Ciclo de vida: ir a inicio y volver.
  await gotoLoopback("/");
  await gotoLoopback("/ejecuciones");
  console.log("OK GUI: ciclo inicio ↔ ejecuciones");

  // Borrado de datos (Ajustes).
  await gotoLoopback("/ajustes");
  const wipeClicked = await browser.execute(() => {
    // eslint-disable-next-line no-undef
    const buttons = Array.from(document.querySelectorAll("button"));
    const b = buttons.find((el) => /Borrar todos los datos/i.test(el.textContent || ""));
    if (!b) return false;
    b.click();
    return true;
  });
  if (wipeClicked) {
    await browser.pause(400);
    // confirm() nativo: WebDriver puede auto-aceptar o no; registrar resultado.
    console.log("OK GUI: acción borrado invocada (confirm nativo según WebKit)");
  } else {
    console.warn("aviso: no se encontró botón de borrado en Ajustes");
  }

  // V-03 al final: no debe impedir aserciones previas.
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
  // Recuperar documento usable tras el intento externo.
  await gotoLoopback("/");

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
