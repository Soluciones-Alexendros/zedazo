"use client";

import { useState } from "react";
import { buttonClassName } from "./button";

type Props = {
  href?: string;
  label: string;
  description?: string;
  meta?: string;
  /** Si se define, descarga vía fetch+Blob (Bearer) en lugar de navegación. */
  onDownload?: () => Promise<void>;
};

export function ArtifactDownload({
  href,
  label,
  description,
  meta,
  onDownload,
}: Props) {
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  if (onDownload) {
    return (
      <div className="zed-row zed-artifact">
        <div>
          <div className="zed-artifact__label">{label}</div>
          {description ? (
            <p className="zed-muted zed-artifact__meta">{description}</p>
          ) : null}
          {meta ? (
            <p className="zed-mono zed-muted zed-artifact__meta">{meta}</p>
          ) : null}
          {err ? <p className="zed-muted">{err}</p> : null}
        </div>
        <button
          type="button"
          className={buttonClassName({ variant: "secondary" })}
          disabled={busy}
          onClick={async () => {
            setBusy(true);
            setErr(null);
            try {
              await onDownload();
            } catch (e) {
              setErr(String(e));
            } finally {
              setBusy(false);
            }
          }}
        >
          Descargar
        </button>
      </div>
    );
  }

  return (
    <div className="zed-row zed-artifact">
      <div>
        <div className="zed-artifact__label">{label}</div>
        {description ? (
          <p className="zed-muted zed-artifact__meta">{description}</p>
        ) : null}
        {meta ? (
          <p className="zed-mono zed-muted zed-artifact__meta">{meta}</p>
        ) : null}
      </div>
      <a className={buttonClassName({ variant: "secondary" })} href={href}>
        Descargar
      </a>
    </div>
  );
}
