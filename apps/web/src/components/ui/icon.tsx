type Props = {
  name: string;
  className?: string;
  "aria-hidden"?: boolean | "true" | "false";
  style?: React.CSSProperties;
};

/** Iconos lineales locales (sin CDN). Nombres alineados con FA/Web Awesome. */
const PATHS: Record<string, string> = {
  "file-arrow-up":
    "M14 3v4h4M14 3l5 5v11a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7m-3 14v-8m0 0l-3 3m3-3 3 3",
  "list-check": "M9 6h11M9 12h11M9 18h11M4 6h.01M4 12h.01M4 18h.01",
  "magnifying-glass": "M11 11 16 16M10 17a7 7 0 1 1 0-14 7 7 0 0 1 0 14z",
  sliders: "M4 8h10M18 8h2M14 6v4M4 16h4M12 16h8M8 14v4",
  book: "M5 4h11a3 3 0 0 1 3 3v13H8a3 3 0 0 0-3 3V4zm0 0v16",
  gear: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zm7.4-3a7.4 7.4 0 0 0-.1-1l2-1.5-2-3.5-2.4 1a7 7 0 0 0-1.7-1L13 3h-2l-.2 2.9a7 7 0 0 0-1.7 1L6.7 6l-2 3.5 2 1.5a7.4 7.4 0 0 0 0 2l-2 1.5 2 3.5 2.4-1a7 7 0 0 0 1.7 1L11 21h2l.2-2.9a7 7 0 0 0 1.7-1l2.4 1 2-3.5-2-1.5c.1-.3.1-.7.1-1z",
  bars: "M4 7h16M4 12h16M4 17h16",
  "circle-question":
    "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zm-1-7h2v2h-2zm1-9a3 3 0 0 1 2.8 4c-.5.8-1.8 1.4-1.8 3h-2c0-2 1.5-2.7 2-3.2A1.5 1.5 0 1 0 12 8",
  copy: "M8 8V5a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-3M5 10h9a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2z",
  xmark: "M6 6l12 12M18 6 6 18",
  "circle-info":
    "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zm-1-6h2v2h-2zm0-8h2v6h-2z",
  "circle-exclamation":
    "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zm-1-6h2v2h-2zm0-8h2v6h-2z",
  "circle-check":
    "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zm-1.5-6 5-5-1.4-1.4-3.6 3.6-1.6-1.6L8 14.1l2.5 1.9z",
  "circle-xmark":
    "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zm-3.5-6.5 7-7M15.5 15.5l-7-7",
  "arrows-rotate":
    "M20 12a8 8 0 0 1-14 5.3M4 12a8 8 0 0 1 14-5.3M20 5v5h-5M4 19v-5h5",
  "cloud-arrow-up":
    "M8 18h8a4 4 0 0 0 0-8 5 5 0 0 0-9.6-1.5A3.5 3.5 0 0 0 8 18zm4-2v-7m0 0-2.5 2.5M12 9l2.5 2.5",
  "cloud-check":
    "M8 18h8a4 4 0 0 0 0-8 5 5 0 0 0-9.6-1.5A3.5 3.5 0 0 0 8 18zm1-4 2 2 4-4",
  clock: "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zm0-10V7m0 5 3.5 2",
  "shield-halved": "M12 3 4 7v5c0 5 3.4 8.4 8 10 4.6-1.6 8-5 8-10V7l-8-4zm0 0v19",
  "file-lines":
    "M14 3v4h4M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8l-5-5zM8 13h8M8 17h8M8 9h3",
  filter: "M4 5h16l-6 7v5l-4 2v-7L4 5z",
  "wand-magic-sparkles":
    "m15 4 1 3 3 1-3 1-1 3-1-3-3-1 3-1 1-3zM5 16l9-9 3 3-9 9H5v-3z",
  tags: "M3 8v4l9 9 7-7-9-9H3zm3.5 1.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2z",
  "clipboard-check":
    "M9 4h6a2 2 0 0 1 2 2v1h1a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V9a2 2 0 0 1 2-2h1V6a2 2 0 0 1 2-2zm0 10 2 2 4-4",
  "box-archive":
    "M3 7h18v3H3V7zm2 3v9a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-9M10 13h4",
  ban: "M4.9 4.9l14.2 14.2M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z",
  "hourglass-end": "M6 3h12M6 21h12M7 3c0 5 5 5 5 9s-5 4-5 9m10-18c0 5-5 5-5 9s5 4 5 9",
  trash:
    "M5 7h14m-1 0-.8 12a2 2 0 0 1-2 1.9H8.8a2 2 0 0 1-2-1.9L6 7m3 0V5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2",
  circle: "M12 20a8 8 0 1 0 0-16 8 8 0 0 0 0 16z",
  "shield-check":
    "M12 3 4 7v5c0 5 3.4 8.4 8 10 4.6-1.6 8-5 8-10V7l-8-4zm-1.5 11 4.5-4.5L13.6 8l-3.1 3.1-1.5-1.5L7.5 11l3 3z",
  "triangle-exclamation":
    "M12 4 3 20h18L12 4zm0 5v5m0 3h.01",
  "hard-drive":
    "M4 12h16v6a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-6zm0 0 2.5-6.5A2 2 0 0 1 8.4 4h7.2a2 2 0 0 1 1.9 1.5L20 12M15 16h.01",
  link: "M10 14a4 4 0 0 0 6 0l3-3a4 4 0 0 0-6-6l-1 1M14 10a4 4 0 0 0-6 0l-3 3a4 4 0 0 0 6 6l1-1",
  "file-magnifying-glass":
    "M14 3v4h4M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h4m7-7a3 3 0 1 1-6 0 3 3 0 0 1 6 0zm1.5 4.5L21 21",
  flask: "M9 3h6M10 3v6l-5 9a2 2 0 0 0 1.8 3h10.4A2 2 0 0 0 19 18l-5-9V3",
  "clock-rotate-left":
    "M3 12a9 9 0 1 0 3-6.7M3 5v4h4M12 8v5l3 2",
};

export function Icon({ name, className, style, ...rest }: Props) {
  const d = PATHS[name] || PATHS.circle;
  return (
    <svg
      width="1em"
      height="1em"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.75"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={["zed-icon", className].filter(Boolean).join(" ")}
      style={style}
      {...rest}
    >
      <path d={d} />
    </svg>
  );
}
