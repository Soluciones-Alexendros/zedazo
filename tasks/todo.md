# Tasks: cierre ship v0.6.0

## Task 0: Baseline

- [x] Registrar SHA y alcance en `tasks/plan.md`

## Task 1: Diagnóstico y reparación E2E

- [x] V-01 antes de V-03; readiness; token redactado; diagnóstico de body vacío
- [x] CSP dinámica para `__next_f` (`fd2781e5`); input file sr-only (`56fe1960`)
- [x] CI verde: ambos Desktop e2e + `ci-gate` (run `41034936451`)
- [x] #107 fusionado en `plan/zedazo-v060-seguridad` (`45e27d76`)

## Task 2: AppImage autónoma

- [x] `bundle.resources` + `resolve_static_dir` vía `resource_dir`; smoke sin STATIC_DIR
- [x] CI `desktop-appimage` + smoke autónomo verdes en el mismo run

## Task 3: Producto GUI

- [x] Recorrido en `i7-sample.mjs` (Procesar, error, ciclo, borrado, auditoría)

## Task 4: Offline y seguridad

- [x] Prueba negativa del detector; token no en stdout; e2e no-network verde

## Task 5: CI reproducible + fachadas

- [x] web-ci frozen; gate local; `desktop-preflight|verify|evidence`; check-frozen-lockfile

## Task 6: Identidad

- [x] `build.rs` + `option_env!`; smoke exige 401; sin bump manual (release-please)

## Task 7: Ubuntu 26 + release

- [x] Acta `tasks/ubuntu26-acta.md` con expediente; **stop operador**
- [ ] Aceptación Ubuntu 26 (Alejandro)
- [ ] Publicación autorizada del mismo SHA256 (Alejandro)
- [ ] PR release-please `v0.6.0` tras plan→main (Alejandro)

## Checkpoint

Código integrado en `plan/zedazo-v060-seguridad` @ `45e27d76`. CI del head pre-merge verde. Siguiente: acta Ubuntu 26 + release por el operador.
