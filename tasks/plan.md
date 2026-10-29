# Plan técnico: cierre ship Zedazo v0.6.0

## Baseline

| Ref | SHA |
| --- | --- |
| `origin/main` | `0830b5da99c7c706c8bb5059c01863c67ae3239d` |
| `plan/zedazo-v060-seguridad` | `4ab30e1bc59833ba3b0a525293c39dbc7f9fb971` |
| `refactor/web-ligera` (#107) | `51fb32b953a43ebdb78a72a16d16bcd46969064f` |
| Run CI fallido | `40279090494` / merge `1e1f8de2452b2c1abe3b46909d492b6d8bc7dce2` |

Trabajo en `refactor/web-ligera` hasta verde; destino de fusión: `plan/zedazo-v060-seguridad`. Sin PRs parciales a `main`.

## Componentes

1. **E2E / GUI detalle**: diagnosticar cuerpo vacío en `/ejecuciones?job=`; readiness; selector robusto; no emitir token.
2. **Autonomía AppImage**: `resolve_static_dir` → `resource_dir` Tauri; abortar sin `index.html`; smoke autónomo.
3. **Producto GUI**: recorrido WebView (upload, job, detalle, descarga, error, ciclo de vida) + paridad CLI + I7.
4. **Offline**: unshare + strace + prueba negativa; fallo si falta aislamiento.
5. **CI/Makefile**: `web-ci` frozen; fachadas `desktop-preflight` / `desktop-verify` / `desktop-evidence`; test local del gate.
6. **Identidad**: SHA/fecha embebidos en build; smoke exige 401 + identidad ≠ unknown.
7. **Ubuntu 26 / release**: expediente + acta; publicación solo Alejandro.

## Riesgos

- E2E vacío: posible navegación SPA / hidratación / ruta no servida; no ampliar timeouts a ciegas.
- `resource_dir` Tauri en AppImage: confirmar path real del extract.
- Rutas `scripts/desktop/**` y `.github/**`: fusión por operador (P9).

## Verificación

`make ci`, build AppImage, prueba decisiva fuera de checkout, `run-e2e.sh` ×3, `e2e-no-network.sh`, CI remota `ci-gate`.
