# Evidencia: workflow_dispatch bloqueado hasta merge

**Fecha:** 2026-11-18  
**PR:** https://github.com/Soluciones-Alexendros/zedazo/pull/132  
**Rama:** `docs/security-audit-2026`

## Intento

```bash
gh workflow run codeql.yml --ref docs/security-audit-2026
```

## Resultado

`HTTP 404: workflow codeql.yml not found on the default branch`

GitHub solo permite `workflow_dispatch` cuando el fichero del workflow existe en la rama por defecto (`main`). CodeQL, Scorecard y `sbom-crates` viven solo en este PR hasta el merge.

## Acción tras merge a main

```bash
gh workflow run codeql.yml --ref main
gh workflow run scorecard.yml --ref main
gh workflow run sbom-crates.yml --ref main -f ref=v0.6.4
```

No es un fallo del YAML: es la limitación de la API de Actions.
