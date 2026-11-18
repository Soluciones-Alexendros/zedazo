# Revisión OpenSSF Scorecard

Workflow: [`.github/workflows/scorecard.yml`](../../.github/workflows/scorecard.yml).

- Cadencia del job: día 1 de cada mes, 08:00 UTC, y `workflow_dispatch`.
- Dónde leer el resultado: pestaña Security → Code scanning (SARIF) del run, y la publicación que hace la action con `publish_results: true` hacia `api.scorecard.dev`.
- No es check de `ci-gate`. No hay badge en el README.
- Revisión humana: **una vez al año**. Anotar fecha, puntuación y qué cambió respecto a la fila anterior.

| Fecha | Puntuación | Run | Notas |
|---|---|---|---|
| | | | Primera fila vacía hasta el primer `workflow_dispatch` tras el merge. |
