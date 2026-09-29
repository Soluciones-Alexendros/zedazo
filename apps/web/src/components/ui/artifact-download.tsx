import { buttonClassName } from "./button";

type Props = {
  href: string;
  label: string;
  description?: string;
  meta?: string;
};

export function ArtifactDownload({ href, label, description, meta }: Props) {
  return (
    <div className="zed-row zed-artifact">
      <div>
        <div className="zed-artifact__label">{label}</div>
        {description ? (
          <p className="zed-muted zed-artifact__meta">{description}</p>
        ) : null}
        {meta ? (
          <p className="zed-mono zed-muted zed-artifact__meta">{meta}</p>
        ) : null}
      </div>
      <a className={buttonClassName({ variant: "secondary" })} href={href}>
        Descargar
      </a>
    </div>
  );
}
