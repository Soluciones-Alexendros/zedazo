# Protección de rama main — Spec (histórico + actualizado)

**Feature:** `proteccion-main`  
**Versión spec:** 1.1.0  
**Fecha:** 2026-08-15  
**Estado:** Objetivo de protección de `main` **cumplido**. Dependabot **sustituido por Renovate** (ADR-0007).

---

## Objetivo (histórico)

Proteger `main`, estabilizar CI y documentar el estado post-merge.

## Requerimientos (estado)

| ID | Descripción | Estado |
|----|-------------|--------|
| RF-1 | Branch protection en `main` | ✅ |
| RF-2/3/4 | PRs Dependabot históricos | ✅ / obsoleto |
| RF-5 | Excluir bumps que suban MSRV | ✅ vía Renovate `allowedVersions` |
| RF-6 | README badges y docs | ✅ (org Soluciones-Alexendros en v0.1.1) |
| RF-7 | Roadmap | ✅ [ROADMAP.md](../../ROADMAP.md) |
| RF-8 | Arquitectura CI | ✅ [ARCHITECTURE.md](../../ARCHITECTURE.md) |
| RF-9 | Doc warnings | Revisar en CI |

## Contrato actual de dependencias

Ver [contract.md](./contract.md) (Renovate, no Dependabot).
