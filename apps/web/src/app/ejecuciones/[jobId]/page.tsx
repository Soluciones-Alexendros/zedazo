"use client";

import { useParams } from "next/navigation";
import { useEffect, useMemo, useState } from "react";
import { isCancellableStatus, type ContactView, type JobManifest } from "@/lib/api";
import {
  artifactUrl,
  cancelJob,
  getAudit,
  getJob,
  getStats,
  listContacts,
  listDuplicates,
} from "@/lib/data-adapter";
import { PageHeader } from "@/components/shell/page-header";
import { JobSummary } from "@/components/jobs/job-summary";
import { JobStatus } from "@/components/jobs/job-status";
import { RetentionNotice } from "@/components/jobs/retention-notice";
import { ContactTable } from "@/components/contacts/contact-table";
import { ContactDrawer } from "@/components/contacts/contact-drawer";
import { DuplicateGroup } from "@/components/contacts/duplicate-group";
import { AuditTimeline } from "@/components/audit/audit-timeline";
import { ArtifactDownload } from "@/components/ui/artifact-download";
import { MetadataList } from "@/components/ui/metadata-list";
import { Card } from "@/components/ui/card";
import { Callout } from "@/components/ui/callout";
import { ErrorState } from "@/components/ui/error-state";
import { LoadingState } from "@/components/ui/loading-state";
import { FilterBar } from "@/components/ui/filter-bar";
import { Button, buttonClassName } from "@/components/ui/button";

type Tab =
  | "resumen"
  | "contactos"
  | "duplicados"
  | "auditoria"
  | "estadisticas"
  | "exportar"
  | "metadatos";

const ARTIFACT_META: Record<string, { label: string; description: string }> = {
  vcf: { label: "VCF", description: "Agenda resultante" },
  audit_tsv: { label: "TSV auditoría", description: "Trazas de decisión" },
  stats_json: { label: "JSON estadísticas", description: "Conteos estructurados" },
  stats_markdown: { label: "Markdown", description: "Resumen legible" },
  csv: { label: "CSV", description: "Contactos en CSV" },
  json: { label: "JSON", description: "Contactos en JSON" },
};

