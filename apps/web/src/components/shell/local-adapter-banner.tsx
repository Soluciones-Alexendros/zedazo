"use client";

import { usePathname } from "next/navigation";
import { Callout } from "@/components/ui/callout";
import { buttonClassName } from "@/components/ui/button";
import { REAL_VCF_MESSAGE } from "@/lib/data-adapter";

const FIXTURES = [
  ["vacio", "Vacío"],
  ["error", "Error"],
  ["resultado", "Resultado"],
] as const;

export function LocalAdapterBanner() {
  const pathname = usePathname();

  return (
    <Callout variant="warning" title="Adaptador local" icon="hard-drive">
      <p>{REAL_VCF_MESSAGE}</p>
      <nav className="zed-row" aria-label="Fixtures sintéticos">
        {FIXTURES.map(([id, label]) => (
          <button
            key={id}
            type="button"
            className={buttonClassName({ variant: "tertiary" })}
            onClick={() => {
              window.location.assign(`${pathname}?fixture=${id}`);
            }}
          >
            {label}
          </button>
        ))}
      </nav>
    </Callout>
  );
}
