# Tasks: remediación de hallazgos P0–P2

> Plan en `tasks/plan.md`. Rama `fix/remediacion-p0-p2` → PR #115.
> Estado: **8/8 tareas cerradas** (2026-11-17). CI verde: 41 pass / 0 fail, `ci-gate` OK.

## Task 1: Bump quinn-udp 0.5.15 → 0.5.16 (RUSTSEC-2026-0538) — HECHA

- [x] `cargo update -p quinn-udp` (sólo toca ese crate; `cargo audit` limpio).
- [x] Acceptance: `cargo audit` sin vulnerabilidades; quedan los 2 avisos permitidos.
- [x] Efecto colateral documentado en el commit: el resolver re-unifica `windows-sys`
      a 0.52.0/0.48.0 para errno/ring/rustix/tempfile/winapi-util/dirs-sys (cfg(windows)).
- [x] Commit `fix(deps): quinn-udp 0.5.16 (RUSTSEC-2026-0538)`.

## Task 2: Alertas Dependabot minimatch + glob — HECHA

- [x] **Corrección al plan**: las alertas NO estaban en `apps/web` (allí `minimatch`
      está overrideado a 11.0.3) sino en `scripts/desktop/e2e/package-lock.json`,
      cadena `webdriverio → @wdio/config → glob → minimatch` y `archiver → readdir-glob`.
- [x] `webdriverio ^10.1.0 → ^10.2.0` (@wdio/config 10.2.0 exige `glob ^14.0.2`:
      parchea GHSA-3g3m-2fjr-6pw4).
- [x] Override `readdir-glob ^4.0.0` (arrastra `minimatch ^11.0.1`, parchea
      GHSA-g3hv-75wg-m249) en vez de 10.2.6.
- [x] Verification: `npm audit` = 0 vulnerabilities; `npm ci --ignore-scripts
      --prefix scripts/desktop/e2e` OK; `node --check` del spec OK.

## Task 3: Pins Renovate Opción A — HECHA

- [x] **Corrección al plan**: `allowedVersions` **no** contiene `lockFileMaintenance`
      (Renovate borra el lock y ejecuta el package manager; verificado en su doc).
      Además de los pins se desactiva `lockFileMaintenance` sólo para `cargo`.
- [x] `allowedVersions`: `encoding_rs < 0.8.40`, `uuid < 1.27.0` (0.8.40 y 1.27.0
      declaran rustc 1.88 y 1.89; la imagen es rust:1.85).
- [x] Verification: `renovate-config-validator` → Config validated successfully.
- [x] Descartado `[resolver] incompatible-rust-versions = "fallback"`: con MSRV 1.80
      degradaría media flota (clap, brotli, cfb, darling…). Probado en dry-run.
- [x] Anexo P9: toca `.github/**` → `policy-exception`, no auto-merge.

## Task 4: Desktop e2e `unexpected alert open` — HECHA (verificación en CI)

- [x] Causa: `window.confirm()` de `/ajustes` y `confirm()+alert()` del menú nativo
      (`apps/desktop/src/menu_es.rs`); WebKitGTK/tauri-driver no los auto-descarta.
- [x] Helper `settleNativeDialog()`: `dismissAlert()` → interpreta `no such alert` como
      «no había diálogo» → fallback a `acceptAlert()`. **No** usa `isAlertOpen()`
      (comando solo-Chromium, no existe en WebKitGTK).
- [x] Se invoca tras el click de borrado, en `gotoLoopback()` y antes de V-03.
- [x] Verification local: `node --check` OK.
- [x] **Verificado en CI** (run 44110799290): `Desktop e2e (tauri-driver)` 3m15s y
      `Desktop e2e (no network)` 1m05s en verde.

## Task 5: Fixture duplicado consolidado — HECHA

- [x] `examples/sample.vcf` pasa a **symlink relativo** al fixture canónico del crate
      (`crates/zedazo-core/tests/fixtures/sample-contacts.vcf`); se conserva el
      contenido en el crate (autocontenido para tests/publicación).
