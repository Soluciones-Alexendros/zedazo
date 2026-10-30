import { Badge } from "@/components/ui/badge";
import { Icon } from "@/components/ui/icon";

type JobStatusValue =
  | "created"
  | "uploading"
  | "uploaded"
  | "queued"
  | "validating"
  | "parsing"
  | "screening"
  | "normalizing"
  | "classifying"
  | "deduplicating"
  | "verifying"
  | "writing_artifacts"
  | "completed"
  | "failed"
  | "cancel_requested"
  | "cancelled"
  | "expired"
  | "deleted"
  | string;

type Meta = {
  label: string;
  tone: "neutral" | "info" | "success" | "warning" | "danger" | "technical";
  icon: string;
};

const PROCESSING: Meta = {
  label: "Procesando",
  tone: "info",
  icon: "arrows-rotate",
};

const MAP: Record<string, Meta> = {
  created: { label: "Creado", tone: "neutral", icon: "circle" },
  uploading: { label: "Subiendo", tone: "info", icon: "cloud-arrow-up" },
  uploaded: { label: "Subido", tone: "info", icon: "cloud-check" },
  queued: { label: "En cola", tone: "neutral", icon: "clock" },
  validating: { label: "Validando", tone: "info", icon: "shield-halved" },
  parsing: { label: "Leyendo vCards", tone: "info", icon: "file-lines" },
  screening: { label: "Cribando", tone: "info", icon: "filter" },
  normalizing: { label: "Normalizando", tone: "info", icon: "wand-magic-sparkles" },
  classifying: { label: "Clasificando", tone: "info", icon: "tags" },
  deduplicating: { label: "Detectando duplicados", tone: "info", icon: "copy" },
  verifying: { label: "Verificando", tone: "info", icon: "clipboard-check" },
  writing_artifacts: {
    label: "Generando artefactos",
    tone: "info",
    icon: "box-archive",
  },
  completed: { label: "Completado", tone: "success", icon: "circle-check" },
  failed: { label: "Fallido", tone: "danger", icon: "circle-xmark" },
  cancel_requested: {
    label: "Cancelación pedida",
    tone: "warning",
    icon: "ban",
  },
  cancelled: { label: "Cancelado", tone: "warning", icon: "ban" },
  expired: { label: "Expirado", tone: "neutral", icon: "hourglass-end" },
  deleted: { label: "Eliminado", tone: "neutral", icon: "trash" },
};

export function jobStatusMeta(status: JobStatusValue): Meta {
  return MAP[status] ?? { ...PROCESSING, label: String(status) };
}

type Props = { status: JobStatusValue };

export function JobStatus({ status }: Props) {
  const meta = jobStatusMeta(status);
  return (
    <Badge
      tone={meta.tone}
      icon={<Icon name={meta.icon} aria-hidden={true} />}
    >
      {meta.label}
    </Badge>
  );
}
