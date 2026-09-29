import Link from "next/link";
import { ZedazoMark } from "./zedazo-mark";
import { ZedazoWordmark } from "./zedazo-wordmark";

type Props = {
  href?: string;
  subtitle?: string;
  size?: 16 | 20 | 24 | 32 | 48;
  className?: string;
};

export function ProductLockup({
  href = "/",
  subtitle = "Procesamiento VCF local",
  size = 24,
  className,
}: Props) {
  const inner = (
    <>
      <span className="zed-lockup__row">
        <ZedazoMark size={size} decorative />
        <ZedazoWordmark />
      </span>
      {subtitle ? <span className="zed-lockup__subtitle">{subtitle}</span> : null}
    </>
  );

  const classes = ["zed-lockup", className].filter(Boolean).join(" ");

  if (href) {
    return (
      <Link href={href} className={classes}>
        {inner}
      </Link>
    );
  }

  return <div className={classes}>{inner}</div>;
}
