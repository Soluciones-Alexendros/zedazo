"use client";

import { PageHeader } from "@/components/shell/page-header";
import { JobProgress } from "@/components/jobs/job-progress";
import { JobStatus } from "@/components/jobs/job-status";
import { ArtifactDownload } from "@/components/ui/artifact-download";
import { Badge } from "@/components/ui/badge";
import { Button, buttonClassName } from "@/components/ui/button";
import { Callout } from "@/components/ui/callout";
import { Card } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
import { ErrorState } from "@/components/ui/error-state";
import { FileDropzone } from "@/components/ui/file-dropzone";
import { FilterBar } from "@/components/ui/filter-bar";
import { Icon } from "@/components/ui/icon";
import { IconButton } from "@/components/ui/icon-button";
import { LoadingState } from "@/components/ui/loading-state";
import { MetadataList } from "@/components/ui/metadata-list";
import { ProgressStepper } from "@/components/ui/progress-stepper";
import { SectionHeading } from "@/components/ui/section-heading";
import { StatCard } from "@/components/ui/stat-card";
import { VisuallyHidden } from "@/components/ui/visually-hidden";
import formStyles from "@/styles/forms.module.css";

const TOC = [
  { href: "#ds-tokens", label: "Tokens" },
  { href: "#ds-boton", label: "Botón" },
  { href: "#ds-badge", label: "Badge" },
  { href: "#ds-card", label: "Card" },
  { href: "#ds-callout", label: "Callout" },
  { href: "#ds-formulario", label: "Formulario" },
  { href: "#ds-icono", label: "Icono" },
  { href: "#ds-filtro", label: "Filtro" },
  { href: "#ds-datos", label: "Datos" },
  { href: "#ds-stepper", label: "Stepper" },
  { href: "#ds-ejecucion", label: "Ejecución" },
  { href: "#ds-estados", label: "Estados" },
] as const;

const TOKEN_SWATCHES = [
  "--zed-bg-canvas",
  "--zed-bg-raised",
  "--zed-fg-default",
  "--zed-fg-muted",
  "--zed-accent",
  "--zed-success",
  "--zed-warning",
  "--zed-danger",
  "--zed-info",
  "--zed-focus-ring",
] as const;

/**
 * Catálogo in-app de átomos y patrones de producto (fase 4).
 * Sin Storybook. Traza: docs/gui/design-system-plan.md · epic #59.
 */
