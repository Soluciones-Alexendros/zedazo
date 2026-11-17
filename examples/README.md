# Ejemplos

`sample.vcf` es un **enlace simbólico** al fixture canónico de tests:

```
examples/sample.vcf -> ../crates/zedazo-core/tests/fixtures/sample-contacts.vcf
```

El contenido es 100 % sintético (sin PII real) y se mantiene en un único sitio
para que el ejemplo de usuario y la suite de tests no divergan.

Consumidores: `README.md`, `deploy/scripts/compose-smoke.sh`,
`scripts/desktop/parity-cribar.sh` y el e2e de escritorio
(`ZEDAZO_SAMPLE_VCF` por defecto).
