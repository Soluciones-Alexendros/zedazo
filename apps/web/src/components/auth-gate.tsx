"use client";

import { useEffect, useState, type ReactNode } from "react";
import { usePathname, useRouter } from "next/navigation";
import { ApiError, getVersion } from "@/lib/api";
import { LoadingState } from "@/components/ui/loading-state";

type Props = {
  children: ReactNode;
};

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
    void getVersion()
      .then(() => {
        if (!cancelled) setReady(true);
      })
      .catch((e) => {
        if (cancelled) return;
        if (e instanceof ApiError && e.status === 401) {
          router.replace("/acceso");
          return;
        }
        setReady(true);
      });
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
