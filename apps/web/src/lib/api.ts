/**
 * Base de la API. Same-origin por defecto (rutas relativas `/api/v1/...`).
 * En desarrollo, `next.config` reescribe `/api/*` hacia la API en loopback.
 * Solo se usa un origen absoluto si `NEXT_PUBLIC_API_BASE` está definido y no vacío
 * (p. ej. pruebas aisladas); la cadena vacía no cuenta como absoluta.
 */
function resolveApiBase(): string {
  const raw = process.env.NEXT_PUBLIC_API_BASE;
  if (typeof raw === "string" && raw.trim() !== "") {
    return raw.replace(/\/$/, "");
  }
  return "";
}

export const API_BASE = resolveApiBase();

export class ApiError extends Error {
  status: number;
  body: string;

  constructor(status: number, body: string) {
    super(body || `HTTP ${status}`);
    this.name = "ApiError";
    this.status = status;
    this.body = body;
  }
}

export type JobManifest = {
  job_id: string;
  status: string;
  display_name?: string | null;
  created_at: string;
  started_at?: string | null;
  completed_at?: string | null;
  core_version?: string;
  summary?: {
    input_contacts: number;
    retained: number;
    needs_review: number;
    eliminated: number;
    quarantine: number;
    duplicate_groups: number;
  } | null;
  artifacts: string[];
  error?: string | null;
  retention_hours?: number;
  rules?: {
    mode: string;
    sha256?: string | null;
  };
  input: {
    original_name: string;
    sha256: string;
    bytes: number;
    upload_id: string;
    source_detected?: string | null;
    vcard_version?: string | null;
  };
};

export type ContactView = {
  uid: string;
  fn_value: string;
  result: string;
  screening_rule?: string | null;
  categories: string[];
  emails: string[];
  tels: string[];
  merged_uids: string[];
};

const defaultInit: RequestInit = {
  credentials: "include",
};

async function parseJson<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const text = await res.text();
    throw new ApiError(res.status, text || res.statusText);
  }
  return res.json() as Promise<T>;
}

export async function getHealth() {
  const res = await fetch(`${API_BASE}/api/v1/health`, defaultInit);
  return parseJson<{
    status: string;
    api_version: string;
    core_version: string;
    storage_mode: string;
  }>(res);
}

/** Sondea sesión (protegido). 401 → requiere login en modo token. */
export async function getVersion() {
  const res = await fetch(`${API_BASE}/api/v1/version`, defaultInit);
  return parseJson<{
    version: string;
    git_sha: string;
    build_date: string;
    schema_version?: string;
  }>(res);
}

export async function loginWithToken(token: string) {
  const res = await fetch(`${API_BASE}/api/v1/auth/login`, {
    ...defaultInit,
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ token }),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new ApiError(res.status, text || res.statusText);
  }
}

export async function logoutSession() {
  const res = await fetch(`${API_BASE}/api/v1/auth/logout`, {
    ...defaultInit,
    method: "POST",
  });
  if (!res.ok && res.status !== 204) {
    const text = await res.text();
    throw new ApiError(res.status, text || res.statusText);
  }
}

export async function uploadVcf(file: File) {
  const fd = new FormData();
  fd.append("file", file);
  const res = await fetch(`${API_BASE}/api/v1/uploads`, {
    ...defaultInit,
    method: "POST",
    body: fd,
  });
  return parseJson<{
    upload_id: string;
    original_name: string;
    bytes: number;
    sha256: string;
  }>(res);
}

export async function createJob(body: {
  upload_id: string;
  display_name?: string;
  artifacts?: string[];
  rules?: { mode: string; toml?: string };
  retention_hours?: number;
}) {
  const res = await fetch(`${API_BASE}/api/v1/jobs`, {
    ...defaultInit,
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return parseJson<JobManifest>(res);
}

export async function listJobs() {
  const res = await fetch(`${API_BASE}/api/v1/jobs`, defaultInit);
  return parseJson<{ items: JobManifest[] }>(res);
}

export async function getJob(jobId: string) {
  const res = await fetch(`${API_BASE}/api/v1/jobs/${jobId}`, defaultInit);
  return parseJson<{
    job: JobManifest;
    warnings: { code: string; message: string }[];
  }>(res);
}

export async function cancelJob(jobId: string) {
  const res = await fetch(`${API_BASE}/api/v1/jobs/${jobId}/cancel`, {
    ...defaultInit,
    method: "POST",
  });
  if (!res.ok) throw new ApiError(res.status, await res.text());
}

export async function wipeAllData() {
  const res = await fetch(`${API_BASE}/api/v1/admin/wipe`, {
    ...defaultInit,
    method: "POST",
  });
  return parseJson<{ wiped: boolean }>(res);
}

export async function deleteJob(jobId: string) {
  const res = await fetch(`${API_BASE}/api/v1/jobs/${jobId}`, {
    ...defaultInit,
    method: "DELETE",
  });
  if (!res.ok && res.status !== 204) {
    throw new ApiError(res.status, await res.text());
  }
}

export async function listContacts(jobId: string, q?: string, result?: string) {
  const params = new URLSearchParams();
  if (q) params.set("q", q);
  if (result) params.set("result", result);
  const res = await fetch(
    `${API_BASE}/api/v1/jobs/${jobId}/contacts?${params}`,
    defaultInit,
  );
  return parseJson<{ items: ContactView[] }>(res);
}

export async function listDuplicates(jobId: string) {
  const res = await fetch(
    `${API_BASE}/api/v1/jobs/${jobId}/duplicates`,
    defaultInit,
  );
  return parseJson<{
    groups: { canonical_uid: string; member_uids: string[] }[];
  }>(res);
}

export async function getAudit(jobId: string) {
  const res = await fetch(`${API_BASE}/api/v1/jobs/${jobId}/audit`, defaultInit);
  return parseJson<{ items: { cols: string[] }[] }>(res);
}

export async function getStats(jobId: string) {
  const res = await fetch(`${API_BASE}/api/v1/jobs/${jobId}/stats`, defaultInit);
  return parseJson<Record<string, unknown>>(res);
}

export function artifactUrl(jobId: string, kind: string) {
  return `${API_BASE}/api/v1/jobs/${jobId}/artifacts/${kind}`;
}

export async function createAudit(upload_id: string, config_toml?: string) {
  const res = await fetch(`${API_BASE}/api/v1/audits`, {
    ...defaultInit,
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ upload_id, config_toml }),
  });
  return parseJson<JobManifest>(res);
}

export async function validateRules(toml: string) {
  const res = await fetch(`${API_BASE}/api/v1/rules/validate`, {
    ...defaultInit,
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ toml }),
  });
  return parseJson<{ ok: boolean; diagnostics: { message: string }[] }>(res);
}

export function eventsUrl(jobId: string, lastEventId?: string | null) {
  const base = `${API_BASE}/api/v1/jobs/${jobId}/events`;
  if (lastEventId) {
    return `${base}?last_event_id=${encodeURIComponent(lastEventId)}`;
  }
  return base;
}

export function isTerminalStatus(status: string) {
  return /^(completed|failed|cancelled|expired|deleted)$/i.test(status);
}

export function isCancellableStatus(status: string) {
  return (
    !isTerminalStatus(status) && !/^cancel_requested$/i.test(status)
  );
}
