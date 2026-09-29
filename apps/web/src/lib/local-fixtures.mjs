/**
 * Fixtures sintéticos del adaptador local. Sin PII.
 * La GUI los usa solo cuando el health check de la API falla.
 */

export const REAL_VCF_MESSAGE =
  "Procesar un VCF real exige la API (zedazo-api). El adaptador local solo muestra fixtures sintéticos.";

export const LOCAL_FIXTURE_KEY = "zedazo-local-fixture";

export const SYNTHETIC_JOB_ID = "job-sintetico";

const FIXTURES = new Set(["vacio", "error", "resultado"]);

const ERROR_MESSAGE = "Fixture sintético: no se pudo leer el expediente.";

function resultJob() {
  return {
    job_id: SYNTHETIC_JOB_ID,
    status: "completed",
    display_name: "agenda-sintetica.vcf",
    created_at: "2026-09-12T00:00:00Z",
    completed_at: "2026-09-12T00:01:00Z",
    core_version: "fixture-local",
    summary: {
      input_contacts: 3,
      retained: 2,
      needs_review: 1,
      eliminated: 0,
      quarantine: 0,
      duplicate_groups: 1,
    },
    artifacts: ["vcf", "audit_tsv", "stats_json"],
    retention_hours: 24,
    rules: { mode: "builtin", sha256: "b".repeat(64) },
    input: {
      original_name: "agenda-sintetica.vcf",
      sha256: "a".repeat(64),
      bytes: 256,
      upload_id: "upload-sintetico",
      source_detected: "generic",
      vcard_version: "4.0",
    },
  };
}

const CONTACTS = [
  {
    uid: "uid-sintetico-1",
    fn_value: "Ada Sintética",
    result: "conserved",
    screening_rule: "C1",
    categories: ["TECH-SW"],
    emails: ["ada@example.invalid"],
    tels: ["+34000000001"],
    merged_uids: [],
  },
  {
    uid: "uid-sintetico-2",
    fn_value: "Ben Sintético",
    result: "needs_review",
    screening_rule: null,
    categories: ["PROF-JUD"],
    emails: ["ben@example.invalid"],
    tels: ["+34000000002"],
    merged_uids: ["uid-sintetico-1"],
  },
];

const AUDIT = [
  { cols: ["conserved", "C1", "categoría TECH-SW", "uid-sintetico-1"] },
  { cols: ["needs_review", "D2", "FN fuzzy sintético", "uid-sintetico-2"] },
];

/**
 * @param {string} search
 * @param {string | null | undefined} stored
 * @returns {"vacio" | "error" | "resultado"}
 */
export function readFixture(search, stored) {
  const query = new URLSearchParams(search.startsWith("?") ? search.slice(1) : search).get(
    "fixture",
  );
  if (query && FIXTURES.has(query)) return /** @type {"vacio" | "error" | "resultado"} */ (query);
  if (stored && FIXTURES.has(stored)) return /** @type {"vacio" | "error" | "resultado"} */ (stored);
  return "resultado";
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function assertResult(fixture, jobId) {
  if (fixture === "error") {
    throw new Error(ERROR_MESSAGE);
  }
  if (fixture !== "resultado" || jobId !== SYNTHETIC_JOB_ID) {
    throw new Error("Fixture sintético: expediente no encontrado.");
  }
}

/** @type {ReturnType<typeof resultJob>[]} */
let jobs = [resultJob()];

export function resetLocalFixtures() {
  jobs = [resultJob()];
}

export function localListJobs(fixture) {
  if (fixture === "error") throw new Error(ERROR_MESSAGE);
  if (fixture === "vacio") return { items: [] };
  return { items: jobs.map((job) => clone(job)) };
}

export function localGetJob(fixture, jobId) {
  assertResult(fixture, jobId);
  const job = jobs.find((item) => item.job_id === jobId);
  if (!job) throw new Error("Fixture sintético: expediente no encontrado.");
  return { job: clone(job), warnings: [] };
}

export function localListContacts(fixture, jobId, q, result) {
  assertResult(fixture, jobId);
  const query = (q || "").toLowerCase();
  const items = CONTACTS.filter((contact) => {
    const haystack = `${contact.fn_value} ${contact.uid}`.toLowerCase();
    const matchQ = !query || haystack.includes(query);
    const matchResult = !result || contact.result === result;
    return matchQ && matchResult;
  });
  return { items: items.map((item) => clone(item)) };
}

export function localListDuplicates(fixture, jobId) {
  assertResult(fixture, jobId);
  return {
    groups: [
      {
        canonical_uid: "uid-sintetico-1",
        member_uids: ["uid-sintetico-1", "uid-sintetico-2"],
      },
    ],
  };
}

export function localGetAudit(fixture, jobId) {
  assertResult(fixture, jobId);
  return { items: AUDIT.map((row) => clone(row)) };
}

export function localGetStats(fixture, jobId) {
  assertResult(fixture, jobId);
  return {
    input_contacts: 3,
    retained: 2,
    needs_review: 1,
    eliminated: 0,
  };
}

export function localCancelJob(fixture, jobId) {
  assertResult(fixture, jobId);
  const job = jobs.find((item) => item.job_id === jobId);
  if (job) job.status = "cancelled";
}

export function localDeleteJob(fixture, jobId) {
  if (fixture === "error") throw new Error(ERROR_MESSAGE);
  jobs = jobs.filter((item) => item.job_id !== jobId);
}

export function localValidateRules(toml) {
  if (!toml.trim()) {
    return {
      ok: false,
      diagnostics: [{ message: "El TOML está vacío (fixture sintético)." }],
    };
  }
  return {
    ok: true,
    diagnostics: [
      { message: "Adaptador local: el parser del core no se ha ejecutado." },
    ],
  };
}

export function localArtifactText(jobId, kind) {
  return `fixture sintetico\njob=${jobId}\nartefacto=${kind}\n`;
}

export function localHealth() {
  return {
    status: "adaptador-local",
    api_version: "n/a",
    core_version: "n/a",
    storage_mode: "fixture",
  };
}
