"use client";

import { Icon } from "@/components/ui/icon";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { ProductLockup } from "@/components/brand/product-lockup";
import styles from "@/styles/shell.module.css";

export const NAV_ITEMS = [
  { href: "/procesar", label: "Procesar", icon: "file-arrow-up" },
  { href: "/ejecuciones", label: "Ejecuciones", icon: "list-check" },
  { href: "/auditar", label: "Auditar", icon: "magnifying-glass" },
  { href: "/reglas", label: "Reglas", icon: "sliders" },
  { href: "/documentacion", label: "Documentación", icon: "book" },
] as const;

type Props = {
  open: boolean;
  onNavigate?: () => void;
};

export function AppSidebar({ open, onNavigate }: Props) {
  const pathname = usePathname();

  return (
    <aside
      className={styles.sidebar}
      data-open={open}
      aria-label="Navegación principal"
    >
      <div className={styles.brandBlock}>
        <ProductLockup />
      </div>

      <nav aria-label="Secciones">
        <ul className={styles.navList} role="list">
          {NAV_ITEMS.map((item) => {
            const active =
              pathname === item.href || pathname.startsWith(`${item.href}/`);
            return (
              <li key={item.href}>
                <Link
                  href={item.href}
                  className={styles.navLink}
                  aria-current={active ? "page" : undefined}
                  onClick={onNavigate}
                >
                  <Icon name={item.icon} aria-hidden={true} />
                  {item.label}
                </Link>
              </li>
            );
          })}
        </ul>
      </nav>

      <div className={styles.sidebarFooter}>
        <Link
          href="/ajustes"
          className={styles.navLink}
          aria-current={pathname === "/ajustes" ? "page" : undefined}
          onClick={onNavigate}
        >
          <Icon name="gear" aria-hidden={true} />
          Ajustes
        </Link>
        <p className={styles.sidebarNote}>
          Instancia self-hosted · sin cuentas de usuario en V1
        </p>
      </div>
    </aside>
  );
}
