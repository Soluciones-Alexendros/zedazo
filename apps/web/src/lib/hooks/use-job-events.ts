"use client";

import { useEffect, useRef, useState } from "react";
import {
  eventsFetchInit,
  eventsUrl,
  hasDesktopBearer,
  isTerminalStatus,
  type JobManifest,
} from "@/lib/api";
import { getJob, probeAdapter } from "@/lib/data-adapter";

type SseHandlers = {
  onPhase: (data: string, id?: string) => void;
  onMetric: (data: string, id?: string) => void;
  onMessage: (data: string, id?: string) => void;
  onError: () => void;
};

/** Parseo mínimo de SSE sobre ReadableStream (Bearer; EventSource no envía Authorization). */
async function consumeFetchSse(
  url: string,
  signal: AbortSignal,
  handlers: SseHandlers,
): Promise<void> {
  const res = await fetch(url, { ...eventsFetchInit(), signal });
  if (!res.ok || !res.body) {
    handlers.onError();
    return;
  }
  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  let eventName = "message";
  let dataLines: string[] = [];
  let lastId: string | undefined;

  const dispatch = () => {
    if (dataLines.length === 0) {
      eventName = "message";
      return;
    }
    const data = dataLines.join("\n");
    dataLines = [];
    const name = eventName;
    eventName = "message";
    if (name === "phase") handlers.onPhase(data, lastId);
    else if (name === "metric") handlers.onMetric(data, lastId);
    else handlers.onMessage(data, lastId);
  };

  while (!signal.aborted) {
    const { done, value } = await reader.read();
    if (done) break;
    buffer += decoder.decode(value, { stream: true });
    const parts = buffer.split(/\r?\n/);
    buffer = parts.pop() ?? "";
    for (const line of parts) {
      if (line === "") {
        dispatch();
        continue;
      }
      if (line.startsWith(":")) continue;
      if (line.startsWith("event:")) {
        eventName = line.slice(6).trim() || "message";
      } else if (line.startsWith("data:")) {
        dataLines.push(line.slice(5).trimStart());
      } else if (line.startsWith("id:")) {
        lastId = line.slice(3).trim();
      }
    }
  }
}

export function useJobEvents(jobId: string | null) {
  const [job, setJob] = useState<JobManifest | null>(null);
  const [phases, setPhases] = useState<string[]>([]);
  const [metrics, setMetrics] = useState<string[]>([]);
  const [messages, setMessages] = useState<string[]>([]);
  const [reconnecting, setReconnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const lastEventId = useRef<string | null>(null);

  useEffect(() => {
    if (!jobId) return;
    let cancelled = false;
    let es: EventSource | null = null;
    let poll: number | undefined;
    let reconnectTimer: number | undefined;
    let abort: AbortController | null = null;

    const handlers: SseHandlers = {
      onPhase: (data, id) => {
        if (id) lastEventId.current = id;
        setPhases((prev) => [...prev, data]);
        setReconnecting(false);
      },
      onMetric: (data, id) => {
        if (id) lastEventId.current = id;
        setMetrics((prev) => [...prev, data]);
      },
      onMessage: (data, id) => {
        if (id) lastEventId.current = id;
        setMessages((prev) => [...prev, data]);
      },
      onError: () => {
        setReconnecting(true);
        if (cancelled) return;
        reconnectTimer = window.setTimeout(() => {
          if (!cancelled) connect();
        }, 800);
      },
    };

    const attachListeners = (source: EventSource) => {
      source.addEventListener("phase", (ev) => {
        const me = ev as MessageEvent;
        if (me.lastEventId) lastEventId.current = me.lastEventId;
        setPhases((prev) => [...prev, String(me.data)]);
        setReconnecting(false);
      });
      source.addEventListener("metric", (ev) => {
        const me = ev as MessageEvent;
        if (me.lastEventId) lastEventId.current = me.lastEventId;
        setMetrics((prev) => [...prev, String(me.data)]);
      });
      source.addEventListener("message", (ev) => {
        const me = ev as MessageEvent;
        if (me.lastEventId) lastEventId.current = me.lastEventId;
        setMessages((prev) => [...prev, String(me.data)]);
      });
      source.onerror = () => {
        setReconnecting(true);
        source.close();
        if (cancelled) return;
        reconnectTimer = window.setTimeout(() => {
          if (!cancelled) connect();
        }, 800);
      };
    };

    const connect = () => {
      es?.close();
      abort?.abort();
      const url = eventsUrl(jobId, lastEventId.current);

      // Escritorio: Bearer vía fetch-stream (EventSource no soporta Authorization).
      if (hasDesktopBearer()) {
        abort = new AbortController();
        void consumeFetchSse(url, abort.signal, handlers).catch(() => {
          if (!cancelled) handlers.onError();
        });
        return;
      }

      es = new EventSource(url, { withCredentials: true });
      attachListeners(es);
    };

    void probeAdapter().then((adapter) => {
      if (cancelled) return;
      if (adapter === "local") {
        void getJob(jobId)
          .then((j) => {
            if (!cancelled) setJob(j.job);
          })
          .catch((e) => {
            if (!cancelled) setError(String(e));
          });
        return;
      }

      connect();
      poll = window.setInterval(async () => {
        try {
          const j = await getJob(jobId);
          if (cancelled) return;
          setJob(j.job);
          if (isTerminalStatus(j.job.status)) {
            if (poll) window.clearInterval(poll);
            if (reconnectTimer) window.clearTimeout(reconnectTimer);
            es?.close();
            abort?.abort();
            setReconnecting(false);
          }
        } catch (e) {
          if (!cancelled) setError(String(e));
        }
      }, 1500);
    });

    return () => {
      cancelled = true;
      es?.close();
      abort?.abort();
      if (poll) window.clearInterval(poll);
      if (reconnectTimer) window.clearTimeout(reconnectTimer);
    };
  }, [jobId]);

  return { job, phases, metrics, messages, reconnecting, error, setJob };
}
