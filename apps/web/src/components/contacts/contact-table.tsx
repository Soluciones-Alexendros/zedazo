"use client";

import type { ContactView } from "@/lib/api";
import { ContactResultBadge } from "./contact-result-badge";
import { EmptyState } from "@/components/ui/empty-state";
import tableStyles from "@/styles/tables.module.css";

type Props = {
  contacts: ContactView[];
  onSelect?: (contact: ContactView) => void;
};

export function ContactTable({ contacts, onSelect }: Props) {
  if (contacts.length === 0) {
    return (
      <EmptyState
        compact
        title="Sin contactos"
        description="Ningún contacto coincide con los filtros de esta ejecución."
      />
    );
  }

  return (
    <>
      <div className={`${tableStyles.tableWrap} ${tableStyles.desktopOnly}`}>
        <table className={tableStyles.table}>
          <caption>Resultados de contactos de la ejecución</caption>
          <thead>
            <tr>
              <th scope="col">Nombre</th>
              <th scope="col">Resultado</th>
              <th scope="col">Regla</th>
              <th scope="col">Emails</th>
              <th scope="col">Teléfonos</th>
            </tr>
          </thead>
          <tbody>
            {contacts.map((c) => (
              <tr
                key={c.uid}
                className={onSelect ? tableStyles.interactive : undefined}
                onClick={() => onSelect?.(c)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    onSelect?.(c);
                  }
                }}
                tabIndex={onSelect ? 0 : undefined}
              >
                <td>
                  <span className="zed-contact-name zed-contact-name--table">
                    {c.fn_value || "(sin nombre)"}
                  </span>
                </td>
                <td>
                  <ContactResultBadge result={c.result} />
                </td>
                <td className="zed-mono">{c.screening_rule || "—"}</td>
                <td className="zed-mono">{c.emails.join(", ") || "—"}</td>
                <td className="zed-mono">{c.tels.join(", ") || "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className={tableStyles.cardList}>
        {contacts.map((c) => (
          <button
            key={c.uid}
            type="button"
            className={`${tableStyles.dossierCard} ${tableStyles.dossierButton}`}
            onClick={() => onSelect?.(c)}
          >
            <div className="zed-row zed-row--spread">
              <strong className="zed-contact-name">{c.fn_value || "(sin nombre)"}</strong>
              <ContactResultBadge result={c.result} />
            </div>
            <div className={tableStyles.dossierMeta}>
              <span className="zed-mono">{c.screening_rule || "Sin regla"}</span>
              <span>{c.emails[0] || "Sin email"}</span>
            </div>
          </button>
        ))}
      </div>
    </>
  );
}
