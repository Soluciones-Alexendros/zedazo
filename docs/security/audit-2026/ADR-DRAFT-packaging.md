# Borrador — empaquetado multi-arch

Estado: **borrador**. No está aceptado. No edita [`DECISIONS.md`](../../../DECISIONS.md).

- Fecha: 2026-11-18
- Contexto: el informe de auditoría pidió matriz multi-distro y multi-arquitectura. ADR-0021 fija AppImage Linux x86_64 (prueba en Ubuntu 26). ADR-0012 fija cargo-dist para la CLI y deja la validación de releases reales en v1.0. [`dist-workspace.toml`](../../../dist-workspace.toml) solo construye `x86_64-apple-darwin`.
- Decisión propuesta por defecto: **(A)** mantener ese status quo hasta la validación real del hito v1.0. No se añaden runners, `.deb` ni `.rpm` en este cambio.
- Alternativa (B): ampliar targets de cargo-dist cuando haya runners que terminen en verde, sin tocar el AppImage de ADR-0021.
- Alternativa (C): añadir `.deb` o `.rpm`. Exige un ADR aceptado y el mismo checksum más atestación que [`docs/desktop.md`](../../desktop.md).
- Consecuencias de aceptar (A): la brecha queda documentada en [packaging-gap.md](./packaging-gap.md). Aceptar (B) o (C) es otro PR, con confirmación humana (AGENTS §4).
