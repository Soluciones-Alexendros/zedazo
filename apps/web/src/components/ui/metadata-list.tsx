"use client";

import { Icon } from "@/components/ui/icon";
import { IconButton } from "./icon-button";

export type MetadataItem = {
  label: string;
  value: string;
  mono?: boolean;
  copyable?: boolean;
};

type Props = {
  items: MetadataItem[];
};

async function copyText(value: string) {
  try {
    await navigator.clipboard.writeText(value);
  } catch {
    /* ignore */
  }
}

export function MetadataList({ items }: Props) {
  return (
    <dl className="zed-metadata">
      {items.map((item) => (
        <div key={item.label} className="zed-metadata__row">
          <dt className="zed-label zed-metadata__term">{item.label}</dt>
          <dd
            className={`${item.mono ? "zed-mono" : ""} zed-metadata__value`.trim()}
            title={item.value}
          >
            <span className="zed-truncate">{item.value}</span>
            {item.copyable ? (
              <IconButton
                label={`Copiar ${item.label}`}
                onClick={() => void copyText(item.value)}
              >
                <Icon name="copy" aria-hidden={true} />
              </IconButton>
            ) : null}
          </dd>
        </div>
      ))}
    </dl>
  );
}
