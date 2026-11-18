# Validación de release

**Traza:** [ADR-0012](./../DECISIONS.md) (distribución), [ADR-0021](./../DECISIONS.md)
(escritorio), [SPECS.md](./../SPECS.md) O10. Workflow:
[`.github/workflows/release-validation.yml`](../.github/workflows/release-validation.yml).

## Para qué

Cada release publicada se valida sola antes de que nadie la use: que los
artefactos existen, que sus checksums cuadran, que el SBOM es válido, que el
binario dice la versión del tag y que CLI y API siguen dando el mismo resultado.

No publica nada. El draft lo crea `release-please` y los assets los sube
`release.yml`. Este workflow solo verifica y falla si algo no cuadra.

## Cadena completa

```
commit en main
  └─ release-please.yml ─ abre PR de release (bump + CHANGELOG)
       └─ merge del PR ─ crea tag vX.Y.Z + draft de release
            └─ release-please.yml (job dispatch-release-build)
                 ├─ gh workflow run release.yml --ref vX.Y.Z
                 │    └─ cargo-dist construye 10 artefactos + sbom → draft
                 └─ gh workflow run release-validation.yml --ref vX.Y.Z
                      └─ espera los assets y los valida
publicación manual del draft
  ├─ publish.yml ─ build AppImage + atestación + upload al release
  └─ release-validation.yml (release: published) ─ revalida, ya con AppImage
```

## Por qué hace falta el despacho explícito

`release-please` crea el tag con `GITHUB_TOKEN`, y **GitHub no dispara workflows
para eventos generados por ese token** (evita recursión). Un `push: tags` normal
nunca ocurre para esos tags.

Consecuencia real: los drafts de **v0.6.1 y v0.6.2** quedaron con 0 assets porque
`release.yml` no llegaba a ejecutarse. Desde v0.6.3 lo cubre el job
`dispatch-release-build` de `release-please.yml`.

## Qué valida

| Job | Comprueba |
| --- | --- |
| `resolve` | Que el evento trae un tag con forma `vX.Y.Z`. |
| `assets` | Bloqueantes: al menos un binario y al menos un SBOM (`sbom.json` o `sbom-*.json`), CycloneDX válido y checksums de todo lo que traiga `.sha256`/`SHA256SUMS`. Avisos: los tarballs de cargo-dist. |
| `appimage` | Checksum contra `.sha256` o `SHA256SUMS`, extracción squashfs, `AppRun` ejecutable y `zedazo-desktop` ELF no vacío. |
| `parity` | `cargo test -p zedazo-api --test equivalence_http` sobre el código del tag (O10). |
| `gate` | Resume y falla si algún job crítico no terminó en `success`. |

El AppImage **no se ejecuta**: es una app GTK, no termina con `--version` y
colgaría el job. Su smoke de ejecución real ya lo cubre `Desktop smoke (xvfb)`
en cada push de `ci.yml`; aquí solo se verifica que el artefacto está íntegro y
completo.

## Cómo relanzarla

```bash
# Revalidar una release concreta (p. ej. tras subir un asset a mano)
gh workflow run release-validation.yml -f tag=v0.6.3

# Ver el resultado
gh run list --workflow=release-validation.yml --limit 3
```

## Estado de cargo-dist (aviso conocido)

Los tarballs multi-plataforma **no se generan** desde v0.6.1:

- **Linux:** el build de `dist` compila el workspace e incluye `apps/desktop`
  (Tauri/GTK) → falla con `Package gdk-3.0 was not found`.
- **Windows:** el workflow generado por dist 0.32 usa `eval` con `shell: pwsh`
  → `The term 'eval' is not recognized`.
- **macOS:** compila.

Mientras eso se arregla en su propio cambio, la validación los trata como
**aviso**: exige al menos un binario y un SBOM (hoy el AppImage y los
`sbom-*.json` de `publish.yml`), y publica en el log la lista exacta de lo que
falta. Una release sin binarios o sin SBOM sí falla.

El smoke del binario CLI solo corre si existe el tarball Linux; si la release
solo trae AppImage, ese job termina con un `notice` y la validación del
AppImage la hace el job `appimage`.

## Qué NO hace

- No crea ni publica releases (`gh release create` está prohibido en este repo).
- No publica en crates.io: eso es `publish-crates.yml`, despacho manual.
- No sustituye a `ci-gate`: la calidad por commit sigue en `ci.yml`.
