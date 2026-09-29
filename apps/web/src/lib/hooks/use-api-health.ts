"use client";

import { useCallback, useEffect, useState } from "react";
import { API_BASE } from "@/lib/api";
import { probeAdapter, readHealth, resetAdapterProbe } from "@/lib/data-adapter";

export type ConnectionState = "checking" | "connected" | "disconnected" | "local";

function isLoopbackHost(hostname: string) {
  return (
    hostname === "127.0.0.1" ||
    hostname === "localhost" ||
    hostname === "::1"
  );
}

function isLocalProcessing(base: string) {
  if (!base) {
    if (typeof window === "undefined") return false;
    return isLoopbackHost(window.location.hostname);
  }
  try {
    const url = new URL(base);
    return isLoopbackHost(url.hostname);
  } catch {
    return false;
  }
}

export function useApiHealth(pollMs = 30000) {
  const [state, setState] = useState<ConnectionState>("checking");
  const [health, setHealth] = useState<{
    status: string;
    api_version: string;
    core_version: string;
    storage_mode: string;
  } | null>(null);

  const refresh = useCallback(async () => {
    resetAdapterProbe();
    try {
      const adapter = await probeAdapter();
      if (adapter === "local") {
        setHealth(null);
        setState("local");
        return;
      }
      const h = await readHealth();
      setHealth(h);
      setState("connected");
    } catch {
      setHealth(null);
      setState("disconnected");
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => void refresh(), pollMs);
    return () => window.clearInterval(id);
  }, [pollMs, refresh]);

  return {
    state,
    health,
    refresh,
    isLocalProcessing: isLocalProcessing(API_BASE),
    apiBase: API_BASE || "(same-origin)",
  };
}
