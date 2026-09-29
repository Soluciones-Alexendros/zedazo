---
version: "0.3.2"
date: "2026-09-09"
status: "Aprobado"
canonical: true
supersedes: "v0.3.1"
---

# SPECS.md

### Propósito de este documento

- **Objetivos:** Especificación de producto, invariantes y criterios de aceptación de Zedazo.
- **Estructura:** Propósito → visión → objetivos → no-objetivos → invariantes → criterios.
- **Contenido a integrar según contexto:** No inventes requisitos. Si falta ancla, paras. No copies SPECS de otro producto. CardDAV/red van en PRs separados del dominio/UI.

**Versión:** 0.3.2  
**Fecha:** 2026-09-09  
**Estado:** Aprobado (v0.3.0 calidad; web local ADR-0015; remoto single-user ADR-0016)  
**Canónico:** este archivo. [`docs/spec.md`](docs/spec.md) redirige aquí.

**Documentos relacionados:** [ROADMAP.md](./ROADMAP.md) · [DECISIONS.md](./DECISIONS.md) · [ARCHITECTURE.md](./ARCHITECTURE.md) · [AGENTS.md](./AGENTS.md)

---

## 1. Visión

**Zedazo** (`zedazo`) criba, normaliza, clasifica y deduplica contactos VCF (vCard 4.0/3.0) exportados desde ProtonMail, Google Contacts y Apple iCloud. El núcleo de dominio es compartido por la CLI y, desde v0.5.0 (ADR-0015), por una API HTTP + GUI web self-hosted. Procesamiento 100 % local, sin telemetría remota ni APIs externas por defecto.

## 2. Objetivos

| ID | Objetivo | Criterio de aceptación |
|----|----------|------------------------|
| **O1** | Parsear VCF vCard 4.0 y 3.0 (Proton, Google, Apple) | 100 % contactos parseados de los fixtures de referencia |
| **O2** | Cribar con reglas E\* + C\* y precedencia determinista | Tasa de reducción 5–30 % sobre corpus de referencia |
| **O3** | Normalizar FN, N, TEL, ADR, ORG (RFC 6350 + reglas ES) | 0 FN con `@` en salida; TEL en E.164 o `non_normalizable` |
| **O4** | Clasificar con taxonomía multinivel | Todo contacto conservado con ≥1 categoría N1 |
| **O5** | Fusionar duplicados D1–D2 con cierre transitivo | 0 % duplicados D1–D2 residuales |
| **O6** | Generar auditoría TSV trazable | Una fila por contacto con `DecisionTrace` |
| **O7** | Detectar fuente y versión vCard | `source_detail` correcto en audit/stats |
| **O8** | Exportar CSV y JSON además de VCF 4.0 | Formatos válidos y completos |
| **O9** | Cargar reglas desde TOML | Append por defecto; `replace` para sustitución total |
| **O10** | Paridad funcional CLI↔GUI self-hosted | Misma semántica de resultados sobre fixtures; ver [docs/gui/functional-parity-matrix.md](./docs/gui/functional-parity-matrix.md) |
| **O11** | Acceso remoto self-hosted single-user | HTTPS (proxy) + `ZEDAZO_AUTH_MODE=token`; fail-closed si auth disabled fuera de loopback (ADR-0016) |

## 3. No-objetivos

- APIs externas (CardDAV, Proton, Google People) — hito v0.4.0+; red/proveedor **[ADR-0018](./DECISIONS.md)** (aceptada). CLI: crate `zedazo-carddav` + `zedazo carddav list|pull|put|delete|watch` ([docs/carddav.md](./docs/carddav.md)); Google People, OAuth de proveedor y GUI CardDAV siguen fuera
- TUI interactiva
- Multiusuario / colaboración / edición manual de contactos en GUI (pospuesto; no cubierto por ADR-0016)
- SaaS, cuentas OAuth/OIDC o multi-tenant
- Soporte vCard 2.1
- Mutación de datos binarios (PHOTO, LOGO, SOUND, KEY): solo preservar u omitir
- Sustituir la CLI: permanece canal oficial de automatización y rollback

**En alcance (ADR-0015, hito v0.5.0):** GUI web + API HTTP local single-user sobre `zedazo-core`.

**En alcance (ADR-0016, hito v0.5.1):** misma GUI/API expuesta en red vía Caddy same-origin + auth por token/cookie; un operador; portable a miniPC self-hosted.

## 4. Invariantes de dominio

1. **I1 — Integridad:** Todo `Contact` conservado tiene `uid`, `ScreeningDecision::Conserved`, `source_detail` y ≥1 categoría N1.
2. **I2 — FN canónico:** Ningún `Contact.fn_value` final contiene `@`.
3. **I3 — TEL E.164:** Todo TEL normalizado está en E.164 (`+` + dígitos) o marcado `non_normalizable`.
4. **I4 — Salida canónica:** Toda salida es vCard 4.0 (RFC 6350). Folding ≤75 octetos sin cortar UTF-8 multibyte.
5. **I5 — Compatibilidad de entrada:** Entrada 3.0 aceptada. AGENT/LABEL/MAILER no se propagan a 4.0.
6. **I6 — Auditabilidad:** Toda acción genera entrada inmutable en `audit.tsv` con `DecisionTrace`.
7. **I7 — No destrucción:** El VCF original nunca se sobrescribe; la salida va a otra ruta.

## 5. Criterios de aceptación (MVP — estado)

### Fase 1 — Parseo (hecho en v0.1.0)

- [x] Proton 4.0, Google 3.0, Apple 3.0: parseo sin panic
- [x] VCF vacío → error controlado
- [x] Malformado → error con contexto
- [x] ISO-8859-1 → UTF-8
- [x] Binarios / PHOTO plegado preservados

### Fase 2 — Cribado (parcial)

- [x] C2–C6, E1–E3 en pipeline
- [x] C1/C5/C7, E4/E6 con tests (`screening.rs:272–465`, #21)
- [x] `DecisionTrace` con regla, evidencia y timestamp

### Fase 3 — Normalización (hecho)

- [x] Títulos/cargos, capitalización, siglas, E.164 ES, ORG + forma jurídica

### Fase 4 — Clasificación (hecho N1/N2; N3 en ramas)

- [x] Taxonomía N1/N2 en main
- [ ] N3 + ADR field (rescate `feat/phase4-n3-t4-adr-clean`)

### Fase 5 — Deduplicación (hecho)

- [x] D1/D2 + transitivo; D3–D6 solo propuesta en NOTE

### Fase 6 — Verificación (hecho v0.3.0)

- [x] Módulo `domain::verification` + I1–I7 en pipeline (`verify` + `verify_post` I4/I5/I6 en `cribar.rs:258`)
- [x] `audit.tsv` y `stats` operativos

### Fase 7 — Exportación (hecho)

- [x] VCF 4.0, CSV, JSON

## 6. Métricas de calidad

| Métrica | Objetivo |
|---------|----------|
| Cobertura unitaria | ≥ 80 % (`cargo llvm-cov` → Coveralls) |
| Tests de integración | ≥ 10 escenarios |
| Tiempo (fixture ~2 KB) | < 1 s |
| Binario release | < 8 MB |
| Sin panics en inputs malformados | 100 % (dominio sin `unwrap` de fallo) |

## 7. Definition of Done (cambio de código)

- Cumple criterios de aceptación citados
- `make ci` verde (fmt, clippy `-D warnings`, test, check, doc, docs-validate, parity, web-ci)
- Documentación canónica actualizada si cambia contrato
- Sin secretos ni PII en fixtures/logs
