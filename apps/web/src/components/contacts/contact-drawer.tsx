"use client";

import { useEffect, useRef } from "react";
import type { ContactView } from "@/lib/api";
import { ContactResultBadge } from "./contact-result-badge";
import { ContactField } from "./contact-field";
import { FieldProvenance } from "./field-provenance";
import { IconButton } from "@/components/ui/icon-button";
import { Icon } from "@/components/ui/icon";

type Props = {
  contact: ContactView | null;
  open: boolean;
  onClose: () => void;
};

export function ContactDrawer({ contact, open, onClose }: Props) {
  const closeRef = useRef<HTMLButtonElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (open) {
      previousFocus.current = document.activeElement as HTMLElement | null;
      closeRef.current?.focus();
      const onKey = (e: KeyboardEvent) => {
        if (e.key === "Escape") onClose();
      };
      window.addEventListener("keydown", onKey);
      return () => window.removeEventListener("keydown", onKey);
    }
    previousFocus.current?.focus();
    return undefined;
  }, [open, onClose]);

  if (!open || !contact) return null;

  return (
    <div
      role="presentation"
      className="zed-scrim zed-scrim--modal"
      onClick={onClose}
    >
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-label="Detalle de contacto"
        className="zed-card zed-card--document zed-animate-slide zed-drawer"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="zed-stack">
          <div className="zed-row zed-row--spread">
            <h2 className="zed-contact-name zed-flush">
              {contact.fn_value || "(sin nombre)"}
            </h2>
            <IconButton ref={closeRef} label="Cerrar detalle" onClick={onClose}>
              <Icon name="xmark" aria-hidden={true} />
            </IconButton>
          </div>

          <section>
            <h3>Resultado final</h3>
            <ContactResultBadge result={contact.result} />
          </section>

          <section>
            <h3>Cambios aplicados</h3>
            <ContactField label="UID" value={contact.uid} mono copyable />
            <ContactField
              label="Regla"
              value={contact.screening_rule || "—"}
              mono
            />
            <ContactField
              label="Categorías"
              value={contact.categories.join(", ") || "—"}
            />
          </section>

          <section>
            <h3>Evidencias</h3>
            <ContactField
              label="Emails"
              value={contact.emails.join(", ") || "—"}
              mono
            />
            <ContactField
              label="Teléfonos"
              value={contact.tels.join(", ") || "—"}
              mono
            />
            {contact.merged_uids.length > 0 ? (
              <ContactField
                label="UIDs fusionados"
                value={contact.merged_uids.join(", ")}
                mono
                copyable
              />
            ) : (
              <p className="zed-muted">
                Sin fusiones asociadas a este contacto.
              </p>
            )}
          </section>

          <section>
            <h3>Proveniencia</h3>
            <FieldProvenance rule={contact.screening_rule} />
          </section>
        </div>
      </div>
    </div>
  );
}
