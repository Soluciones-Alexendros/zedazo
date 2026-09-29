declare module "@/lib/local-fixtures.mjs" {
  export const REAL_VCF_MESSAGE: string;
  export const LOCAL_FIXTURE_KEY: string;
  export const SYNTHETIC_JOB_ID: string;

  export type LocalFixture = "vacio" | "error" | "resultado";

  export function readFixture(
    search: string,
    stored: string | null | undefined,
  ): LocalFixture;

  export function resetLocalFixtures(): void;
  export function localListJobs(fixture: LocalFixture): { items: unknown[] };
  export function localGetJob(
    fixture: LocalFixture,
    jobId: string,
  ): { job: unknown; warnings: { code: string; message: string }[] };
  export function localListContacts(
    fixture: LocalFixture,
    jobId: string,
    q?: string,
    result?: string,
  ): { items: unknown[] };
  export function localListDuplicates(
    fixture: LocalFixture,
    jobId: string,
  ): { groups: { canonical_uid: string; member_uids: string[] }[] };
  export function localGetAudit(
    fixture: LocalFixture,
    jobId: string,
  ): { items: { cols: string[] }[] };
  export function localGetStats(
    fixture: LocalFixture,
    jobId: string,
  ): Record<string, unknown>;
  export function localCancelJob(fixture: LocalFixture, jobId: string): void;
  export function localDeleteJob(fixture: LocalFixture, jobId: string): void;
  export function localValidateRules(toml: string): {
    ok: boolean;
    diagnostics: { message: string }[];
  };
  export function localArtifactText(jobId: string, kind: string): string;
  export function localHealth(): {
    status: string;
    api_version: string;
    core_version: string;
    storage_mode: string;
  };
}
