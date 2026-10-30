# Acta de aceptación Ubuntu 26 (operador)

Plantilla para firmar el **SHA256 concreto** del AppImage entregable. No aprobar “la última AppImage” sin hash.

## Expediente (agente → operador)

| Campo | Valor |
| --- | --- |
| Integración | `plan/zedazo-v060-seguridad` @ `45e27d7691907c4b34659452e51be0c70fa9be82` (#107 squash) |
| Head verde pre-merge | `56fe19605c5373baba4b24fe312192b710f81ca2` (`refactor/web-ligera`) |
| CI verde | [run 41034936451](https://github.com/Soluciones-Alexendros/zedazo/actions/runs/41034936451) — `ci-gate`, ambos Desktop e2e, AppImage, smoke |
| Artefacto CI | job `Desktop AppImage` del mismo run (descargar `desktop-appimage` + `SHA256SUMS`) |
| Versión producto en árbol | `0.5.1` (sin bump manual; `v0.6.0` vía release-please al fusionar plan→main) |
| Puerta restante | Aceptación Ubuntu 26 + autorización de publicación del **mismo** SHA256; crates.io fuera de este hito |

**Nota:** la firma Ubuntu 26 del ship v0.6.0 debe hacerse sobre el AppImage del **commit de release** (PR release-please), no solo sobre el tip del plan. Si el binario se reconstruye, repetir paquete, E2E, offline y esta acta.

## Entorno

| Campo | Valor |
| --- | --- |
| Ubuntu (exacto) | |
| Arquitectura | x86_64 |
| Sesión gráfica | |
| Bibliotecas relevantes (WebKit, FUSE) | |
| Modo de ejecución | FUSE / `--appimage-extract-and-run` |

## Artefacto

| Campo | Valor |
| --- | --- |
| Nombre fichero | (rellenar desde `SHA256SUMS` del artefacto CI o del build de release) |
| SHA256 | |
| Git SHA del commit de release | |
| Versión leída en `/api/v1/version` (con Bearer) | |
| `git_sha` / `build_date` del producto | (≠ `unknown`) |

## Pruebas (marcar)

- [ ] Arranque como usuario normal, fuera del checkout, sin `ZEDAZO_STATIC_DIR`
- [ ] GUI real carga (temas claro/oscuro/sistema legibles)
- [ ] Procesar `examples/sample.vcf` sintético
- [ ] Detalle + descarga VCF y auditoría
- [ ] I7: origen intacto; salida y auditoría verificadas
- [ ] Offline (sin red externa usable)
- [ ] Persistencia y borrado de datos locales
- [ ] Cerrar ventana → servidor loopback inaccesible
- [ ] Fallos gráficos / warnings registrados abajo

## Limitaciones residuales

- Caddy con CSP estática (solo hash theme-boot) no hashea los `__next_f` del export Next; la GUI same-origin servida por la API sí (hashes dinámicos en `text/html`).
- Igualdad byte a byte AppImage (`desktop-appimage-repro`) informativa, fuera de `ci-gate`.
- crates.io no forma parte del hito v0.6.0.
- Metadatos worktree locales con `commondir` bind-mount RO: requieren `sudo umount` fuera del sandbox del agente.

## Aprobación

| Campo | Valor |
| --- | --- |
| Operador | Alejandro |
| Fecha UTC | |
| Decisión | APROBADO / RECHAZADO |
| Firma (hash) | SHA256=… |

**Stop del agente:** la publicación y esta firma las realiza el operador. Reconstruir el binario tras aprobar obliga a repetir paquete, E2E, offline y esta acta.
