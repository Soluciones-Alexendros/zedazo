import { AuditEvent } from "./audit-event";
import { EmptyState } from "@/components/ui/empty-state";
import tableStyles from "@/styles/tables.module.css";

type Props = {
  items: { cols: string[] }[];
  technical?: boolean;
};

export function AuditTimeline({ items, technical = false }: Props) {
  if (items.length === 0) {
    return (
      <EmptyState
        compact
        title="Sin auditoría"
        description="No hay filas de auditoría para esta ejecución."
      />
    );
  }

  if (technical) {
    return (
      <div className={tableStyles.tableWrap}>
        <table className={tableStyles.table}>
          <caption>Auditoría técnica</caption>
          <tbody>
            {items.slice(0, 200).map((row, i) => (
              <AuditEvent key={i} cols={row.cols} />
            ))}
          </tbody>
        </table>
      </div>
    );
  }

  return (
    <div className="zed-stack">
      {items.slice(0, 100).map((row, i) => (
        <AuditEvent
          key={i}
          action={row.cols[0] || "Acción"}
          rule={row.cols[1]}
          reason={row.cols.slice(2).filter(Boolean).join(" · ")}
        />
      ))}
    </div>
  );
}
