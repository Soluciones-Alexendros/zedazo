import { Card } from "./card";

type Props = {
  label: string;
  value: string | number;
  hint?: string;
};

export function StatCard({ label, value, hint }: Props) {
  return (
    <Card variant="metric">
      <p className="zed-label zed-stat-card__label">{label}</p>
      <p className="zed-stat-value zed-stat-card__value">{value}</p>
      {hint ? <p className="zed-muted zed-stat-card__hint">{hint}</p> : null}
    </Card>
  );
}
