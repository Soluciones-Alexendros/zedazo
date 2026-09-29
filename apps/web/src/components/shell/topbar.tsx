"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { IconButton } from "@/components/ui/icon-button";
import { Icon } from "@/components/ui/icon";
import { buttonClassName } from "@/components/ui/button";
import { useTheme } from "@/lib/hooks/use-theme";
import type { ConnectionState } from "@/lib/hooks/use-api-health";
import styles from "@/styles/shell.module.css";
import { NAV_ITEMS } from "./app-sidebar";

const LABELS: Record<string, string> = {
  "/": "Inicio",
  "/procesar": "Procesar",
  "/ejecuciones": "Ejecuciones",
  "/auditar": "Auditar",
  "/reglas": "Reglas",
  "/documentacion": "Documentación",
  "/ajustes": "Ajustes",
};

type Props = {
  connection: ConnectionState;
  isLocalProcessing: boolean;
  onOpenNav: () => void;
};

export function Topbar({ connection, isLocalProcessing, onOpenNav }: Props) {
  const pathname = usePathname();
  const { preference, setPreference } = useTheme();

  const segments = pathname.split("/").filter(Boolean);
  const crumbLabel =
    LABELS[`/${segments[0] || ""}`] ||
    (segments[0] === "ejecuciones" && segments[1]
      ? "Detalle de ejecución"
      : "Inicio");

  const connectionLabel =
    connection === "connected"
      ? "API conectada"
      : connection === "local"
        ? "Adaptador local"
        : connection === "disconnected"
          ? "Sin conexión"
          : "Comprobando API…";

  return (
    <header className={styles.topbar}>
      <IconButton
        label="Abrir navegación"
        className={styles.menuButton}
        onClick={onOpenNav}
      >
        <Icon name="bars" aria-hidden={true} />
      </IconButton>

      <nav className={styles.breadcrumb} aria-label="Miga de pan">
        <Link href="/">Inicio</Link>
        {pathname !== "/" ? (
          <>
            <span aria-hidden>/</span>
            <span className={styles.breadcrumbCurrent}>{crumbLabel}</span>
          </>
        ) : null}
      </nav>

      <div className={styles.topbarActions}>
        <span className="zed-row zed-row--tight">
          <span
            className={styles.statusDot}
            data-ok={connection === "connected"}
            aria-hidden
          />
          <span className={`zed-muted ${styles.statusLabel}`}>
            {connectionLabel}
            {connection === "connected" && isLocalProcessing
              ? " · Procesamiento local"
              : ""}
          </span>
        </span>

        <label className="zed-sr-only" htmlFor="theme-select">
          Tema
        </label>
        <select
          id="theme-select"
          className="zed-input zed-input--auto"
          value={preference}
          onChange={(e) =>
            setPreference(e.target.value as "system" | "light" | "dark")
          }
        >
          <option value="system">Sistema</option>
          <option value="light">Claro</option>
          <option value="dark">Oscuro</option>
        </select>

        <Link
          href="/documentacion"
          className={buttonClassName({ variant: "tertiary" })}
          aria-label="Ayuda y documentación"
        >
          <Icon name="circle-question" aria-hidden={true} />
          Ayuda
        </Link>
      </div>

      {/* Keep NAV_ITEMS referenced for parity with sidebar labels in mobile a11y docs */}
      <span className="zed-sr-only">
        Secciones: {NAV_ITEMS.map((i) => i.label).join(", ")}
      </span>
    </header>
  );
}
