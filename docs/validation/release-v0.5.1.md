# Diagnóstico release `v0.5.1`

**Run:** https://github.com/Soluciones-Alexendros/zedazo/actions/runs/34556887867  
**Tag push SHA (run):** `aea63c1c19b60881b5dcd37cfecd73e43f705093`  
**Assets publicados:** solo `dist-manifest.json`, `sbom.json`, `sbom.json.sha256`.

## Causa

1. El job `plan` escribió el manifiesto completo en `GITHUB_OUTPUT` (`manifest=...`).
2. Actions avisó `Skip output 'val' since it may contain secret` → `needs.plan.outputs.val` vacío.
3. `build-local-artifacts` se evaluó con matriz nula → **skipped**.
4. El job `host` aceptaba `skipped` como válido y creó la release sin binarios.

## Reparación

- Workflow: matriz en output compacto; `host` exige `build-local-artifacts == success` y `sbom == success`; validación de los cinco triples + SBOM antes de `gh release create`; `dist host` solo `--steps=upload`.
- Assets de `v0.5.1`: reconstruir desde el tag original (misma procedencia) cuando CI de release esté corregido, o publicar versión nueva si hace falta cambio de código. **No mover el tag** ni reconstruir desde `main` actual sin decisión humana.

## Recuperación parcial

| Situación | Acción |
|-----------|--------|
| `zedazo-core` en crates.io, CLI pendiente | `workflow_dispatch` de `publish-crates.yml` con el mismo ref |
| Release creada sin assets | No dar por buena; volver a host con gates nuevos o release draft + upload |
