"use client";

import { Icon } from "@/components/ui/icon";
import { JobStatus, jobStatusMeta } from "./job-status";

const PHASE_ORDER = [
  "validating",
  "parsing",
  "screening",
  "normalizing",
  "classifying",
  "deduplicating",
  "verifying",
  "writing_artifacts",
];

type Props = {
  status: string;
  phases?: string[];
  reconnecting?: boolean;
};

export function JobProgress({ status, phases = [], reconnecting }: Props) {
  const currentIdx = PHASE_ORDER.indexOf(status);
  return (
    <div className="zed-stack" aria-live="polite">
      <div className="zed-row">
        <JobStatus status={status} />
        {reconnecting ? (
          <span className="zed-muted">Reconectando eventos…</span>
        ) : null}
      </div>
      <ol className="zed-stack zed-phase-list">
        {PHASE_ORDER.map((phase, index) => {
          const meta = jobStatusMeta(phase);
          const state =
            status === "completed"
              ? "done"
              : status === "failed" && index <= Math.max(currentIdx, 0)
                ? index === currentIdx
                  ? "failed"
                  : "done"
                : index < currentIdx
                  ? "done"
                  : index === currentIdx
                    ? "current"
                    : "todo";
          return (
            <li
              key={phase}
              className={`zed-row zed-phase-list__item--${state}`}
            >
              <Icon
                name={
                  state === "done"
                    ? "circle-check"
                    : state === "failed"
                      ? "circle-xmark"
                      : state === "current"
                        ? "arrows-rotate"
                        : "circle"
                }
                aria-hidden={true}
              />
              <span>{meta.label}</span>
            </li>
          );
        })}
      </ol>
      {phases.length > 0 ? (
        <details>
          <summary>Eventos recibidos ({phases.length})</summary>
          <ul className="zed-mono zed-prose-list">
            {phases.map((p, i) => (
              <li key={`${i}-${p}`}>{p}</li>
            ))}
          </ul>
        </details>
      ) : null}
    </div>
  );
}
