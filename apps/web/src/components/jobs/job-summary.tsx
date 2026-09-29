import type { JobManifest } from "@/lib/api";
import { Card } from "@/components/ui/card";
import { JobStatus } from "./job-status";
import { StatCard } from "@/components/ui/stat-card";

type Props = {
  job: JobManifest;
};

export function JobSummary({ job }: Props) {
  const s = job.summary;
  return (
    <div className="zed-stack">
      <Card variant="document">
        <div className="zed-row zed-row--spread">
          <div>
            <h2 className="zed-title-section zed-card-kicker">
              {job.display_name || job.job_id}
            </h2>
            <p className="zed-mono zed-muted zed-flush">{job.job_id}</p>
          </div>
          <JobStatus status={job.status} />
        </div>
      </Card>
      <div className="zed-grid-metrics">
        <StatCard label="Entrada" value={s?.input_contacts ?? "—"} />
        <StatCard label="Conservados" value={s?.retained ?? "—"} />
        <StatCard label="Revisar" value={s?.needs_review ?? "—"} />
        <StatCard label="Descartados" value={s?.eliminated ?? "—"} />
        <StatCard label="Cuarentena" value={s?.quarantine ?? "—"} />
        <StatCard label="Duplicados" value={s?.duplicate_groups ?? "—"} />
      </div>
    </div>
  );
}
