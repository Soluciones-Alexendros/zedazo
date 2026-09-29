"use client";

import { Icon } from "@/components/ui/icon";
import Link from "next/link";
import { useEffect, useState } from "react";
import { PageHeader } from "@/components/shell/page-header";
import { Card } from "@/components/ui/card";
import { Callout } from "@/components/ui/callout";
import { EmptyState } from "@/components/ui/empty-state";
import { ErrorState } from "@/components/ui/error-state";
import { LoadingState } from "@/components/ui/loading-state";
import { JobStatus } from "@/components/jobs/job-status";
import { buttonClassName } from "@/components/ui/button";
import type { JobManifest } from "@/lib/api";
import { listJobs } from "@/lib/data-adapter";
import { useApiHealth } from "@/lib/hooks/use-api-health";

export default function HomePage() {
  const [jobs, setJobs] = useState<JobManifest[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { isLocalProcessing, state } = useApiHealth();

  useEffect(() => {
    void listJobs()
      .then((d) => {
        setJobs(d.items.slice(0, 3));
        setError(null);
      })
      .catch((e) => {
        setJobs([]);
        setError(String(e));
      })
      .finally(() => setLoaded(true));
  }, []);

  return (
    <div className="zed-stack zed-animate-fade">
      <PageHeader
        eyebrow="Procesamiento VCF local"
        title="Ordena tus contactos. Conserva las decisiones."
        description="Zedazo normaliza, clasifica y revisa duplicados sin convertir tu agenda en una caja negra."
        actions={
          <>
            <Link className={buttonClassName({ variant: "primary" })} href="/procesar">
              Procesar un archivo VCF
            </Link>
            <Link className={buttonClassName({ variant: "secondary" })} href="/ejecuciones">
              Ver ejecuciones
            </Link>
          </>
        }
      />

      <Callout
        variant="privacy"
        title="Privacidad de esta instancia"
        icon="hard-drive"
      >
        {state === "local"
          ? "Adaptador local activo. Los listados son fixtures sintéticos. Procesar un VCF real exige la API."
          : state === "connected" && isLocalProcessing
            ? "API en loopback: el procesamiento se ejecuta en esta máquina."
            : state === "connected"
              ? "API conectada. El frontend no afirma modo local porque la base no es loopback."
              : "No hay conexión con la API. Comprueba que el backend local esté en marcha."}
      </Callout>

      <section>
        <h2 className="zed-title-section">Flujo</h2>
        <div className="zed-grid-metrics">
          {[
            {
              icon: "file-arrow-up",
              title: "Importar",
              text: "Sube un VCF de Proton, Google o Apple.",
            },
            {
              icon: "magnifying-glass",
              title: "Revisar",
              text: "Comprende conservados, revisiones y descartes.",
            },
            {
              icon: "box-archive",
              title: "Exportar",
              text: "Descarga artefactos verificables de la ejecución.",
            },
          ].map((step) => (
            <Card key={step.title} variant="document">
              <Icon name={step.icon} aria-hidden={true} />
              <h3 className="zed-card-title">{step.title}</h3>
              <p className="zed-muted zed-flush">{step.text}</p>
            </Card>
          ))}
        </div>
      </section>

      <section>
        <h2 className="zed-title-section">Actividad reciente</h2>
        {!loaded ? (
          <LoadingState label="Cargando ejecuciones…" />
        ) : error ? (
          <ErrorState message={error} />
        ) : jobs.length === 0 ? (
          <Card variant="document">
            <EmptyState
              compact
              title="Aún no hay ejecuciones"
              description="Cuando proceses un VCF, las tres ejecuciones más recientes aparecerán aquí."
              action={
                <Link className={buttonClassName({ variant: "primary" })} href="/procesar">
                  Procesar un archivo VCF
                </Link>
              }
            />
          </Card>
        ) : (
          <div className="zed-stack">
            {jobs.map((j) => (
              <Card key={j.job_id} variant="action">
                <div className="zed-row zed-row--spread">
                  <div>
                    <strong>{j.display_name || j.input.original_name}</strong>
                    <p className="zed-mono zed-muted zed-card-kicker">{j.created_at}</p>
                  </div>
                  <div className="zed-row">
                    <JobStatus status={j.status} />
                    <Link
                      className={buttonClassName({ variant: "tertiary" })}
                      href={`/ejecuciones/${j.job_id}`}
                    >
                      Abrir
                    </Link>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        )}
      </section>

      <section>
        <h2 className="zed-title-section">Garantías</h2>
        <div className="zed-grid-metrics">
          <Card variant="outlined">
            <h3>Local</h3>
            <p className="zed-muted zed-flush">
              Self-hosted: tus datos permanecen bajo el control de la instancia.
            </p>
          </Card>
          <Card variant="outlined">
            <h3>Trazable</h3>
            <p className="zed-muted zed-flush">
              Cada cambio tiene una razón audible en auditoría y reglas.
            </p>
          </Card>
          <Card variant="outlined">
            <h3>Exportable</h3>
            <p className="zed-muted zed-flush">
              De VCF disperso a agenda verificable en formatos abiertos.
            </p>
          </Card>
        </div>
      </section>

      <p className="zed-muted">
        La automatización y <code className="zed-mono">zedazo completions</code>{" "}
        siguen en la CLI.{" "}
        <Link href="/documentacion">Ver documentación</Link>.
      </p>
    </div>
  );
}
