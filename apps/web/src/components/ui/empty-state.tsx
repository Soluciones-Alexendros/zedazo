import type { ReactNode } from "react";
import styles from "@/styles/states.module.css";

type Props = {
  title: string;
  description?: string;
  action?: ReactNode;
  icon?: ReactNode;
  centered?: boolean;
  compact?: boolean;
};

export function EmptyState({
  title,
  description,
  action,
  icon,
  centered = false,
  compact = false,
}: Props) {
  const className = [
    styles.state,
    centered ? styles.stateCenter : "",
    compact ? styles.stateCompact : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={className}>
      {icon}
      <h2 className={styles.stateTitle}>{title}</h2>
      {description ? <p className={styles.stateBody}>{description}</p> : null}
      {action}
    </div>
  );
}
