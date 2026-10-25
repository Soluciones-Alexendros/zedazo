#!/usr/bin/env node
/**
 * Sirve el export estático `out/` para Playwright y Docker.
 * - SPA fallback solo para GUI (nunca /api/*).
 * - Proxy opcional /api → ZEDAZO_API_ORIGIN (e2e-real-api).
 */
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const OUT = path.resolve(ROOT, process.env.ZEDAZO_OUT_DIR || "out");
const PORT = Number(process.env.PORT || 3000);
const HOST = process.env.HOST || "127.0.0.1";
const API_ORIGIN = (process.env.ZEDAZO_API_ORIGIN || "http://127.0.0.1:8080").replace(
  /\/$/,
  "",
);
const PROXY_API = process.env.ZEDAZO_PROXY_API === "1";

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
  ".woff": "font/woff",
  ".woff2": "font/woff2",
  ".txt": "text/plain; charset=utf-8",
  ".map": "application/json",
  ".webp": "image/webp",
};

function safeJoin(root, urlPath) {
  const decoded = decodeURIComponent(urlPath.split("?")[0]);
  // Quitar `/` inicial: path.join ignora `root` si el 2º arg es absoluto.
  const cleaned = path
    .normalize(decoded)
    .replace(/^(\.\.[/\\])+/, "")
    .replace(/^[/\\]+/, "");
  const full = path.join(root, cleaned);
  if (!full.startsWith(root + path.sep) && full !== root) return null;
  return full;
}

function tryFile(candidates) {
  for (const c of candidates) {
    if (c && fs.existsSync(c) && fs.statSync(c).isFile()) return c;
  }
  return null;
}

function resolveGui(urlPath) {
  const base = safeJoin(OUT, urlPath);
  if (!base) return null;
  if (fs.existsSync(base) && fs.statSync(base).isFile()) return base;
  if (fs.existsSync(base) && fs.statSync(base).isDirectory()) {
    const index = path.join(base, "index.html");
    if (fs.existsSync(index)) return index;
  }
  // Next export sin trailingSlash: /ejecuciones → ejecuciones.html
  const asHtml = `${base}.html`;
  if (fs.existsSync(asHtml) && fs.statSync(asHtml).isFile()) return asHtml;
  // SPA fallback solo GUI (ruta desconocida → index.html)
  const index = path.join(OUT, "index.html");
  if (fs.existsSync(index)) return index;
  return null;
}

async function proxyApi(req, res) {
  const target = `${API_ORIGIN}${req.url}`;
  const headers = { ...req.headers, host: new URL(API_ORIGIN).host };
  delete headers["content-length"];
  const init = {
    method: req.method,
    headers,
    duplex: "half",
  };
  if (req.method !== "GET" && req.method !== "HEAD") {
    init.body = req;
  }
  let upstream;
  try {
    upstream = await fetch(target, init);
  } catch (err) {
    res.writeHead(502, { "content-type": "text/plain; charset=utf-8" });
    res.end(`proxy error: ${err}`);
    return;
  }
  const outHeaders = {};
  upstream.headers.forEach((v, k) => {
    if (k === "transfer-encoding") return;
    outHeaders[k] = v;
  });
  res.writeHead(upstream.status, outHeaders);
  if (!upstream.body) {
    res.end();
    return;
  }
  const buf = Buffer.from(await upstream.arrayBuffer());
  res.end(buf);
}

function sendFile(res, filePath) {
  const ext = path.extname(filePath).toLowerCase();
  const type = MIME[ext] || "application/octet-stream";
  res.writeHead(200, { "content-type": type });
  fs.createReadStream(filePath).pipe(res);
}

if (!fs.existsSync(OUT)) {
  console.error(`serve-out: falta ${OUT} (ejecuta next build / output export)`);
  process.exit(1);
}

const server = http.createServer((req, res) => {
  const url = req.url || "/";
  if (PROXY_API && url.startsWith("/api")) {
    void proxyApi(req, res);
    return;
  }
  if (url.startsWith("/api")) {
    res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    res.end("not found");
    return;
  }
  const pathname = new URL(url, "http://127.0.0.1").pathname;
  const file = resolveGui(pathname === "/" ? "/index.html" : pathname);
  if (!file) {
    res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    res.end("not found");
    return;
  }
  sendFile(res, file);
});

server.listen(PORT, HOST, () => {
  console.log(
    `serve-out: http://${HOST}:${PORT} root=${OUT} proxy_api=${PROXY_API ? API_ORIGIN : "off"}`,
  );
});
