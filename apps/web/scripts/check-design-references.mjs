#!/usr/bin/env node
// Control de integridad de design-references (C19): recalcula los 3 sha256
// y los compara con DESIGN-REFERENCES.sha256. Idéntico a lo que hará ArchMaker
// con su DESIGN-REFERENCES-SOURCE.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, ".."); // apps/web

const sha256 = (data) => createHash("sha256").update(data).digest("hex");
const read = (p) => readFileSync(resolve(ROOT, p), "utf8").replace(/\r\n/g, "\n");

function extract(src, startMark, endMark) {
  const s = src.indexOf(startMark);
  const e = src.indexOf(endMark);
  if (s === -1 || e === -1) {
    throw new Error(`marcador no encontrado: ${startMark} / ${endMark}`);
  }
  return src.slice(s, e + endMark.length);
}

// 1. Bloque @layer design-references (marcadores incluidos).
const nucleus = extract(
  read("src/design-system/design-references.css"),
  "/* design-references v1.0.0 BEGIN */",
  "/* design-references v1.0.0 END */",
);

// 2. Script de arranque del tema (marcadores incluidos).
const boot = extract(
  read("src/lib/theme.ts"),
  "/* design-references v1.0.0 theme-boot BEGIN */",
  "/* design-references v1.0.0 theme-boot END */",
);

// 3. Test de contraste (fichero entero).
const contrast = read("src/lib/contrast.test.mjs");

const actual = [sha256(nucleus), sha256(boot), sha256(contrast)];

const raw = read("src/design-system/DESIGN-REFERENCES.sha256");
const expected = raw
  .split("\n")
  .map((l) => l.trim())
  .filter((l) => l !== "" && !l.startsWith("#"))
  .map((l) => l.split(/\s+/)[0]);

if (expected.length !== 3) {
  console.error(
    `DESIGN-REFERENCES.sha256 debe declarar 3 sha256; encontré ${expected.length}.`,
  );
  process.exit(1);
}

let ok = true;
actual.forEach((a, i) => {
  if (a !== expected[i]) {
    ok = false;
    console.error(`[${i}] esperado ${expected[i]} · actual ${a}`);
  }
});

if (!ok) {
  process.exit(1);
}
console.log("design-references sha256 OK (3/3)");