export default function JobDetailPage() {
  const params = useParams();
  const jobId = String(params.jobId);
  const [tab, setTab] = useState<Tab>("resumen");
  const [job, setJob] = useState<JobManifest | null>(null);
  const [contacts, setContacts] = useState<ContactView[]>([]);
  const [selected, setSelected] = useState<ContactView | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [q, setQ] = useState("");
  const [result, setResult] = useState("");
  const [dups, setDups] = useState<
    { canonical_uid: string; member_uids: string[] }[]
  >([]);
  const [audit, setAudit] = useState<{ cols: string[] }[]>([]);
  const [auditTechnical, setAuditTechnical] = useState(false);
  const [stats, setStats] = useState<Record<string, unknown> | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [warnings, setWarnings] = useState<
    { code: string; message: string }[]
  >([]);

  useEffect(() => {
    void getJob(jobId)
      .then((d) => {
        setJob(d.job);
        setWarnings(d.warnings || []);
      })
      .catch((e) => setError(String(e)));
  }, [jobId]);

  useEffect(() => {
    if (tab === "contactos") {
      void listContacts(jobId, q || undefined, result || undefined)
        .then((d) => setContacts(d.items))
        .catch((e) => setError(String(e)));
    }
    if (tab === "duplicados") {
      void listDuplicates(jobId)
        .then((d) => setDups(d.groups))
        .catch((e) => setError(String(e)));
      void getAudit(jobId)
        .then((d) => setAudit(d.items))
        .catch((e) => setError(String(e)));
    }
    if (tab === "auditoria") {
      void getAudit(jobId)
        .then((d) => setAudit(d.items))
        .catch((e) => setError(String(e)));
    }
    if (tab === "estadisticas") {
      void getStats(jobId)
        .then(setStats)
        .catch((e) => setError(String(e)));
    }
  }, [tab, jobId, q, result]);

  const tabs: { id: Tab; label: string }[] = useMemo(
    () => [
      { id: "resumen", label: "Resumen" },
      { id: "contactos", label: "Contactos" },
      { id: "duplicados", label: "Duplicados" },
      { id: "auditoria", label: "Auditoría" },
      { id: "estadisticas", label: "Estadísticas" },
      { id: "exportar", label: "Exportar" },
      { id: "metadatos", label: "Metadatos" },
    ],
    [],
  );

  if (!job) {
    return error ? (
      <ErrorState message={error} />
    ) : (
      <LoadingState label="Cargando expediente…" />
    );
  }

  const numericStats = stats
    ? Object.entries(stats).filter(
        ([, v]) => typeof v === "number",
      ) as [string, number][]
    : [];
  const maxStat = Math.max(1, ...numericStats.map(([, v]) => v));

  return (
    <div className="zed-stack">
      <PageHeader
        title={job.display_name || job.job_id}
        description={`${job.created_at}${
          job.input.source_detected ? ` · Fuente: ${job.input.source_detected}` : ""
        }${job.input.vcard_version ? ` · vCard ${job.input.vcard_version}` : ""}`}
        actions={
          <>
            <JobStatus status={job.status} />
            {isCancellableStatus(job.status) ? (
              <Button
                variant="secondary"
                onClick={async () => {
                  await cancelJob(jobId);
                  const d = await getJob(jobId);
                  setJob(d.job);
                  setWarnings(d.warnings || []);
                }}
              >
                Cancelar
              </Button>
            ) : null}
            <a
              className={buttonClassName({ variant: "primary" })}
              href={artifactUrl(jobId, "vcf")}
            >
              Descargar VCF
            </a>
          </>
        }
      />

      {error ? <ErrorState message={error} /> : null}

      <div className="zed-tabs" role="tablist" aria-label="Secciones del expediente">
        {tabs.map((t) => (
          <button
            key={t.id}
            type="button"
            role="tab"
            aria-selected={tab === t.id}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </div>

      {tab === "resumen" && (
        <div className="zed-stack">
          <JobSummary job={job} />
          <RetentionNotice hours={job.retention_hours} />
          {warnings.length > 0 ? (
            <Callout variant="warning" title="Avisos">
              <ul className="zed-prose-list zed-flush">
                {warnings.map((w) => (
                  <li key={`${w.code}-${w.message}`}>
                    <code className="zed-mono">{w.code}</code>: {w.message}
                  </li>
                ))}
              </ul>
            </Callout>
          ) : (
            <Callout variant="verification" title="Verificación">
              La verificación I1–I7 está integrada en el pipeline del core.
            </Callout>
          )}
          <Card variant="document">
            <h3>Artefactos</h3>
            <p className="zed-muted">
              {(job.artifacts.length
                ? job.artifacts
                : ["vcf", "audit_tsv", "stats_json", "csv", "json"]
              ).join(", ")}
            </p>
          </Card>
        </div>
      )}

      {tab === "contactos" && (
        <div className="zed-stack">
          <FilterBar>
            <label className="zed-sr-only" htmlFor="contact-q">
              Buscar contactos
            </label>
            <input
              id="contact-q"
              className="zed-input zed-input--filter"
              placeholder="Nombre, email o teléfono"
              value={q}
              onChange={(e) => setQ(e.target.value)}
            />
            <label className="zed-sr-only" htmlFor="contact-result">
              Resultado
            </label>
            <select
              id="contact-result"
              className="zed-input zed-input--auto"
              value={result}
              onChange={(e) => setResult(e.target.value)}
            >
              <option value="">Todos</option>
              <option value="conserved">Conservado</option>
              <option value="needs_review">Revisar</option>
              <option value="eliminated">Descartado</option>
              <option value="quarantine">Cuarentena</option>
            </select>
          </FilterBar>
          <ContactTable
            contacts={contacts}
            onSelect={(c) => {
              setSelected(c);
              setDrawerOpen(true);
            }}
          />
          <ContactDrawer
            contact={selected}
            open={drawerOpen}
            onClose={() => setDrawerOpen(false)}
          />
        </div>
      )}

      {tab === "duplicados" && (
        <DuplicateGroup groups={dups} auditRows={audit} />
      )}

      {tab === "auditoria" && (
        <div className="zed-stack">
          <div className="zed-row">
            <Button
              variant={auditTechnical ? "secondary" : "primary"}
              onClick={() => setAuditTechnical(false)}
            >
              Vista humana
            </Button>
            <Button
              variant={auditTechnical ? "primary" : "secondary"}
              onClick={() => setAuditTechnical(true)}
            >
              Vista técnica
            </Button>
            <a
              className={buttonClassName({ variant: "tertiary" })}
              href={artifactUrl(jobId, "audit_tsv")}
            >
              Descargar TSV
            </a>
          </div>
          <AuditTimeline items={audit} technical={auditTechnical} />
        </div>
      )}

      {tab === "estadisticas" && (
        <Card variant="document">
          {numericStats.length === 0 ? (
            <pre className="zed-mono">{JSON.stringify(stats, null, 2)}</pre>
          ) : (
            <div className="zed-stack">
              <table className="zed-sr-only">
                <caption>Estadísticas numéricas</caption>
                <tbody>
                  {numericStats.map(([k, v]) => (
                    <tr key={k}>
                      <th scope="row">{k}</th>
                      <td>{v}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {numericStats.map(([k, v]) => (
                <div key={k}>
                  <div className="zed-row zed-row--spread">
                    <span>{k}</span>
                    <span className="zed-stat-value zed-stat-bar__value">{v}</span>
                  </div>
                  <div className="zed-bar" aria-hidden>
                    <div
                      className="zed-bar__fill"
                      style={{ width: `${(v / maxStat) * 100}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>
      )}

      {tab === "exportar" && (
        <Card variant="document">
          {(job.artifacts.length
            ? job.artifacts
            : ["vcf", "audit_tsv", "stats_json", "csv", "json"]
          ).map((k) => (
            <ArtifactDownload
              key={k}
              href={artifactUrl(jobId, k)}
              label={ARTIFACT_META[k]?.label || k}
              description={ARTIFACT_META[k]?.description}
            />
          ))}
        </Card>
      )}

      {tab === "metadatos" && (
        <Card variant="document">
          <MetadataList
            items={[
              { label: "ID de ejecución", value: job.job_id, mono: true, copyable: true },
              {
                label: "Hash input",
                value: job.input.sha256,
                mono: true,
                copyable: true,
              },
              {
                label: "Hash reglas",
                value: job.rules?.sha256 || "—",
                mono: true,
                copyable: Boolean(job.rules?.sha256),
              },
              { label: "Core", value: job.core_version || "—", mono: true },
              {
                label: "Fuente",
                value: job.input.source_detected || "—",
              },
              {
                label: "vCard",
                value: job.input.vcard_version || "—",
                mono: true,
              },
              { label: "Creado", value: job.created_at, mono: true },
              {
                label: "Retención",
                value:
                  job.retention_hours != null
                    ? `${job.retention_hours} h`
                    : "—",
              },
              {
                label: "Artefactos",
                value: job.artifacts.join(", ") || "—",
                mono: true,
              },
            ]}
          />
        </Card>
      )}
    </div>
  );
}
