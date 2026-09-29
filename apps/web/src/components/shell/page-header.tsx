import type { ReactNode } from "react";

type Props = {
  title: string;
  description?: string;
  actions?: ReactNode;
  eyebrow?: string;
};

export function PageHeader({ title, description, actions, eyebrow }: Props) {
  return (
    <header className="zed-row zed-page-header">
      <div>
        {eyebrow ? (
          <p className="zed-label zed-page-header__eyebrow">{eyebrow}</p>
        ) : null}
        <h1 className="zed-title-page">{title}</h1>
        {description ? (
          <p className="zed-muted zed-page-header__lead">{description}</p>
        ) : null}
      </div>
      {actions ? <div className="zed-row">{actions}</div> : null}
    </header>
  );
}
