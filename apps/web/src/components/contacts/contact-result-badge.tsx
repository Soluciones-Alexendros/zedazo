import { Badge } from "@/components/ui/badge";
import { Icon } from "@/components/ui/icon";

type ContactResult =
  | "conserved"
  | "needs_review"
  | "eliminated"
  | "quarantine"
  | string;

type Meta = {
  label: string;
  tone: "success" | "warning" | "danger" | "info" | "neutral";
  icon: string;
};

const MAP: Record<string, Meta> = {
  conserved: {
    label: "Conservado",
    tone: "success",
    icon: "shield-check",
  },
  needs_review: {
    label: "Revisar",
    tone: "warning",
    icon: "triangle-exclamation",
  },
  eliminated: {
    label: "Descartado",
    tone: "danger",
    icon: "circle-xmark",
  },
  quarantine: {
    label: "Cuarentena",
    tone: "info",
    icon: "box-archive",
  },
};

function contactResultMeta(result: ContactResult): Meta {
  return (
    MAP[result] ?? {
      label: String(result),
      tone: "neutral",
      icon: "circle",
    }
  );
}

type Props = { result: ContactResult };

export function ContactResultBadge({ result }: Props) {
  const meta = contactResultMeta(result);
  return (
    <Badge
      tone={meta.tone}
      icon={<Icon name={meta.icon} aria-hidden={true} />}
    >
      {meta.label}
    </Badge>
  );
}