export default function DesignSystemCatalogPage() {
  return (
    <div className="zed-stack zed-catalog-grid">
      <PageHeader
        title="Sistema de diseño"
        eyebrow="catálogo"
        description="Átomos y patrones usados en la GUI, alineados a tokens --zed-*. Wordmark: zedazo. APCA es informativo; el gate de CI es WCAG 2.2 AA."
      />

      <Card variant="document">
        <nav aria-label="Secciones del catálogo">
          <ul className="zed-catalog-toc">
            {TOC.map((item) => (
              <li key={item.href}>
                <a href={item.href}>{item.label}</a>
              </li>
            ))}
          </ul>
        </nav>
      </Card>

      <Card variant="document" id="ds-tokens">
        <h2 className="zed-title-section">Tokens</h2>
        <p className="zed-muted">
          Semánticos OKLCH. El contraste se verifica en{" "}
          <code className="zed-mono">pnpm tokens:contrast</code>.
        </p>
        <div className="zed-catalog-token-row">
          {TOKEN_SWATCHES.map((token) => (
            <span key={token} className="zed-catalog-token">
              <span
                className="zed-catalog-token__chip"
                style={{ background: `var(${token})` }}
                aria-hidden={true}
              />
              <code className="zed-mono">{token}</code>
            </span>
          ))}
        </div>
        <div className="zed-catalog-type">
          <p className="zed-title-section">system-ui · título de sección</p>
          <p>Cuerpo de interfaz. Texto esencial sobre canvas.</p>
          <p className="zed-muted">Texto secundario (muted).</p>
          <p className="zed-mono">ui-monospace / Fira Code · hash o TOML</p>
        </div>
      </Card>

      <Card variant="document" id="ds-boton">
        <h2 className="zed-title-section">Botón</h2>
        <p className="zed-muted">
          Variantes primario / secundario / terciario / peligro; tamaños md / sm;
          estados hover, foco visible, deshabilitado y carga. Diana ≥{" "}
          <code className="zed-mono">--zed-target-min</code>.
        </p>
        <div className="zed-catalog-swatch">
          <Button variant="primary">Primario</Button>
          <Button variant="secondary">Secundario</Button>
          <Button variant="tertiary">Terciario</Button>
          <Button variant="danger">Peligro</Button>
          <Button variant="primary" size="sm">
            Compacto
          </Button>
          <Button variant="primary" disabled>
            Deshabilitado
          </Button>
          <Button variant="primary" loading>
            Cargando
          </Button>
          <IconButton label="Cerrar ejemplo">
            <Icon name="xmark" aria-hidden={true} />
          </IconButton>
        </div>
      </Card>

      <Card variant="document" id="ds-badge">
        <h2 className="zed-title-section">Badge</h2>
        <div className="zed-catalog-swatch">
          <Badge tone="neutral">Neutral</Badge>
          <Badge tone="info">Info</Badge>
          <Badge tone="success">Success</Badge>
          <Badge tone="warning">Warning</Badge>
          <Badge tone="danger">Danger</Badge>
          <Badge tone="technical">hash</Badge>
        </div>
      </Card>

      <section id="ds-card" className="zed-stack">
        <h2 className="zed-title-section">Card</h2>
        <div className="zed-catalog-cards">
          <Card variant="document">
            <p className="zed-label">document</p>
            <p className="zed-flush">Ficha de expediente.</p>
          </Card>
          <Card variant="action">
            <p className="zed-label">action</p>
            <p className="zed-flush">Fila accionable.</p>
          </Card>
          <Card variant="metric">
            <p className="zed-label">metric</p>
            <p className="zed-stat-value">12</p>
          </Card>
          <Card variant="outlined">
            <p className="zed-label">outlined</p>
            <p className="zed-flush">Contorno.</p>
          </Card>
        </div>
      </section>

      <section id="ds-callout" className="zed-stack">
        <h2 className="zed-title-section">Callout</h2>
        <Callout variant="info" title="Información" icon="circle-info">
          Receta tokenizada. El foco visible usa --zed-focus-ring.
        </Callout>
        <Callout variant="success" title="Éxito" icon="circle-check">
          Operación sintético-correcta.
        </Callout>
        <Callout variant="warning" title="Aviso" icon="triangle-exclamation">
          No se retira el outline.
        </Callout>
        <Callout variant="danger" title="Error" icon="circle-exclamation">
          Fallo de ejemplo, sin PII.
        </Callout>
        <Callout variant="privacy" title="Privacidad" icon="hard-drive">
          Procesamiento local; sin cuentas.
        </Callout>
        <Callout variant="verification" title="Verificación" icon="clipboard-check">
          I1–I7 viven en el core, no en esta página.
        </Callout>
      </section>

      <Card variant="document" id="ds-formulario" className={formStyles.form}>
        <h2 className="zed-title-section">Formulario</h2>
        <div className={formStyles.field}>
          <label className="zed-label" htmlFor="ds-input">
            Campo de ejemplo
          </label>
          <input
            id="ds-input"
            className="zed-input"
            defaultValue="texto sintético"
          />
        </div>
        <div className={formStyles.field}>
          <label className="zed-label" htmlFor="ds-input-disabled">
            Deshabilitado
          </label>
          <input
            id="ds-input-disabled"
            className="zed-input"
            defaultValue="solo lectura de ejemplo"
            disabled
          />
        </div>
        <div className={formStyles.field}>
          <label className="zed-label" htmlFor="ds-textarea">
            Área de texto
          </label>
          <textarea
            id="ds-textarea"
            className="zed-textarea"
            rows={3}
            defaultValue={"# reglas sintéticas\n[zedazo]\n"}
          />
        </div>
        <div className={formStyles.field}>
          <label className="zed-label" htmlFor="ds-select">
            Selector
          </label>
          <select id="ds-select" className="zed-input" defaultValue="claro">
            <option value="sistema">Sistema</option>
            <option value="claro">Claro</option>
            <option value="oscuro">Oscuro</option>
          </select>
        </div>
        <fieldset className={formStyles.fieldset}>
          <legend className="zed-label">Opciones</legend>
          <div className={formStyles.checkboxRow}>
            <label className={formStyles.checkbox}>
              <input type="checkbox" defaultChecked />
              Conservar VCF
            </label>
            <label className={formStyles.choice}>
              <input type="radio" name="ds-rules-mode" defaultChecked />
              Reglas integradas
            </label>
            <label className={formStyles.choice}>
              <input type="radio" name="ds-rules-mode" />
              TOML personalizado
            </label>
          </div>
        </fieldset>
        <FileDropzone
          title="Zona de archivo de ejemplo"
          description="No se envía nada; es una muestra del control."
          buttonLabel="Elegir VCF de ejemplo"
          onFile={() => undefined}
        />
      </Card>

      <Card variant="document" id="ds-icono">
        <h2 className="zed-title-section">Icono</h2>
        <p className="zed-muted">
          SVG local (<code className="zed-mono">currentColor</code>). Sin{" "}
          <code className="zed-mono">&lt;wa-*&gt;</code>.
        </p>
        <div className="zed-catalog-swatch">
          <Icon name="file-arrow-up" aria-hidden={true} />
          <Icon name="list-check" aria-hidden={true} />
          <Icon name="magnifying-glass" aria-hidden={true} />
          <Icon name="sliders" aria-hidden={true} />
          <Icon name="book" aria-hidden={true} />
          <Icon name="circle-check" aria-hidden={true} />
          <VisuallyHidden>Ejemplo de texto solo para lectores de pantalla.</VisuallyHidden>
        </div>
      </Card>

      <Card variant="document" id="ds-filtro">
        <h2 className="zed-title-section">Filtro</h2>
        <FilterBar
          chips={[{ id: "q", label: "Búsqueda: sintético" }]}
          liveMessage="1 filtro de ejemplo activo"
        >
          <label className="zed-sr-only" htmlFor="ds-filter-q">
            Buscar ejemplo
          </label>
          <input
            id="ds-filter-q"
            className="zed-input zed-input--filter"
            defaultValue="sintético"
          />
        </FilterBar>
      </Card>

      <section id="ds-datos" className="zed-stack">
        <SectionHeading
          title="Datos"
          description="Stat, metadatos y descarga de artefacto."
          action={
            <span className={buttonClassName({ variant: "tertiary", size: "sm" })}>
              Acción de sección
            </span>
          }
        />
        <div className="zed-grid-metrics">
          <StatCard label="Entrada" value={12} hint="vCards sintéticas" />
          <StatCard label="Conservados" value={9} />
          <StatCard label="Revisar" value={2} />
        </div>
        <Card variant="document">
          <MetadataList
            items={[
              {
                label: "ID de ejecución",
                value: "job-sintetico",
                mono: true,
                copyable: true,
              },
              {
                label: "Hash input",
                value:
                  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                mono: true,
              },
            ]}
          />
          <ArtifactDownload
            href="#ds-datos"
            label="VCF"
            description="Agenda resultante (enlace de ejemplo)"
            meta="fixture"
          />
        </Card>
      </section>

      <Card variant="document" id="ds-stepper">
        <h2 className="zed-title-section">Stepper</h2>
        <ProgressStepper
          steps={[
            { id: "archivo", label: "Archivo" },
            { id: "reglas", label: "Reglas" },
            { id: "salidas", label: "Salidas" },
          ]}
          current={1}
        />
      </Card>

      <Card variant="document" id="ds-ejecucion">
        <h2 className="zed-title-section">Ejecución</h2>
        <p className="zed-muted">
          Estados de job con fixture sintético (sin agenda real).
        </p>
        <div className="zed-catalog-swatch">
          <JobStatus status="queued" />
          <JobStatus status="screening" />
          <JobStatus status="completed" />
          <JobStatus status="failed" />
        </div>
        <JobProgress status="screening" phases={["validating", "parsing", "screening"]} />
      </Card>

      <Card variant="document" id="ds-estados">
        <h2 className="zed-title-section">Estados</h2>
        <EmptyState
          compact
          title="Vacío"
          description="Misma ilustración tipográfica en listados y fichas."
        />
        <LoadingState label="Cargando ejemplo…" lines={2} />
        <ErrorState title="Error de ejemplo" message="Mensaje sintético de fallo." />
      </Card>
    </div>
  );
}
