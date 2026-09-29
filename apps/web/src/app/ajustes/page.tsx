"use client";

import { useEffect, useState } from "react";
import { API_BASE } from "@/lib/api";
import { logoutSession, readHealth, wipeAllData } from "@/lib/data-adapter";
import { useRouter } from "next/navigation";
import { PageHeader } from "@/components/shell/page-header";
import { Card } from "@/components/ui/card";
import { Callout } from "@/components/ui/callout";
import { MetadataList } from "@/components/ui/metadata-list";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/lib/hooks/use-theme";
import { useApiHealth } from "@/lib/hooks/use-api-health";
import formStyles from "@/styles/forms.module.css";

export default function AjustesPage() {
  const router = useRouter();
  const { preference, setPreference } = useTheme();
  const { state, health, isLocalProcessing, refresh, apiBase } = useApiHealth();
  const [raw, setRaw] = useState("");
  const [wipeBusy, setWipeBusy] = useState(false);
  const [wipeMsg, setWipeMsg] = useState<string | null>(null);
  const [logoutBusy, setLogoutBusy] = useState(false);
  const [httpsOk, setHttpsOk] = useState(true);

  useEffect(() => {
    setHttpsOk(
      window.location.protocol === "https:" ||
        window.location.hostname === "127.0.0.1" ||
        window.location.hostname === "localhost",
    );
  }, []);

  useEffect(() => {
    void readHealth()
      .then((h) => setRaw(JSON.stringify(h, null, 2)))
      .catch((e) => setRaw(String(e)));
  }, []);

  return (
    <div className="zed-stack">
      <PageHeader
        title="Ajustes"
        description="Apariencia, privacidad y estado de la instancia self-hosted."
      />

      <Card variant="document" className={formStyles.form}>
        <h2 className="zed-title-section">Apariencia</h2>
        <div className={formStyles.field}>
          <label className="zed-label" htmlFor="ajustes-tema">
            Tema
          </label>
          <select
            id="ajustes-tema"
            className="zed-input"
            value={preference}
            onChange={(e) =>
              setPreference(e.target.value as "system" | "light" | "dark")
            }
          >
            <option value="system">Sistema</option>
            <option value="light">Claro</option>
            <option value="dark">Oscuro</option>
          </select>
        </div>
        <p className="zed-muted zed-flush">
          La tipografía base ya es más amplia en tablas y fichas. El tamaño
          global sigue el zoom del sistema o del navegador (sin toggle de
          densidad).
        </p>
      </Card>

      <Card variant="document">
        <h2 className="zed-title-section">Privacidad y datos</h2>
        <Callout variant="privacy" icon="hard-drive">
          {isLocalProcessing
            ? "El frontend apunta a una API en loopback (procesamiento local)."
            : "El frontend no afirma procesamiento local porque la API no está en loopback."}
        </Callout>
        <MetadataList
          items={[
            {
              label: "Modo almacenamiento",
              value: health?.storage_mode || "desconocido",
              mono: true,
            },
            {
              label: "Retención",
              value: "Configurable por ejecución (horas)",
            },
          ]}
        />
        <p className="zed-muted">
          Borrar todos los datos locales limpia{" "}
          <code className="zed-mono">jobs/</code>,{" "}
          <code className="zed-mono">uploads/</code> y{" "}
          <code className="zed-mono">tmp/</code> bajo{" "}
          <code className="zed-mono">ZEDAZO_DATA_DIR</code>. No hay telemetría
          remota por defecto.
        </p>
        <Button
          variant="danger"
          loading={wipeBusy}
          onClick={async () => {
            const ok = window.confirm(
              "¿Borrar todos los datos locales de esta instancia? Esta acción no se puede deshacer.",
            );
            if (!ok) return;
            setWipeBusy(true);
            setWipeMsg(null);
            try {
              await wipeAllData();
              setWipeMsg("Datos locales borrados.");
              void refresh();
            } catch (e) {
              setWipeMsg(String(e));
            } finally {
              setWipeBusy(false);
            }
          }}
        >
          Borrar todos los datos locales
        </Button>
        {wipeMsg ? <p className="zed-muted">{wipeMsg}</p> : null}
      </Card>

      <Card variant="document" className="zed-stack">
        <h2 className="zed-title-section">Conectividad y acceso</h2>
        <p>
          Estado:{" "}
          <strong>
            {state === "connected"
              ? "API conectada"
              : state === "local"
                ? "Adaptador local"
                : state === "disconnected"
                  ? "Sin conexión"
                  : "Comprobando…"}
          </strong>
        </p>
        {!httpsOk ? (
          <Callout variant="warning" title="Sin HTTPS">
            En exposición remota usa Caddy o un túnel TLS. El token viaja en
            cookie; sin HTTPS el riesgo aumenta.
          </Callout>
        ) : null}
        <MetadataList
          items={[
            {
              label: "URL API",
              value: apiBase || API_BASE || "(same-origin)",
              mono: true,
              copyable: true,
            },
            {
              label: "Auth",
              value:
                "Token/cookie si ZEDAZO_AUTH_MODE=token (ADR-0016)",
            },
          ]}
        />
        <div className="zed-row">
          <Button variant="secondary" onClick={() => void refresh()}>
            Comprobar salud
          </Button>
          <Button
            variant="secondary"
            loading={logoutBusy}
            onClick={async () => {
              setLogoutBusy(true);
              try {
                await logoutSession();
                router.replace("/acceso");
              } catch (e) {
                setWipeMsg(String(e));
              } finally {
                setLogoutBusy(false);
              }
            }}
          >
            Cerrar sesión
          </Button>
        </div>
        <pre className="zed-mono zed-pre">
          {raw || "…"}
        </pre>
      </Card>

      <Card variant="document">
        <h2 className="zed-title-section">Información</h2>
        <MetadataList
          items={[
            { label: "Frontend", value: "0.5.0-draft", mono: true },
            {
              label: "API",
              value: health?.api_version || "—",
              mono: true,
            },
            {
              label: "Core",
              value: health?.core_version || "—",
              mono: true,
            },
          ]}
        />
      </Card>
    </div>
  );
}
