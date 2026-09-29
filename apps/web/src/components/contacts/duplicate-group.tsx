import { Card } from "@/components/ui/card";
import { EmptyState } from "@/components/ui/empty-state";
import { DuplicateEvidence } from "./duplicate-evidence";

type Group = {
  canonical_uid: string;
  member_uids: string[];
};

type Props = {
  groups: Group[];
  auditRows?: { cols: string[] }[];
};

export function DuplicateGroup({ groups, auditRows = [] }: Props) {
  if (groups.length === 0) {
    return (
      <EmptyState
        compact
        title="Sin duplicados"
        description="Esta ejecución no tiene grupos de duplicados."
      />
    );
  }
  return (
    <div className="zed-stack">
      {groups.map((g) => (
        <Card key={g.canonical_uid} variant="document">
          <h3 className="zed-flush">Contacto canónico</h3>
          <p className="zed-mono zed-wrap">{g.canonical_uid}</p>
          <h3>Miembros</h3>
          <ul className="zed-prose-list">
            {g.member_uids.map((m) => (
              <li key={m} className="zed-mono zed-wrap">
                {m}
              </li>
            ))}
          </ul>
          <DuplicateEvidence memberUids={g.member_uids} auditRows={auditRows} />
        </Card>
      ))}
    </div>
  );
}
