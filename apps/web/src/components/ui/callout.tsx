"use client";

import { Icon } from "@/components/ui/icon";
import type { ReactNode } from "react";

export type CalloutVariant =
  | "info"
  | "success"
  | "warning"
  | "danger"
  | "privacy"
  | "verification";

type Props = {
  variant?: CalloutVariant;
  title?: string;
  children: ReactNode;
  icon?: string;
};

export function Callout({
  variant = "info",
  title,
  children,
  icon = "circle-info",
}: Props) {
  return (
    <div className={`zed-callout zed-callout--${variant}`} role="note">
      <Icon name={icon} aria-hidden={true} />
      <div>
        {title ? <strong className="zed-callout__title">{title}</strong> : null}
        <div>{children}</div>
      </div>
    </div>
  );
}
