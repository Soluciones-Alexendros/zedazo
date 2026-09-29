import styles from "@/styles/shell.module.css";
import type { ConnectionState } from "@/lib/hooks/use-api-health";
import { ZEDAZO_WORDMARK } from "@/components/brand/zedazo-wordmark";

type Props = {
  connection: ConnectionState;
  isLocalProcessing: boolean;
  storageMode?: string | null;
  apiVersion?: string | null;
  coreVersion?: string | null;
  retentionHint?: string | null;
  appVersion?: string;
};

export function Statusbar({
  connection,
  isLocalProcessing,
  storageMode,
  apiVersion,
  coreVersion,
  retentionHint,
  appVersion = "0.5.0-draft",
}: Props) {
  const conn =
    connection === "connected"
      ? "API conectada"
      : connection === "local"
        ? "Adaptador local"
        : connection === "disconnected"
          ? "Sin conexión"
          : "Comprobando…";

  const parts = [
    conn,
    isLocalProcessing ? "Procesamiento local" : null,
    storageMode ? `Almacenamiento: ${storageMode}` : null,
    retentionHint,
    `${ZEDAZO_WORDMARK} ${appVersion}`,
    apiVersion ? `API ${apiVersion}` : null,
    coreVersion ? `core ${coreVersion}` : null,
  ].filter(Boolean);

  return (
    <footer className={styles.statusbar} role="contentinfo">
      <span
        className={styles.statusDot}
        data-ok={connection === "connected"}
        aria-hidden
      />
      <span>{parts.join(" · ")}</span>
    </footer>
  );
}
