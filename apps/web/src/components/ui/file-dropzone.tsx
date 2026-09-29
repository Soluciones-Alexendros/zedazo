"use client";

import { useState, type ReactNode } from "react";
import { Icon } from "./icon";
import { buttonClassName } from "./button";

type Props = {
  accept?: string;
  disabled?: boolean;
  busy?: boolean;
  icon?: string;
  title: string;
  description?: ReactNode;
  buttonLabel: string;
  busyLabel?: string;
  onFile: (file: File) => void;
};

export function FileDropzone({
  accept = ".vcf,text/vcard",
  disabled = false,
  busy = false,
  icon = "file-arrow-up",
  title,
  description,
  buttonLabel,
  busyLabel = "Cargando…",
  onFile,
}: Props) {
  const [dragActive, setDragActive] = useState(false);
  const blocked = disabled || busy;

  function takeFile(file: File | undefined) {
    if (file && !blocked) onFile(file);
  }

  return (
    <div
      className="zed-dropzone"
      data-active={dragActive}
      onDragOver={(e) => {
        e.preventDefault();
        if (!blocked) setDragActive(true);
      }}
      onDragLeave={() => setDragActive(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragActive(false);
        takeFile(e.dataTransfer.files?.[0]);
      }}
    >
      <Icon name={icon} className="zed-dropzone__icon" aria-hidden={true} />
      <p className="zed-dropzone__title">{title}</p>
      {description ? <p className="zed-muted zed-flush">{description}</p> : null}
      <label className={buttonClassName({ variant: "secondary" })}>
        {busy ? busyLabel : buttonLabel}
        <input
          className="zed-sr-only"
          type="file"
          accept={accept}
          disabled={blocked}
          onChange={(e) => {
            takeFile(e.target.files?.[0]);
            e.target.value = "";
          }}
        />
      </label>
    </div>
  );
}
