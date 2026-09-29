"use client";

import { useState } from "react";
import Link from "next/link";
import { createAudit, uploadVcf } from "@/lib/data-adapter";
import { PageHeader } from "@/components/shell/page-header";
import { Card } from "@/components/ui/card";
import { Callout } from "@/components/ui/callout";
import { ErrorState } from "@/components/ui/error-state";
import { FileDropzone } from "@/components/ui/file-dropzone";
import { buttonClassName } from "@/components/ui/button";

export default function AuditarPage() {
  const [jobId, setJobId] = useState<string | null>(null);
  const [fileMeta, setFileMeta] = useState<{
    name: string;
    size: number;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function onFile(file: File) {
    setBusy(true);
    setError(null);
    setFileMeta({ name: file.name, size: file.size });
    try {
      const u = await uploadVcf(file);
      const j = await createAudit(u.upload_id);
      setJobId(j.job_id);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="zed-stack">
      <PageHeader
        title="Auditar"
        description="Inspecciona un VCF sin generar una agenda modificada."
      />

      <Callout variant="info" title="Análisis no destructivo" icon="magnifying-glass">
        Auditar difiere de Procesar: no produce VCF de salida ni aplica deduplicación
        destructiva. Sirve para revisar integridad, avisos y riesgos.
      </Callout>

      <Card variant="document">
        <FileDropzone
          icon="file-magnifying-glass"
          title="Selecciona un VCF para analizar"
          buttonLabel="Analizar archivo"
          busyLabel="Analizando…"
          busy={busy}
          onFile={(f) => void onFile(f)}
        />

        {fileMeta ? (
          <p className="zed-muted">
            Archivo: <span className="zed-mono">{fileMeta.name}</span> ·{" "}
            {(fileMeta.size / 1024).toFixed(1)} KiB
          </p>
        ) : null}

        {error ? <ErrorState message={error} /> : null}

        {jobId ? (
          <Callout variant="success" title="Análisis creado" icon="circle-check">
            <p className="zed-flush">
              La inspección está disponible como ejecución de auditoría.
            </p>
            <Link
              className={buttonClassName({ variant: "secondary" })}
              href={`/ejecuciones/${jobId}`}
            >
              Ver informe
            </Link>
          </Callout>
        ) : null}
      </Card>
    </div>
  );
}
