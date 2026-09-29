import { EmptyState } from "@/components/ui/empty-state";

type EventItem = {
  id: string;
  title: string;
  detail?: string;
  time?: string;
};

type Props = {
  events: EventItem[];
};

export function JobTimeline({ events }: Props) {
  if (events.length === 0) {
    return (
      <EmptyState
        compact
        title="Sin eventos"
        description="Esta ejecución aún no ha registrado eventos de progreso."
      />
    );
  }
  return (
    <ol className="zed-timeline">
      {events.map((ev) => (
        <li key={ev.id} className="zed-timeline__item">
          <span aria-hidden className="zed-timeline__dot" />
          <div className="zed-title-section zed-card-kicker">{ev.title}</div>
          {ev.detail ? <p className="zed-muted zed-flush">{ev.detail}</p> : null}
          {ev.time ? <p className="zed-mono zed-muted zed-flush">{ev.time}</p> : null}
        </li>
      ))}
    </ol>
  );
}
