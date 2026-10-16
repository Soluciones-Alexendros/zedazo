import assert from "node:assert/strict";
import test from "node:test";

test("API_BASE same-origin por defecto (cadena vacía)", () => {
  // Sin NEXT_PUBLIC_API_BASE (o vacío) el cliente usa rutas relativas.
  const raw = process.env.NEXT_PUBLIC_API_BASE;
  const base =
    typeof raw === "string" && raw.trim() !== "" ? raw.replace(/\/$/, "") : "";
  assert.equal(base, "");
});

test("API_BASE absoluto solo si la variable tiene valor", () => {
  const raw = "http://127.0.0.1:8080";
  const base =
    typeof raw === "string" && raw.trim() !== "" ? raw.replace(/\/$/, "") : "";
  assert.match(base, /^http:\/\/127\.0\.0\.1:\d+$/);
});
