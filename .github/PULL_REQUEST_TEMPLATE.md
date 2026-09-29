<!-- canon-managed: true -->

### Propósito de este documento

- **Objetivos:** Plantilla de PR para describir el cambio y exigir checks locales más jobs `quality` / `test` / `smoke`.
- **Estructura:** Qué cambia → tipo → checklist (make ci, tests, docs, CI canónico).
- **Contenido a integrar según contexto:** Adapta el checklist a Rust/`make ci` de este repo. No copies plantillas de landing/SaaS. No reescribas jobs maduros.

## Descripción

<!-- Explica qué hace este PR y por qué -->

## Tipo de cambio

- [ ] Bug fix
- [ ] Nueva feature
- [ ] Breaking change
- [ ] Documentación
- [ ] Refactor
- [ ] CI/CD

## Checklist

- [ ] He ejecutado `make ci` y pasa
- [ ] He añadido tests para los cambios
- [ ] He actualizado la documentación si es necesario
- [ ] He seguido las convenciones de código del proyecto
- [ ] CI `quality` / `test` / `smoke` en verde (wrappers; no sustituyen fmt/clippy/check)

## Issues relacionados

<!-- Referencia a issues con #123 -->
