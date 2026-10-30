import type { Metadata, Viewport } from "next";
import { Providers } from "@/components/providers";
import { THEME_BOOT_SCRIPT } from "@/lib/theme";
import { ZEDAZO_WORDMARK } from "@/components/brand/zedazo-wordmark";
import "@/design-system/design-references.css";
import "@/design-system/reset.css";
import "@/design-system/typography.css";
import "@/design-system/motion.css";
import "@/design-system/utilities.css";
import "@/design-system/components.css";
import "./globals.css";

export const metadata: Metadata = {
  title: ZEDAZO_WORDMARK,
  description:
    "Ordena tus contactos. Conserva las decisiones. Procesamiento VCF local y trazable.",
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#f7f2e9" },
    { media: "(prefers-color-scheme: dark)", color: "#16140f" },
  ],
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="es" suppressHydrationWarning>
      <head>
        <script
          dangerouslySetInnerHTML={{ __html: THEME_BOOT_SCRIPT }}
        />
      </head>
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
