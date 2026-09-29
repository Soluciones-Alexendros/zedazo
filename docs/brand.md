# Marca zedazo

**Traza:** ADR-0014, [#49](https://github.com/Iniciativas-Alexendros/zedazo/issues/49) (resto de #35).  
**Fecha de esta nota:** 2026-09-11.

Esta nota cubre el **wordmark** y la **pesquisa documental** de marca en la UE. No es un dictamen jurídico ni una solicitud de registro.

## Wordmark (patrón Atlaps)

El signo visual del producto es **`zedazo` en minúsculas**. No se usa Title Case ni versales en lockups.

| Superficie | Forma | Ejemplo |
|------------|--------|---------|
| Wordmark / lockup / pestaña / H1 de README / `og:title` | `zedazo` | landing, GUI chrome |
| Prosa (oraciones en docs y UI) | Zedazo (nombre propio) | «Zedazo criba contactos VCF.» |
| Identificadores de código | `zedazo`, `ZEDAZO_*`, `X-ZEDAZO-*` | crate, env, props VCF (ADR-0014) |
| Nombre común español | zedazo | «pasada por el zedazo fino» (cedazo) |

El logomark (tres nodos) es decorativo junto al wordmark (`aria-hidden`); el nombre accesible del lockup es el texto `zedazo`.

Los colores de producción de la GUI se expresan en **OKLCH**. Fuente DTCG: [`apps/web/tokens/`](../apps/web/tokens/) (ADR-0019); custom properties `--zed-*` generadas. Esta nota no define la paleta; el pipeline y el QA visual están en [`docs/gui/design-system-plan.md`](./gui/design-system-plan.md) (ejecutado, [#59](https://github.com/Iniciativas-Alexendros/zedazo/issues/59)). Catálogo in-app: `/documentacion/ds`.

## Pesquisa TMview / EUIPO (clases 9 y 42)

**Alcance de esta pesquisa:** documental, desde índices públicos, el 2026-09-11.  
**No sustituye** una búsqueda oficial en [TMview](https://www.tmdn.org/tmview/welcome) ni un informe de un agente de propiedad industrial.

### Consultas

- Denominación exacta: `Zedazo`, `zedazo`, `ZEDAZO`
- Clases de Niza previstas: **9** (software descargable / CLI) y **42** (servicios de software / diseño; la GUI self-hosted no es un SaaS público, pero 42 sigue siendo la clase habitual de servicios informáticos)
- Oficinas de interés: EUIPO (EM) + oficinas nacionales UE, en especial OEPM (ES)

### Fuentes consultadas

| Fuente | Resultado |
|--------|-----------|
| TMview (`tmdn.org`) | Interfaz/API no alcanzable desde este entorno (TLS timeout). **Pendiente humano.** |
| eSearch plus / API EUIPO | API oficial exige OAuth en el portal de desarrolladores. Sin coincidencia exacta «Zedazo» en índices web públicos (TrademarkElite, búsquedas «EUTM»/«application number»). |
| crates.io | `zedazo` es este proyecto. |
| WIPO Global Brand Database | Portal con captcha; no se pudo listar registros. Un identificador indexado (`DE500002023118335`) **no se verificó** como «Zedazo» (página sin el verbal element). **Pendiente humano.** |
| OEPM Localizador / DPMAregister | Sin ficha pública indexada de la denominación exacta «Zedazo». Los localizadores gratuitos no sustituyen una búsqueda de similitud por clase. |

### Coincidencias exactas

No se encontró una marca comunitaria (EUTM) ni una ficha USPTO/TrademarkElite con verbal element exacto **Zedazo** en clases 9 o 42.

Eso **no** prueba disponibilidad: TMview cubre oficinas nacionales que no aparecen en eSearch plus, y el riesgo real es la **similitud**, no solo la identidad.

### Vecinos a revisar (no bloquean el wordmark en código)

| Signo | Oficina / nº | Clases | Estado (fuente secundaria) | Nota |
|-------|----------------|--------|----------------------------|------|
| **ZEZARO** | EUTM 009317348 | 9, 38, 41, 42, 45 | Registered (EUIPO; titular eVOX Solutions GmbH; renovada 2020) | Vecino visual/fonético (Z-E-**Z**-A-**R**-O vs Z-E-**D**-A-**Z**-O) en las mismas clases 9 y 42. Prioridad alta para el humano. |
| **CDAZO** | EUTM 014328579 | 5 | Registration expired | Farmacia; poco solape con 9/42. |
| **zezzo** | EUTM 018411256 | 8 | Registered | Herramientas de mano; poco solape. |
| **cedazo** | — | — | Nombre común ES; descartado como marca de producto (ADR-0014) | Distintivo débil si se reclamara como denominación descriptiva. |

### Checklist residual (humano)

1. Abrir [TMview](https://www.tmdn.org/tmview/welcome): búsqueda **exacta**, **fuzzy** y **fonética** de `Zedazo` / `Cedazo` / `Zezaro`, oficinas **EM + todas las UE**, clases **9 y 42**, estados vivos (solicitada/registrada).
2. Confirmar **ZEZARO** 009317348 en [eSearch plus](https://euipo.europa.eu/eSearch/) (estado, productos/servicios literales, oposiciones) y valorar riesgo de confusión.
3. OEPM: Localizador +, si se va a solicitar, búsqueda de antecedentes de pago (similitud por clase).
4. Decidir vía: **no registrar** (uso de hecho + crates.io) / **OEPM nacional** / **EUTM** clases 9 y 42 (términos TMclass, no headings genéricos).
5. Si se presenta solicitud: agente/abogado de PI; anotar número de expediente en ADR-0014.
6. Esta pesquisa **no** es un clearance. Un índice vacío no autoriza a usar el signo frente a terceros.

El wordmark lowercase y esta nota cierran la parte de **código/docs** de #49. El registro oficial, si se desea, queda fuera del repo.
