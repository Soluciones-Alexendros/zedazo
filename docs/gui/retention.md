# Política de retención

**Versión:** 0.3.0  
**Fecha:** 2026-10-16  
**Traza:** ADR-0015, ADR-0016

## Variables

| Variable | Default | Descripción |
|----------|---------|-------------|
| `ZEDAZO_DATA_DIR` | `/var/lib/zedazo` (local: `./data`) | Raíz de datos |
| `ZEDAZO_STORAGE_MODE` | `ephemeral` | FS + manifest; sin SQLite obligatorio |
| `ZEDAZO_JOB_RETENTION_HOURS` | `24` | TTL tras `completed`/`failed`/`cancelled` (vía `completed_at`) |
| `ZEDAZO_UPLOAD_TTL_HOURS` | `24` | TTL de uploads que nunca generan un trabajo |
| `ZEDAZO_MAX_UPLOAD_BYTES` | `52428800` (50 MiB) | Límite de upload |

## Precisión del TTL

El barrido corre **al arrancar** y **cada hora**. La eliminación no ocurre en el instante exacto del vencimiento: puede demorarse hasta el siguiente barrido.

## Comportamiento

1. Cada job vive en `$ZEDAZO_DATA_DIR/jobs/{uuid-v7}/` con `manifest.json` (escritura atómica).
2. Al arrancar, la API reconcilia manifiestos en disco:
   - Terminales vencidos → borrado.
   - Estados no terminales (pipeline / `queued`) → `failed` con error `proceso_interrumpido` y `completed_at` (sin reanudación).
   - Manifiestos ilegibles o directorios incompletos → cola `pending_deletes.json`.
3. Tras el TTL, el **barrido** marca `expired` y borra el directorio del job (input, outputs, `events.ndjson` y vistas).
4. Uploads en `uploads/` tienen sidecar `.meta.json`; sin trabajo asociado se borran según `ZEDAZO_UPLOAD_TTL_HOURS`.
5. Si `delete_job_dir` falla, la entrada permanece en `pending_deletes.json` y se reintenta en el siguiente barrido. El log no incluye contactos ni secretos.
6. `DELETE /api/v1/jobs/{id}` borra de inmediato (o deja pendiente si el FS falla). Un segundo DELETE del mismo id es **idempotente** (204).
7. `POST /api/v1/admin/wipe` incrementa `wipe_epoch`, cancela workers y limpia `jobs/`, `uploads/` y `tmp/`. Los workers comprueban la época y no recrean datos tras el wipe.
8. Backups bajo `$ZEDAZO_DATA_DIR/backups/` solo si el operador los crea explícitamente (pueden conservar datos ya borrados de la aplicación).

## Privacidad

- Sin telemetría remota por defecto.
- Nombre original del fichero solo como metadato saneado, nunca como ruta.
- El borrado es lógico a nivel de aplicación y sistema de archivos; no se promete borrado físico irrecuperable del medio.
