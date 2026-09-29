type AuditRow = { cols: string[] };

type Props = {
  memberUids: string[];
  auditRows?: AuditRow[];
};

/** Evidencia D1/D2 desde filas de auditoría (FUSIONADO / merged). */
export function DuplicateEvidence({ memberUids, auditRows = [] }: Props) {
  const uidSet = new Set(memberUids);
  const evidence = auditRows
    .map((row) => row.cols)
    .filter((cols) => cols.length >= 7)
    .filter((cols) => {
      const uid = cols[1] || "";
      const action = (cols[4] || "").toUpperCase();
      return uidSet.has(uid) && (action === "FUSIONADO" || action === "MERGED");
    })
    .map((cols) => ({
      uid: cols[1],
      action: cols[4],
      reason: cols[5] || "",
      rule: cols[6] || "",
    }));

  if (evidence.length === 0) {
    return (
      <p className="zed-muted zed-flush">
        Sin filas FUSIONADO en auditoría para este grupo. Descarga audit.tsv si
        necesitas la traza completa.
      </p>
    );
  }

  return (
    <div className="zed-stack zed-stack--compact">
      <p className="zed-label zed-flush">Evidencia de fusión (auditoría)</p>
      <ul className="zed-prose-list zed-flush">
        {evidence.map((e) => (
          <li key={`${e.uid}-${e.rule}-${e.reason}`}>
            <span className="zed-mono">{e.uid}</span>
            {" — "}
            <strong>{e.rule || "regla"}</strong>
            {e.reason ? `: ${e.reason}` : null}
          </li>
        ))}
      </ul>
    </div>
  );
}
