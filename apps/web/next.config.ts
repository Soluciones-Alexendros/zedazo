import type { NextConfig } from "next";

// Nota (S-02): el header `Cache-Control: no-store, private` para respuestas
// sensibles se aplica en la API (`zedazo-api`, middleware cache_control) y en
// Caddy. `output: "export"` de Next.js no soporta `headers()` en next.config,
// y los estáticos `/_next/**` no contienen PII (solo bundles JS/CSS).
const nextConfig: NextConfig = {
  reactStrictMode: true,
  output: "export",
  images: { unoptimized: true },
};

export default nextConfig;