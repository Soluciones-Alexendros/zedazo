"use client";

import { useEffect, useState, type ReactNode } from "react";
import { usePathname, useRouter } from "next/navigation";
import { ApiError, getVersion, sessionBearer } from "@/lib/api";
import { LoadingState } from "@/components/ui/loading-state";

type Props = {
  children: ReactNode;
};

/** Escritorio Tauri: el init script puede llegar un tick después del primer paint
 * (sobre todo tras navegación WebDriver). Sin esta espera, getVersion → 401 y la
 * GUI queda en blanco / redirige a /acceso. */
async function waitDesktopBearer(ms: number): Promise<boolean> {
  const deadline = Date.now() + ms;
  while (Date.now() < deadline) {
    if (sessionBearer()) return true;
    await new Promise((r) => setTimeout(r, 50));
  }
  return Boolean(sessionBearer());
}

/**
 * Si la API está en modo token y no hay cookie válida, redirige a /acceso.
 * Si la API no responde, deja pasar (las pantallas muestran su propio error).
 */
export function AuthGate({ children }: Props) {
  const pathname = usePathname();
  const router = useRouter();
  const [ready, setReady] = useState(pathname === "/acceso");

  useEffect(() => {
    if (pathname === "/acceso") {
      setReady(true);
      return;
    }
    let cancelled = false;
    setReady(false);

    void (async () => {
      await waitDesktopBearer(3000);
      if (cancelled) return;
      try {
        await getVersion();
        if (!cancelled) setReady(true);
      } catch (e) {
        if (cancelled) return;
        if (e instanceof ApiError && e.status === 401) {
          // Reintento si el Bearer acaba de aparecer tras el 401.
          if (sessionBearer()) {
            try {
              await getVersion();
              if (!cancelled) setReady(true);
              return;
            } catch {
              /* seguir a /acceso */
            }
          }
          router.replace("/acceso");
          return;
        }
        setReady(true);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [pathname, router]);

  if (pathname === "/acceso") {
    return <>{children}</>;
  }
  if (!ready) {
    return (
      <div className="zed-stack zed-gate">
        <LoadingState label="Comprobando acceso…" />
      </div>
    );
  }
  return <>{children}</>;
}
