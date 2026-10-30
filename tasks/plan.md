# Plan técnico: cierre ship Zedazo v0.6.0

## Baseline (actualizado tras #107)

| Ref | SHA |
| --- | --- |
| `origin/main` | `0830b5da99c7c706c8bb5059c01863c67ae3239d` |
| `plan/zedazo-v060-seguridad` | `45e27d7691907c4b34659452e51be0c70fa9be82` (#107) |
| Head verde pre-merge | `56fe19605c5373baba4b24fe312192b710f81ca2` |
| CI verde | [41034936451](https://github.com/Soluciones-Alexendros/zedazo/actions/runs/41034936451) |

`refactor/web-ligera` fusionada. Sin PRs parciales a `main`. plan→main: operador (P9).

## Componentes (estado)

1. **E2E / GUI**: verde (CSP dinámica + input file sr-only).
2. **AppImage autónoma**: verde (resources + smoke sin `ZEDAZO_STATIC_DIR`).
3. **Producto GUI**: recorrido en `i7-sample.mjs`.
4. **Offline**: detector + prueba negativa verdes.
5. **CI/Makefile**: web-ci frozen; fachadas desktop-*.
6. **Identidad**: SHA/fecha embebidos; smoke exige 401.
7. **Ubuntu 26 / release**: expediente en `tasks/ubuntu26-acta.md` — **stop operador**.

## Riesgos residuales

- Metadatos worktree locales con `commondir` montado RO (requiere `umount` de root).
- CSP Caddy estática no hashea `__next_f` (la API sí al servir HTML).
- Rutas `scripts/desktop/**` y `.github/**`: fusión por operador (P9).

## Verificación

`make ci`, build AppImage, prueba decisiva fuera de checkout, E2E ×3, `e2e-no-network.sh`, CI remota `ci-gate`.
