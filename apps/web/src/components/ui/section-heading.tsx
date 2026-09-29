import type { ReactNode } from "react";

type Props = {
  title: string;
  description?: string;
  action?: ReactNode;
};

export function SectionHeading({ title, description, action }: Props) {
  return (
    <div className="zed-row zed-section-heading">
      <div>
        <h2 className="zed-title-section">{title}</h2>
        {description ? (
          <p className="zed-muted zed-flush">
            {description}
          </p>
        ) : null}
      </div>
      {action}
    </div>
  );
}
