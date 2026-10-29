# Acta de aceptación Ubuntu 26 (operador)

Plantilla para firmar el **SHA256 concreto** del AppImage entregable. No aprobar “la última AppImage” sin hash.

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
| Nombre fichero | |
| SHA256 | |
| Git SHA del commit de release | |
| Versión leída en `/api/v1/version` (con Bearer) | |
| `git_sha` / `build_date` del producto | |

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

(escribir)

## Aprobación

| Campo | Valor |
| --- | --- |
| Operador | Alejandro |
| Fecha UTC | |
| Decisión | APROBADO / RECHAZADO |
| Firma (hash) | SHA256=… |

**Stop del agente:** la publicación y esta firma las realiza el operador. Reconstruir el binario tras aprobar obliga a repetir paquete, E2E, offline y esta acta.