- [x] `examples/README.md` explica el enlace y sus consumidores.
- [x] Verification: sha256 idéntico (`14212ea1…`) y `cp` del symlink OK;
      `cargo test -p zedazo-core --test integration_test --all-features` 17 pass.

## Task 6: `.env.example` — HECHA

- [x] Plantilla con API/bind, datos y retención, auth, OTEL y CardDAV; valores por
      defecto contrastados con el código (24 h retención, 24 h upload TTL, 50 MiB,
      4 jobs concurrentes).
- [x] Sin defaults inventados: `ZEDAZO_JOB_TIMEOUT_SECS` comentado (sin límite por
      defecto); secretos (`AUTH_TOKEN`, `CARDDAV_PASSWORD`) comentados.
- [x] Enlazado desde `README.md`; `.env` sigue en `.gitignore`.

## Task 7: Deuda `chrono`/`rustix` registrada — HECHA

- [x] ADR-0022 (Opción A: MSRV 1.80 + pins, `lockFileMaintenance` de cargo off) con
      alternativas rechazadas y motivo.
- [x] ROADMAP: dependencias congeladas actualizadas y deuda registrada
      (`chrono → jiff`, revisión de `rustix` al publicarse parche).
- [x] Verification: `make docs-validate` OK.

## Task 8 (añadida tras la primera pasada de CI): rustix 1.1.6 + chrono → jiff — HECHA

La primera pasada de CI mostró que `Deny (cargo-deny)` y `sca` seguían en rojo por
avisos que el informe había clasificado como «no bloqueantes». Documentar no bastaba:

- `RUSTSEC-2026-0486` (**rustix**) sí tenía parche declarado (`>=1.1.6`). Subido
  1.1.4 → 1.1.6 (cadena `tempfile`).
- `RUSTSEC-2026-0543` (**chrono** unmaintained): `cargo deny` falla con
  `error[unmaintained]` cuando la crate es dependencia **directa** del workspace.
  `zedazo-api` migra de `chrono` a `jiff` (ya en el workspace):
  `Utc::now().to_rfc3339()` → `Timestamp::now().to_string()`; `parse_from_rfc3339` →
  `str::parse::<Timestamp>()`; `signed_duration_since().num_hours()` →
  `duration_since().as_secs()/3600`; `DateTime::from(SystemTime)` →
  `Timestamp::try_from(SystemTime)`.
- El `chrono` restante (0.4.45) es transitivo de Tauri (`serde_with` → `tauri-utils`):
  queda silenciado en `osv-scanner.toml` con motivo y fecha de revisión.

## Checkpoints

- [x] **A (tras Task 1)**: `cargo audit` limpio (0 vulnerabilidades).
- [x] **B (tras Tasks 2–3)**: `npm audit` limpio; `renovate-config-validator` OK;
      `Compose smoke` verde en CI (no ejecutable aquí: sin permisos de docker).
- [x] **C (final)**: `ci-gate`, `Security`, `Deny`, `sca`, `Desktop e2e` y
      `Compose smoke` verdes en CI (run 44110799290, 41 pass / 0 fail / 8 skipping).

## Verificación final (CI, run 44110799290)

**41 pass · 0 fail · 8 skipping. `ci-gate` verde.** Incluye lo no ejecutable en local:
`Compose smoke (Docker)`, `Desktop e2e (tauri-driver)`, `Desktop e2e (no network)`,
`Desktop AppImage`, `MSRV 1.80`, `Deny`, `sca`, `secrets`, `Parity CLI↔API`, `Web`
y `Coverage`.

## Nota de entorno

No ejecutables en esta máquina (verificado en CI):

| Job | Bloqueo local |
| --- | --- |
| Compose smoke (Docker) | `docker` → permission denied en `/var/run/docker.sock` |
| Desktop e2e (tauri-driver / no network) | falta `xvfb-run`, `tauri-driver`, AppImage |

Todo lo demás (fmt, clippy, check, test, doc, docs-validate, audit, deny,
osv-scanner, web typecheck/lint/unit/build/Playwright) se ejecutó localmente. El único
paso de `make ci` que no completa aquí es `playwright install --with-deps`, que pide
elevación de privilegios para instalar dependencias del sistema.
