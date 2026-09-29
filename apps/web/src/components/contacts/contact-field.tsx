"use client";

import { IconButton } from "@/components/ui/icon-button";
import { Icon } from "@/components/ui/icon";

type Props = {
  label: string;
  value: string;
  mono?: boolean;
  copyable?: boolean;
};

export function ContactField({ label, value, mono, copyable }: Props) {
  return (
    <div className="zed-row zed-contact-field">
      <div className="zed-contact-field__body">
        <div className="zed-label zed-flush">{label}</div>
        <div className={[mono ? "zed-mono" : "", "zed-wrap"].filter(Boolean).join(" ")}>
          {value}
        </div>
      </div>
      {copyable ? (
        <IconButton
          label={`Copiar ${label}`}
          onClick={() => void navigator.clipboard.writeText(value)}
        >
          <Icon name="copy" aria-hidden={true} />
        </IconButton>
      ) : null}
    </div>
  );
}
