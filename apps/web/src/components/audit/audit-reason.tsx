type Props = { text: string };

export function AuditReason({ text }: Props) {
  return <p className="zed-muted zed-flush">{text}</p>;
}
