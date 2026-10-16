# Estados de job

**Versión:** 0.3.0  
**Fecha:** 2026-10-16  
**Traza:** ADR-0015, ADR-0016

Estados persistidos en `manifest.json` (UUID v7 como `job_id`):

| Estado | Significado |
|--------|-------------|
| `created` | Metadatos creados |
| `uploading` | Recibiendo bytes |
| `uploaded` | Input en disco; hash disponible |
| `queued` | En cola de worker |
| `validating` | Validación VCF / límites |
| `parsing` | Parseo |
| `screening` | Cribado |
| `normalizing` | Normalización |
| `classifying` | Clasificación |
| `deduplicating` | Deduplicación |
| `verifying` | Invariantes I1–I7 |
| `writing_artifacts` | Escritura de salidas |
| `completed` | Éxito (puede incluir `NeedsReview`) |
| `failed` | Error estructurado (p. ej. VCF malformado) |
| `cancel_requested` | Cancelación cooperativa pedida |
| `cancelled` | Cancelado en punto seguro |
| `expired` | TTL de retención superado |
| `deleted` | Borrado lógico/físico |

`completed` con contactos `NeedsReview` **no** es `failed`.
Jobs `cancelled`/`failed`/`expired`: artefactos incompletos no se sirven como resultados definitivos.

### Reinicio del proceso API

Si el proceso termina mientras un job está en pipeline (`queued` … `writing_artifacts`), al arrancar se marca `failed` con `error: proceso_interrumpido` y `completed_at`. No hay reanudación automática. Los jobs terminales en disco se rehidratan en el registro en memoria; los vencidos se borran en la reconciliación.
