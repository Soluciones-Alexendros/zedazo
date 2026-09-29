"use client";

import { useState, type ReactNode } from "react";
import { AppSidebar } from "./app-sidebar";
import { MobileNavigation } from "./mobile-navigation";
import { Topbar } from "./topbar";
import { Statusbar } from "./statusbar";
import { LocalAdapterBanner } from "./local-adapter-banner";
import { useApiHealth } from "@/lib/hooks/use-api-health";
import styles from "@/styles/shell.module.css";

type Props = {
  children: ReactNode;
};

export function AppShell({ children }: Props) {
  const [navOpen, setNavOpen] = useState(false);
  const { state, health, isLocalProcessing } = useApiHealth();

  return (
    <div className={styles.shell}>
      <a href="#contenido-principal" className="zed-skip-link">
        Saltar al contenido principal
      </a>
      <Topbar
        connection={state}
        isLocalProcessing={isLocalProcessing}
        onOpenNav={() => setNavOpen(true)}
      />
      <AppSidebar open={navOpen} onNavigate={() => setNavOpen(false)} />
      <MobileNavigation open={navOpen} onClose={() => setNavOpen(false)} />
      <div className={styles.main}>
        <div id="contenido-principal" className={styles.mainInner} tabIndex={-1}>
          {state === "local" ? <LocalAdapterBanner /> : null}
          {children}
        </div>
      </div>
      <Statusbar
        connection={state}
        isLocalProcessing={isLocalProcessing}
        storageMode={health?.storage_mode}
        apiVersion={health?.api_version}
        coreVersion={health?.core_version}
        retentionHint="Retención: configurable por ejecución"
      />
    </div>
  );
}
