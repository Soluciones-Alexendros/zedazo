import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const WEB_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const REPO_ROOT = path.resolve(WEB_ROOT, "../..");

const C21_HASH = "sha256-wZHa1Ad+sG4CVkMj20Js/YFPWl/cxb0RiIYwFGB1XXY=";

test("el script de arranque coincide con el hash C21", async () => {
  const theme = await readFile(path.join(WEB_ROOT, "src/lib/theme.ts"), "utf8");
  const m = theme.match(/THEME_BOOT_SCRIPT = `([\s\S]*?)`;/);
  assert.ok(m, "no se encontró THEME_BOOT_SCRIPT");
  const script = m[1];
  const b64 = createHash("sha256").update(script, "utf8").digest("base64");
  assert.equal(`sha256-${b64}`, C21_HASH);
});

test("security_headers.rs autoriza el script por hash y sin 'unsafe-inline'", async () => {
  const headers = await readFile(
    path.join(REPO_ROOT, "crates/zedazo-api/src/security_headers.rs"),
    "utf8",
  );
  const scriptSrc = headers.match(/script-src\s+([^;]+);/);
  assert.ok(scriptSrc, "no se encontró script-src en security_headers.rs");
  const src = scriptSrc[1];
  assert.ok(src.includes(C21_HASH), "script-src no lleva el hash C21");
  assert.ok(!src.includes("'unsafe-inline'"), "script-src aún tiene 'unsafe-inline'");
});
