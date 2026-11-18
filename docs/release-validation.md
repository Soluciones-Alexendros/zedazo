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
| `assets` | Los 12 ficheros esperados (5 plataformas + `.sha256`, `sbom.json` + `.sha256`), checksums, SBOM CycloneDX con el paquete `zedazo`, y que `zedazo --version` coincide con el tag. |
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

## Qué NO hace

- No crea ni publica releases (`gh release create` está prohibido en este repo).
- No publica en crates.io: eso es `publish-crates.yml`, despacho manual.
- No sustituye a `ci-gate`: la calidad por commit sigue en `ci.yml`.
