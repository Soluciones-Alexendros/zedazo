# Informe ejecutivo de estado — zedazo

**Fecha**: 2026-11-17 · **Perfil**: P1 · **Semáforo**: rojo · **Health score**: 50/100

## Veredicto en una frase

El producto está maduro y el CI de calidad es verde en su práctica totalidad, pero
hay **una vulnerabilidad bloqueante en el grafo Rust (`quinn-udp`)** que tumba el gate
de seguridad y `ci-gate`, y **dos alertas Dependabot medium abiertas** en `apps/web`.
El Renovate de mantenimiento de lockfiles está además rompiendo el build Docker por un
desajuste de MSRV.

## Datos clave

| Dato | Valor |
| --- | --- |
| Rama / HEAD local | `main` @ `c040aac3` (1 commit detrás de `origin/main`) |
| HEAD remoto | `e87581f9` chore(main): release 0.6.1 (#113) |
| Árbol | limpio |
| Versión | 0.6.0 (manifest; release 0.6.1 en remoto) |
| Lenguaje | Rust (workspace) + Next.js (`apps/web`) + Tauri (`apps/desktop`) |
| Ficheros | 349 · ~2,1 MiB |
| Tests | 29 ficheros de test / 162 de código (17,9 %) |

## Estado de CI en `main` (push release 0.6.1, run 43198590962)

- **Verdes**: Format, Clippy, Doc, Test, Check (stable y MSRV 1.80), Deny, Coverage,
  Parity CLI↔API, Web, Compose smoke, AppImage (+ repro SHA), Desktop e2e (ambos),
  Web E2E API real, secrets, Health, quality, smoke, SBOM, attest.
- **Rojos**: `sca / sca` (cargo audit) → arrastra `ci-gate` a rojo.
- **Único fallo real en main**: el advisory de RustSec descrito abajo.

## Hallazgos por severidad

### P0 — Bloqueante
1. **RUSTSEC-2026-0538** · `quinn-udp 0.5.15` (vulnerabilidad). Cadena:
   `quinn 0.11.11 → reqwest 0.12.28 → zedazo-carddav`. Solución: `>=0.5.16,<0.6.0`.
   Tumba `Security Audit` (schedule) y `Security / sca` (push/PR/schedule) y, por
   agregación, `ci-gate`. **Verificado localmente** con `cargo audit` y
   `cargo update -p quinn-udp --dry-run` (resuelve a 0.5.16 sin tocar nada más).

### P1 — Alto
2. **Dependabot medium abiertas** (npm, `apps/web`):
   - `minimatch` — CVE-2026-118764 (DoS por amplificación globparts), GHSA-g3hv-75wg-m249.
   - `glob` — CVE-2026-118773 (bypass AbortSignal → traversal post-timeout), GHSA-3g3m-2fjr-6pw4.
   Ambas transitivas de la cadena brace-expansion/minimatch/glob.
3. **Renovate lock-file-maintenance (#114) rompe CI**: el refresh de locks sube
   `encoding_rs` a 0.8.44 (requiere rustc 1.88) y `uuid` a 1.29.2 (requiere 1.89),
   pero `deploy/Dockerfile.api` fija `rust:1.85-bookworm`. El Compose smoke falla.
   Mismo problema afectaría al check MSRV 1.80 al fusionar.
4. **Desktop e2e inestable**: `unexpected alert open` (WebDriver) en `tauri-driver`
   y `no network` en el PR #114 (verde en main). Flaky o inducido por el refresh.

### P2 — Bajo / higiene
5. **Duplicado exacto**: `crates/zedazo-core/tests/fixtures/sample-contacts.vcf` ==
   `examples/sample.vcf` (2 224 B, sha256 `14212ea18318f285`).
6. **Estructura**: falta `.env.example` (documentar `ZEDAZO_*`). Los `LICENSE`,
   `docs/architecture`, `docs/guides`, `docs/runbooks`, `docs/README.md` que reporta el
   escáner genérico son **falsos positivos**: el repo usa licencia dual
   `LICENSE-MIT` + `LICENSE-APACHE` y `docs/architecture.md`/`domain.md` con su propia
   convención. `ISSUE_TEMPLATE` en `.md` (legacy) funciona; no es defecto.
7. **Avisos RustSec no bloqueantes** (informativos, no rompen el gate):
   - `chrono 0.4.45` unmaintained (RUSTSEC-2026-0543) — dep directa de `zedazo-api`;
     el workspace ya incluye `jiff` como alternativa.
   - `rustix 1.1.4` unsound (RUSTSEC-2026-0486) — vía `tempfile`.

### Descartado
- «Asignación de credencial» en `apps/desktop/src/loopback.rs:308`: es un token de
  test hardcodeado (`tokensecret32byteshexpad00000001`) dentro de un `#[tokio::test]`.
  **No es un secreto real.** No rotar nada.

## Conclusiones

- **Acción única de mayor impacto**: `cargo update -p quinn-udp` (0.5.15 → 0.5.16)
  repara de golpe `Security Audit`, `Security/sca` y `ci-gate` en main y en todos los PR.
- El CI de calidad (fmt/clippy/test/check/parity/web/AppImage/e2e) está sano; no hay
  deuda de calidad estructural que impida el trabajo.
- El mantenimiento de dependencias necesita una decisión explícita sobre MSRV
  (mantener 1.80 con pins, o subir el mínimo) para que Renovate deje de romper el build.
