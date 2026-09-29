"use client";

import { Icon } from "@/components/ui/icon";
import type { ReactNode } from "react";
import styles from "@/styles/states.module.css";
import { IconButton } from "./icon-button";

export type FilterChip = {
  id: string;
  label: string;
};

type Props = {
  children: ReactNode;
  chips?: FilterChip[];
  onRemoveChip?: (id: string) => void;
  liveMessage?: string;
};

export function FilterBar({
  children,
  chips = [],
  onRemoveChip,
  liveMessage,
}: Props) {
  return (
    <div className="zed-stack zed-stack--compact">
      <div className="zed-row">{children}</div>
      {chips.length > 0 ? (
        <div className={styles.chipRow} aria-label="Filtros activos">
          {chips.map((chip) => (
            <span key={chip.id} className={styles.chip}>
              {chip.label}
              {onRemoveChip ? (
                <IconButton
                  label={`Quitar filtro ${chip.label}`}
                  size="sm"
                  onClick={() => onRemoveChip(chip.id)}
                >
                  <Icon name="xmark" aria-hidden={true} />
                </IconButton>
              ) : null}
            </span>
          ))}
        </div>
      ) : null}
      <div className={`zed-sr-only ${styles.live}`} aria-live="polite">
        {liveMessage ??
          (chips.length
            ? `${chips.length} filtros activos`
            : "Sin filtros activos")}
      </div>
    </div>
  );
}
