"use client";

import Link from "next/link";
import { isCancellableStatus, type JobManifest } from "@/lib/api";
import { cancelJob, deleteJob } from "@/lib/data-adapter";
import { Button, buttonClassName } from "@/components/ui/button";

type Props = {
  job: JobManifest;
  onChanged: () => Promise<void>;
  primary?: boolean;
};

export function JobRowActions({ job, onChanged, primary = false }: Props) {
  return (
    <div className="zed-row">
      <Link
        className={buttonClassName({ variant: primary ? "primary" : "tertiary" })}
        href={`/ejecuciones/${job.job_id}`}
      >
        Abrir
      </Link>
      {isCancellableStatus(job.status) ? (
        <Button
          variant="secondary"
          onClick={async () => {
            await cancelJob(job.job_id);
            await onChanged();
          }}
        >
          Cancelar
        </Button>
      ) : null}
      <Button
        variant="danger"
        onClick={async () => {
          await deleteJob(job.job_id);
          await onChanged();
        }}
      >
        Eliminar
      </Button>
    </div>
  );
}
