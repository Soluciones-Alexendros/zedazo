# Brecha de empaquetado

Fecha: 2026-11-18. No cambia runners ni formatos.

| Superficie | Hecho | Límite |
|---|---|---|
| AppImage Linux x86_64 | ADR-0021 y [`.github/workflows/publish.yml`](../../../.github/workflows/publish.yml). Prueba manual: Ubuntu 26. | Sin AppImage aarch64. |
| CLI multiplataforma | ADR-0012 (cargo-dist). Validación real aplazada a v1.0 en [`ROADMAP.md`](../../../ROADMAP.md). | [`dist-workspace.toml`](../../../dist-workspace.toml) solo tiene `x86_64-apple-darwin`. El comentario del fichero dice que Linux/Windows fallan en cargo-dist 0.32 y que el runner aarch64 se canceló. |
| Docker Compose | Guía en `docs/gui/deploy.md`. | No es una matriz multi-arch del binario. |
| `.deb` / `.rpm` | No hay empaquetado Debian ni RPM en el árbol. | Fuera de los ADR aceptados. |

Cualquier artefacto futuro usa el mismo modelo que la sección «Verificar el AppImage» de [`docs/desktop.md`](../../desktop.md): `SHA256SUMS` y `gh attestation verify`.

El borrador de decisión está en [ADR-DRAFT-packaging.md](./ADR-DRAFT-packaging.md). No sustituye a ADR-0012 ni a ADR-0021.
