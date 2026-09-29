"use client";

import type { ReactNode } from "react";

/**
 * Frontera Web Awesome (fase 2, opción A acotada — epic #59).
 * Se conserva la dependencia y el bridge `--wa-*` generado.
 * Clases `wa-light` / `wa-dark` en `<html>` (tema). Iconos: SVG local (`Icon`).
 * No se montan componentes compuestos `<wa-*>`.
 */
export function WebAwesomeProvider({ children }: { children: ReactNode }) {
  return children;
}
