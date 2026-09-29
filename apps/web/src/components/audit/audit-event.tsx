import { AuditReason } from "./audit-reason";

type Props = {
  action?: string;
  rule?: string;
  reason?: string;
  cols?: string[];
};

export function AuditEvent({ action, rule, reason, cols }: Props) {
  if (cols && cols.length > 0) {
    return (
      <tr>
        {cols.map((c, i) => (
          <td key={i} className="zed-mono zed-wrap">
            {c}
          </td>
        ))}
      </tr>
    );
  }
  return (
    <article className="zed-audit-event">
      <div className="zed-audit-event__action">{action || "Evento"}</div>
      {rule ? <p className="zed-mono zed-flush">{rule}</p> : null}
      {reason ? <AuditReason text={reason} /> : null}
    </article>
  );
}
