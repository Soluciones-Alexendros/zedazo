---
title: "SPEC-vuln-report-channel"
date: "2026-11-18"
status: "Borrador"
module: "vuln-report-channel"
depends_on: []
traza: "H-01 / SEC-5; docs/security/audit-2026/MAP.md"
---

# Spec: vuln-report-channel

## Objective

Dejar un canal **privado y verificable** para reportar vulnerabilidades, y alinear la tabla de versiones soportadas con el tag publicado **v0.6.4**.

Criterios de aceptación:

1. [`SECURITY.md`](../../../SECURITY.md) deja de decir solo «envía un correo a los mantenedores» sin contacto.
2. Incluye enlace a **GitHub Private Vulnerability Reporting** del repo `Soluciones-Alexendros/zedazo` (o instrucción explícita de usarlo cuando el operador lo active).
3. Incluye una dirección o alias de contacto aportado por el operador (no inventado en esta spec).
4. La tabla «Versiones soportadas» refleja la línea activa **0.6.x** (tag v0.6.4) y degrada 0.5.x / 0.3.x de forma coherente con el ROADMAP.
5. Se conserva la regla: no abrir issue público para avisos de seguridad; plazo de respuesta 48 h.

## Tech Stack

- Documentación Markdown canónica del repo.
- GitHub Security Advisories / Private Vulnerability Reporting (ajuste de repo; paso del operador).
- Sin dependencias Cargo ni cambios de CI en este módulo.

## Commands

Verificación documental (solo lectura / grep):

```bash
# El canal deja de ser genérico
rg -n "mantenedores del proyecto|Private Vulnerability|security@" SECURITY.md

# Tabla alineada a 0.6
rg -n "0\\.6|0\\.5|Versiones soportadas" SECURITY.md

# Tag publicado coherente con ROADMAP
rg -n "v0\\.6\\.4" ROADMAP.md
```

Activación en GitHub (operador, fuera del diff de código):

```bash
# Comprobar si Private Vulnerability Reporting está habilitado (API)
gh api repos/Soluciones-Alexendros/zedazo --jq '.security_and_analysis.secret_scanning.status // "n/a"'
# UI: Settings → Code security → Private vulnerability reporting → Enable
```

## Project Structure

```text
SECURITY.md                          → canal de reporte + versiones
docs/security/audit-2026/MAP.md      → mapa
docs/security/audit-2026/SPEC-*.md   → esta spec
```

## Code Style

- Español; wordmark `zedazo` en minúsculas en lockups; prosa «Zedazo».
- No copiar un `SECURITY.md` de otro producto.
- Enlaces relativos a docs internos; URL absoluta solo para GitHub Security Advisories del repo.

## Testing Strategy

- Revisión humana: un investigador puede seguir el canal sin preguntar en issues públicos.
- `make docs-validate` (o el gate documental del repo) tras el cambio de `SECURITY.md`.
- No hay tests de código en este módulo.

## Boundaries

- **Always:** conservar plazo 48 h; prohibir issue público para vulns; alinear versiones con el tag real.
- **Ask first:** dirección de correo / alias concreto; habilitar Private Vulnerability Reporting en la UI de GitHub; añadir clave PGP.
- **Never:** inventar `security@…`; abrir issue público con detalles de vuln; afirmar que el canal está activo en GitHub sin evidencia del operador.

## Success Criteria

- [ ] `SECURITY.md` contiene contacto verificable **y/o** enlace a Private Vulnerability Reporting.
- [ ] Tabla de versiones menciona **0.6.x** como línea activa.
- [ ] Diff limitado a `SECURITY.md` (más docs de auditoría si se enlazan).
- [x] Private Vulnerability Reporting habilitado (API `enabled: true`, 2026-11-18).

## Open Questions

1. ¿Qué dirección de correo o alias publica el operador? (no se inventa aquí)
2. Resuelta el 2026-11-18: Private Vulnerability Reporting está activado (`enabled: true` en la API del repo).
3. ¿Se incluye clave PGP pública en este módulo o se aplaza?
