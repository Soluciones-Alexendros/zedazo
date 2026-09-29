"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import type { JobManifest } from "@/lib/api";
import { listJobs } from "@/lib/data-adapter";
import { PageHeader } from "@/components/shell/page-header";
import { JobStatus } from "@/components/jobs/job-status";
import { JobRowActions } from "@/components/jobs/job-row-actions";
import { Button } from "@/components/ui/button";
import { buttonClassName } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { ErrorState } from "@/components/ui/error-state";
import { LoadingState } from "@/components/ui/loading-state";
import { FilterBar } from "@/components/ui/filter-bar";
import tableStyles from "@/styles/tables.module.css";

export default function EjecucionesPage() {
  const [items, setItems] = useState<JobManifest[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [q, setQ] = useState("");
  const [status, setStatus] = useState("");

  async function refresh() {
    setLoading(true);
    try {
      const d = await listJobs();
      setItems(d.items);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  const filtered = useMemo(() => {
    return items.filter((j) => {
      const name = (j.display_name || j.input.original_name || "").toLowerCase();
      const matchQ = !q || name.includes(q.toLowerCase()) || j.job_id.includes(q);
      const matchStatus = !status || j.status === status;
      return matchQ && matchStatus;
    });
  }, [items, q, status]);

  const chips = [
    q ? { id: "q", label: `Búsqueda: ${q}` } : null,
    status ? { id: "status", label: `Estado: ${status}` } : null,
  ].filter(Boolean) as { id: string; label: string }[];

  return (
    <div className="zed-stack">
      <PageHeader
        title="Ejecuciones"
        description="Expedientes de procesamiento. Sin datos personales de contactos en este listado."
        actions={
          <Link className={buttonClassName({ variant: "primary" })} href="/procesar">
            Nueva ejecución
          </Link>
        }
      />

      {error ? (
        <ErrorState
          message={error}
          action={<Button onClick={() => void refresh()}>Reintentar</Button>}
        />
      ) : null}

      <FilterBar
        chips={chips}
        onRemoveChip={(id) => {
          if (id === "q") setQ("");
          if (id === "status") setStatus("");
        }}
      >
        <label className="zed-sr-only" htmlFor="jobs-q">
          Buscar
        </label>
        <input
          id="jobs-q"
          className="zed-input zed-input--filter"
          placeholder="Buscar por nombre o ID"
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        <label className="zed-sr-only" htmlFor="jobs-status">
          Estado
        </label>
        <select
          id="jobs-status"
          className="zed-input zed-input--auto"
          value={status}
          onChange={(e) => setStatus(e.target.value)}
        >
          <option value="">Todos los estados</option>
          <option value="completed">Completado</option>
          <option value="failed">Fallido</option>
          <option value="queued">En cola</option>
          <option value="cancel_requested">Cancelación pedida</option>
          <option value="cancelled">Cancelado</option>
        </select>
      </FilterBar>

      {loading ? <LoadingState label="Cargando ejecuciones…" /> : null}

      {!loading && filtered.length === 0 ? (
        <EmptyState
          title="Sin ejecuciones"
          description="Aún no hay ejecuciones que coincidan con los filtros."
          action={
            <Link className={buttonClassName({ variant: "primary" })} href="/procesar">
              Procesar un archivo VCF
            </Link>
          }
        />
      ) : null}

      {!loading && filtered.length > 0 ? (
        <>
          <div className={`${tableStyles.tableWrap} ${tableStyles.desktopOnly}`}>
            <table className={tableStyles.table}>
              <caption>Listado de ejecuciones</caption>
              <thead>
                <tr>
                  <th scope="col">Nombre</th>
                  <th scope="col">Fecha</th>
                  <th scope="col">Estado</th>
                  <th scope="col">Fuente</th>
                  <th scope="col">Entrada</th>
                  <th scope="col">Conservados</th>
                  <th scope="col">Revisar</th>
                  <th scope="col">Descartados</th>
                  <th scope="col">Duplicados</th>
                  <th scope="col">Retención</th>
                  <th scope="col">Acciones</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((j) => (
                  <tr key={j.job_id}>
                    <td>{j.display_name || j.input.original_name}</td>
                    <td className="zed-mono">{j.created_at}</td>
                    <td>
                      <JobStatus status={j.status} />
                    </td>
                    <td>{j.input.source_detected || "—"}</td>
                    <td className={tableStyles.num}>
                      {j.summary?.input_contacts ?? "—"}
                    </td>
                    <td className={tableStyles.num}>{j.summary?.retained ?? "—"}</td>
                    <td className={tableStyles.num}>
                      {j.summary?.needs_review ?? "—"}
                    </td>
                    <td className={tableStyles.num}>
                      {j.summary?.eliminated ?? "—"}
                    </td>
                    <td className={tableStyles.num}>
                      {j.summary?.duplicate_groups ?? "—"}
                    </td>
                    <td className="zed-mono">
                      {j.retention_hours != null ? `${j.retention_hours} h` : "—"}
                    </td>
                    <td>
                      <JobRowActions job={j} onChanged={refresh} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className={tableStyles.cardList}>
            {filtered.map((j) => (
              <article key={j.job_id} className={tableStyles.dossierCard}>
                <div className="zed-row zed-row--spread">
                  <strong>{j.display_name || j.input.original_name}</strong>
                  <JobStatus status={j.status} />
                </div>
                <div className={tableStyles.dossierMeta}>
                  <span className="zed-mono">{j.created_at}</span>
                  <span>Entrada: {j.summary?.input_contacts ?? "—"}</span>
                  <span>Conservados: {j.summary?.retained ?? "—"}</span>
                </div>
                <JobRowActions job={j} onChanged={refresh} primary />
              </article>
            ))}
          </div>
        </>
      ) : null}
    </div>
  );
}
