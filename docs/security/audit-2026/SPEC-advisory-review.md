---
title: "SPEC-advisory-review"
date: "2026-11-18"
status: "Borrador"
module: "advisory-review"
depends_on: []
traza: "H-04 / INT-4 SEC-6; S-18 owasp-review-0.6.0; deny.toml + audit.toml"
---

# Spec: advisory-review

## Objective

Renovar la **revisión vencida** de las excepciones RustSec ligadas a Tauri/gtk-rs, sin borrar ignores en el mismo gesto si el advisory sigue aplicable.

IDs en alcance:

| ID | Origen | Archivos | Fecha actual |
|---|---|---|---|
| `RUSTSEC-2024-0429` | glib 0.18 unsound (Tauri/wry) | [`deny.toml`](../../../deny.toml), [`.cargo/audit.toml`](../../../.cargo/audit.toml), [`osv-scanner.toml`](../../../osv-scanner.toml) | 2026-10-25 / revisión OWASP 2026-10-26 |
| `RUSTSEC-2024-0370` | proc-macro-error unmaintained (transitivo Tauri) | mismos | 2026-10-25 |

Criterios de aceptación:

1. Ejecutar `cargo deny check` y `cargo audit` (o `make deny` / `make audit`) y registrar si los advisories siguen reportados sin parche publicable en el grafo actual.
2. Si siguen aplicables: actualizar **motivo + fecha de revisión** en `deny.toml`, `.cargo/audit.toml` y, si aplica, `osv-scanner.toml` / nota en [`docs/security/owasp-review-0.6.0.md`](../owasp-review-0.6.0.md) (S-18).
3. Redactar texto de **issue de seguimiento** para upgrade Tauri/gtk-rs (no crear el issue sin confirmación del operador).
4. **No** quitar el `ignore` en el mismo PR si OSV/RustSec sigue saliendo (política S-18).

## Tech Stack

- `cargo-deny`, `cargo-audit`, opcional `osv-scanner`.
- Documentación de deuda en ROADMAP / owasp-review.
- Sin bump de Tauri en este módulo (eso es el issue de seguimiento).

## Commands

```bash
make deny
make audit
# O equivalentes:
cargo deny check
cargo audit

# Confirmar que los IDs siguen en ignore con fecha renovada tras el cambio:
rg -n "RUSTSEC-2024-0429|RUSTSEC-2024-0370" deny.toml .cargo/audit.toml osv-scanner.toml
```

Comprobar si hay upgrade publicable (lectura):

```bash
cargo tree -i glib -p zedazo-desktop 2>/dev/null || cargo tree -i glib | head -40
```

## Project Structure

```text
deny.toml
.cargo/audit.toml
osv-scanner.toml
docs/security/owasp-review-0.6.0.md   → S-18
ROADMAP.md                            → deuda Tauri / chrono transitivo (contexto)
```

## Code Style

- Comentarios de ignore: motivo, fecha ISO, criterio de revisión («tras upgrade Tauri/gtk-rs»).
- Mismo ID en `deny.toml` y `.cargo/audit.toml` (política SECURITY.md).
- Español en motivos y en el body del issue propuesto.

## Testing Strategy

- Evidencia: salida de `make deny` / `make audit` en la sesión de implementación (PASS o lista de vulns reales nuevas distintas de estos ignores).
- No afirmar PASS si el comando no se ejecutó.
- Regresión: CI job `deny` / `supply-chain` verdes tras renovar fechas.

## Boundaries

- **Always:** mismo ID en audit + deny; fecha de revisión futura o «próximo trimestre» explícita; documentar si el advisory sigue activo.
- **Ask first:** quitar un ignore; subir Tauri/gtk-rs; crear el issue en GitHub; ampliar ignores a otros RUSTSEC.
- **Never:** silenciar una vulnerability con parche publicable; borrar ignore «para subir score» sin upgrade.

## Success Criteria

- [ ] Fechas de revisión actualizadas (o cierre documentado si ya no aplica).
- [ ] `make deny` y `make audit` ejecutados en la sesión de implementación con resultado registrado.
- [ ] Texto de issue de seguimiento listo (creado solo con sí del operador).
- [ ] S-18 / owasp-review actualizado si cambia el estado.

## Open Questions

1. ¿Nueva fecha de revisión concreta (p. ej. 2026-Q1 post-v0.7) o atada a un milestone Tauri?
2. ¿Se crea el issue ahora o se deja solo el body en docs?
3. ¿Se unifica la fecha también en `osv-scanner.toml` en el mismo PR?
