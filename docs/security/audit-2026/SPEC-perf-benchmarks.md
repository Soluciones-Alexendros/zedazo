---
title: "SPEC-perf-benchmarks"
date: "2026-11-18"
status: "Borrador"
module: "perf-benchmarks"
depends_on: []
traza: "H-05 / ESC-5; ROADMAP v1.0.0 Benchmarks (criterion)"
---

# Spec: perf-benchmarks

## Objective

Definir el **contrato** de benchmarks de rendimiento para rutas críticas de dominio, adelantando el ítem del hito **v1.0.0** en [`ROADMAP.md`](../../../ROADMAP.md) («Benchmarks (criterion)»), sin convertirlos aún en presupuesto bloqueante de CI.

Rutas a medir (mínimo):

1. **Parseo VCF** (entrada sintética / fixtures del repo; sin PII real).
2. **Deduplicación** (pipeline de dominio ya cubierto por tests; p. ej. superficie relacionada con `dedup_perf` / reglas D3–D6 documentadas en la revisión OWASP).

Criterios de aceptación (spec):

1. Lista explícita de benches (`benches/*.rs` o convención del crate `zedazo-core`) y qué métrica reportan (tiempo / throughput).
2. `criterion` como **dev-dependency** (confirmación AGENTS §4 antes de añadirla).
3. Comando canónico documentado (`cargo bench -p zedazo-core --bench …`).
4. **No** es gate de `ci-gate` en la primera entrega; tendencia opcional (artifact / informe) queda como ask first.
5. Fixtures 100 % sintéticos (política AGENTS).

## Tech Stack

- Rust workspace; crate objetivo preferente: `zedazo-core` (dominio puro, sin I/O de red).
- `criterion` (versión a fijar en la implementación tras confirmación).
- Fixtures existentes bajo tests/fixtures del repo (reutilizar; no inventar PII).

## Commands

Tras implementación (no en Phase 1):

```bash
cargo bench -p zedazo-core --bench parse_vcf
cargo bench -p zedazo-core --bench dedup
# o el nombre acordado en la implementación
```

Verificación documental de la spec / ROADMAP:

```bash
rg -n "criterion|Benchmarks" ROADMAP.md docs/security/audit-2026/SPEC-perf-benchmarks.md
```

## Project Structure

```text
crates/zedazo-core/benches/     → futuro
crates/zedazo-core/Cargo.toml → dev-dep criterion (futuro)
ROADMAP.md                    → hito v1.0
docs/security/audit-2026/     → esta spec
```

## Code Style

- Nombres de bench estables; sin `unwrap` fuera de harness de bench si el estilo del crate lo exige en tests.
- Datos sintéticos; sin contactos reales.
- Español en docs; ids de bench en inglés snake_case.

## Testing Strategy

- Los benches **no** sustituyen `cargo test --workspace`.
- Primera entrega: corrida local/CI informativa; umbral de regresión (p. ej. fail si >N% más lento) = ask first.
- Evidencia: salida de `cargo bench` en la sesión de implementación cuando se ejecute el módulo.

## Boundaries

- **Always:** fixtures sintéticos; anclar a v1.0 / ROADMAP; dominio en `zedazo-core`.
- **Ask first:** añadir `criterion`; job CI de benches; presupuestos numéricos bloqueantes; benches en api/carddav.
- **Never:** PII real; afirmar ESC-5 «conseguido» solo con esta spec; meter benches en `ci-gate` sin confirmación.

## Success Criteria

- [ ] Spec lista rutas, comando y exclusión de gate.
- [ ] Implementación posterior: al menos un bench de parseo y uno de dedup verdes localmente.
- [ ] ROADMAP/v1.0 actualizado cuando se aterrice (docs canónicos).
- [ ] Phase 1: solo documentación bajo `audit-2026/`.

## Open Questions

1. ¿Confirma el operador la dep `criterion` para el PR de implementación?
2. ¿Tamaño del corpus de bench (¿reutilizar sample-contacts o generar N cards sintéticas)?
3. ¿Publicar tendencia (GitHub Pages / artifact) en v1.0 o solo corrida manual?
