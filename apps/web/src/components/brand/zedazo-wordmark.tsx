type Props = {
  className?: string;
};

/** Wordmark canónico (ADR-0014 / #49): minúsculas, patrón Atlaps. */
export const ZEDAZO_WORDMARK = "zedazo";

export function ZedazoWordmark({ className }: Props) {
  return (
    <span className={["zed-wordmark", className].filter(Boolean).join(" ")}>
      {ZEDAZO_WORDMARK}
    </span>
  );
}
