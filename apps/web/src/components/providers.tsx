"use client";

import type { ReactNode } from "react";
import { usePathname } from "next/navigation";
import { ThemeProvider } from "@/lib/hooks/use-theme";
import { AppShell } from "@/components/shell/app-shell";
import { AuthGate } from "@/components/auth-gate";

export function Providers({ children }: { children: ReactNode }) {
  return (
    <ThemeProvider>
      <AuthGate>
        <ShellOrBare>{children}</ShellOrBare>
      </AuthGate>
    </ThemeProvider>
  );
}

function ShellOrBare({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  if (pathname === "/acceso") {
    return <>{children}</>;
  }
  return <AppShell>{children}</AppShell>;
}
