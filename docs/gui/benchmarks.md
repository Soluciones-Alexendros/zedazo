# Benchmarks GUI/API (Fase 6) y escritorio

Objetivo: fixtures sintéticos de ~1k / 10k / 50k contactos.

```bash
# Generar corpus sintético (ejemplo)
# cargo run -p zedazo -- cribar corpus_1k.vcf -o /tmp/out.vcf -a /tmp/a.tsv

# API local
ZEDAZO_DATA_DIR=./data cargo run -p zedazo-api

# Medir upload+job con curl/time; anotar RSS del proceso api
```

Criterios beta: sin bloqueos de UI en wizard, cancelación cooperativa, encoding ISO OK, exportaciones equivalentes a CLI.

## Escritorio AppImage (v0.6.0 / ADR-0021)

Medidas en **Ubuntu 26** (host de prueba) o resumen CI cuando exista. Hasta entonces, placeholders.

| Métrica | Valor | Notas |
|---------|-------|-------|
| Tamaño AppImage (MiB) | _TBD_ | Artefacto del draft `v0.6.0`; rellenar desde summary CI / `stat -c %s` |
| TTFB `GET /health` loopback (ms) | _TBD_ | API same-process en `127.0.0.1`; medir con `curl -o /dev/null -s -w '%{time_starttransfer}\n'` tras arranque |
| RSS proceso AppImage en idle (MiB) | _TBD_ | Opcional; `ps`/`smem` tras ventana estable |

Guía de uso: [docs/desktop.md](../desktop.md).
