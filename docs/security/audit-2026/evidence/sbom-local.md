# SBOM local — auditoría 2026

**Fecha:** 2026-11-18  
**Repositorio:** zedazo (`/home/alexendros/Work/zedazo`)

## Comando canónico (CI)

Equivalente a [`.github/workflows/sbom-crates.yml`](../../../.github/workflows/sbom-crates.yml):

```bash
cargo cyclonedx --manifest-path crates/zedazo-cli/Cargo.toml --format json --override-filename sbom-crates
```

En CI se instala previamente: `cargo install cargo-cyclonedx --locked`.

## Ejecución local

| Campo | Valor |
| --- | --- |
| **Estado** | `not_run` |
| **Motivo** | `cargo cyclonedx` no disponible en este host: `command -v cargo-cyclonedx` sin resultado y `cargo cyclonedx --help` devuelve `error: no such command: cyclonedx`. |
| **Tamaño del SBOM** | — |
| **JSON válido** | — |

No se generaron artefactos en el repositorio.