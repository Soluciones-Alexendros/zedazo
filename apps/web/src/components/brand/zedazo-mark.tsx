import { ZEDAZO_WORDMARK } from "./zedazo-wordmark";

type Props = {
  size?: 16 | 20 | 24 | 32 | 48;
  title?: string;
  className?: string;
  /** Junto al wordmark el logomark es decorativo (como en la landing). */
  decorative?: boolean;
};

/** Marca vectorial: tres nodos conectados con trazo en Z sutil. */
export function ZedazoMark({
  size = 24,
  title,
  className,
  decorative = false,
}: Props) {
  const label = title ?? ZEDAZO_WORDMARK;
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      className={className}
      fill="none"
      {...(decorative
        ? { "aria-hidden": true as const }
        : { role: "img" as const, "aria-label": label })}
    >
      {decorative ? null : <title>{label}</title>}
      <path
        d="M8 7h12L10 16h11"
        stroke="currentColor"
        strokeWidth="2.2"
        strokeLinecap="round"
        strokeLinejoin="round"
        opacity="0.35"
      />
      <circle cx="8" cy="7" r="3" fill="currentColor" />
      <circle cx="16" cy="16" r="3.25" fill="currentColor" />
      <circle cx="24" cy="25" r="3" fill="currentColor" />
      <path
        d="M10.2 9.2 14.1 14.1M17.9 17.9 21.8 22.8"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
      />
    </svg>
  );
}
