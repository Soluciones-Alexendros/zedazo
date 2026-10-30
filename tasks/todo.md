# Tasks: cierre ship v0.6.0

## Task 0: Baseline

- [x] Registrar SHA y alcance en `tasks/plan.md`

## Task 1: Diagnóstico y reparación E2E

- [x] V-01 antes de V-03; readiness; token redactado; diagnóstico de body vacío
- Verify pendiente en CI del mismo SHA (linuxdeploy local falló al empaquetar; AppDir sí incluye `usr/lib/zedazo/gui`)

## Task 2: AppImage autónoma

- [x] `bundle.resources` + `resolve_static_dir` vía `resource_dir`; smoke sin STATIC_DIR
- Verify: AppDir contiene GUI; CI `desktop-appimage` + smoke autónomo

## Task 3: Producto GUI

- [x] Recorrido en `i7-sample.mjs` (Procesar, error, ciclo, borrado, auditoría)

## Task 4: Offline y seguridad

- [x] Prueba negativa del detector; token no en stdout

## Task 5: CI reproducible + fachadas

- [x] web-ci frozen; gate local; `desktop-preflight|verify|evidence`; check-frozen-lockfile

## Task 6: Identidad

- [x] `build.rs` + `option_env!`; smoke exige 401; sin bump manual (release-please)

## Task 7: Ubuntu 26 + release

- [x] Acta `tasks/ubuntu26-acta.md` + docs; **stop operador**

## Checkpoint

Código listo en `refactor/web-ligera` → CI #107 → fusión a plan → Ubuntu 26 + publicación por Alejandro.
