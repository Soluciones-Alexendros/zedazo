import {
  artifactUrl as remoteArtifactUrl,
  cancelJob as remoteCancelJob,
  createAudit as remoteCreateAudit,
  createJob as remoteCreateJob,
  deleteJob as remoteDeleteJob,
  getAudit as remoteGetAudit,
  getHealth,
  getJob as remoteGetJob,
  getStats as remoteGetStats,
  listContacts as remoteListContacts,
  listDuplicates as remoteListDuplicates,
  listJobs as remoteListJobs,
  logoutSession as remoteLogout,
  uploadVcf as remoteUploadVcf,
  validateRules as remoteValidateRules,
  wipeAllData as remoteWipe,
  type ContactView,
  type JobManifest,
} from "@/lib/api";
import {
  LOCAL_FIXTURE_KEY,
  REAL_VCF_MESSAGE,
  localArtifactText,
  localCancelJob,
  localDeleteJob,
  localGetAudit,
  localGetJob,
  localGetStats,
  localHealth,
  localListContacts,
  localListDuplicates,
  localListJobs,
  localValidateRules,
  readFixture,
  type LocalFixture,
} from "@/lib/local-fixtures.mjs";

export type AdapterKind = "remote" | "local";
export type { LocalFixture };

let probe: Promise<AdapterKind> | null = null;
let kind: AdapterKind | null = null;

export function resetAdapterProbe() {
  probe = null;
  kind = null;
}

/** Remoto si NEXT_PUBLIC_API_BASE responde al health; local si no hay API. */
export function probeAdapter(): Promise<AdapterKind> {
  if (!probe) {
    probe = getHealth()
      .then(() => {
        kind = "remote";
        return "remote" as const;
      })
      .catch(() => {
        kind = "local";
        return "local" as const;
      });
  }
  return probe;
}

export function currentFixture(): LocalFixture {
  if (typeof window === "undefined") return "resultado";
  const fixture = readFixture(
    window.location.search,
    window.sessionStorage.getItem(LOCAL_FIXTURE_KEY),
  );
  window.sessionStorage.setItem(LOCAL_FIXTURE_KEY, fixture);
  return fixture;
}

async function mode(): Promise<AdapterKind> {
  return probeAdapter();
}

export async function listJobs() {
  if ((await mode()) === "remote") return remoteListJobs();
  return localListJobs(currentFixture()) as { items: JobManifest[] };
}

export async function getJob(jobId: string) {
  if ((await mode()) === "remote") return remoteGetJob(jobId);
  return localGetJob(currentFixture(), jobId) as {
    job: JobManifest;
    warnings: { code: string; message: string }[];
  };
}

export async function listContacts(jobId: string, q?: string, result?: string) {
  if ((await mode()) === "remote") return remoteListContacts(jobId, q, result);
  return localListContacts(currentFixture(), jobId, q, result) as {
    items: ContactView[];
  };
}

export async function listDuplicates(jobId: string) {
  if ((await mode()) === "remote") return remoteListDuplicates(jobId);
  return localListDuplicates(currentFixture(), jobId);
}

export async function getAudit(jobId: string) {
  if ((await mode()) === "remote") return remoteGetAudit(jobId);
  return localGetAudit(currentFixture(), jobId);
}

export async function getStats(jobId: string) {
  if ((await mode()) === "remote") return remoteGetStats(jobId);
  return localGetStats(currentFixture(), jobId);
}

export async function cancelJob(jobId: string) {
  if ((await mode()) === "remote") return remoteCancelJob(jobId);
  localCancelJob(currentFixture(), jobId);
}

export async function deleteJob(jobId: string) {
  if ((await mode()) === "remote") return remoteDeleteJob(jobId);
  localDeleteJob(currentFixture(), jobId);
}

export async function uploadVcf(file: File) {
  if ((await mode()) === "remote") return remoteUploadVcf(file);
  throw new Error(REAL_VCF_MESSAGE);
}

export async function createJob(
  body: Parameters<typeof remoteCreateJob>[0],
) {
  if ((await mode()) === "remote") return remoteCreateJob(body);
  throw new Error(REAL_VCF_MESSAGE);
}

export async function createAudit(uploadId: string, configToml?: string) {
  if ((await mode()) === "remote") return remoteCreateAudit(uploadId, configToml);
  throw new Error(REAL_VCF_MESSAGE);
}

export async function validateRules(toml: string) {
  if ((await mode()) === "remote") return remoteValidateRules(toml);
  return localValidateRules(toml);
}

export async function wipeAllData() {
  if ((await mode()) === "remote") return remoteWipe();
  throw new Error(REAL_VCF_MESSAGE);
}

export async function logoutSession() {
  if ((await mode()) === "remote") return remoteLogout();
  throw new Error("Cerrar sesión exige la API.");
}

export async function readHealth() {
  if ((await mode()) === "remote") return getHealth();
  return localHealth();
}

export function artifactUrl(jobId: string, artifactKind: string) {
  if (kind === "local") {
    const text = localArtifactText(jobId, artifactKind);
    return `data:text/plain;charset=utf-8,${encodeURIComponent(text)}`;
  }
  return remoteArtifactUrl(jobId, artifactKind);
}

export { REAL_VCF_MESSAGE };
