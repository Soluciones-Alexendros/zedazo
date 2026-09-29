# CardDAV

**Traza:** [ADR-0018](../DECISIONS.md) (aceptada) · issue [#48](https://github.com/Iniciativas-Alexendros/zedazo/issues/48)

Cliente RFC 6352 en el crate `zedazo-carddav` (`publish = false`), consumido por la CLI. `zedazo-core` no tiene HTTP. La API/GUI **no** exponen CardDAV.

Una cuenta por configuración. Auth HTTP Basic + contraseña de aplicación (`ZEDAZO_CARDDAV_*`). **Nunca** se reutiliza `ZEDAZO_AUTH_TOKEN`.

## Comandos

| Comando | Efecto |
|---------|--------|
| `zedazo carddav list` | Descubre addressbooks |
| `zedazo carddav pull -o dest.vcf` | GET de vCards → VCF local (I7: destino distinto del origen remoto) |
| `zedazo carddav pull -o dest.vcf --category PROF` | Igual, filtrando N1/N2 en cliente |
| `zedazo carddav put --href … --input src.vcf --etag ETAG --confirm` | PUT con `If-Match` |
| `zedazo carddav put --href … --input src.vcf --create --confirm` | Alta con `If-None-Match: *` |
| `zedazo carddav delete --href … --etag ETAG --confirm` | DELETE con `If-Match` |
| `zedazo carddav watch [--interval 30] [--once] [-o dest.vcf]` | Polling de CTag / `sync-token` |

`cribar` no habla CardDAV. La escritura remota es **opt-in** por invocación (`put`/`delete` + `--confirm`).

## Autenticación

| Fuente | Claves |
|--------|--------|
| Entorno | `ZEDAZO_CARDDAV_URL`, `ZEDAZO_CARDDAV_USERNAME`, `ZEDAZO_CARDDAV_PASSWORD`, `ZEDAZO_CARDDAV_ADDRESSBOOK` (opcional) |
| TOML | sección `[carddav]` en `zedazo.toml` (`url`, `username`, `password`, `addressbook`) |
| CLI | `--url`, `--username`, `--addressbook`, `-c` / `--config` |

Prioridad: flags CLI > env > TOML. Preferir la contraseña en env, no en el fichero.

```toml
[carddav]
url = "https://cloud.example.test"
username = "ada"
# password = "…"  # mejor ZEDAZO_CARDDAV_PASSWORD
# addressbook = "https://cloud.example.test/remote.php/dav/addressbooks/users/ada/contacts/"
```

```bash
export ZEDAZO_CARDDAV_URL=https://cloud.example.test
export ZEDAZO_CARDDAV_USERNAME=ada
export ZEDAZO_CARDDAV_PASSWORD='contraseña-de-aplicación'
zedazo carddav list
zedazo carddav pull -o contactos.vcf
zedazo cribar contactos.vcf -o limpio.vcf
```

## Descubrimiento

1. **URL de addressbook explícita** (`--addressbook` / `ZEDAZO_CARDDAV_ADDRESSBOOK` / `[carddav].addressbook`): se usa esa colección; no hace falta `/.well-known/carddav`.
2. **RFC 6764** `/.well-known/carddav` desde el origen de `ZEDAZO_CARDDAV_URL`, siguiendo redirecciones (p. ej. iCloud a `pNN-contacts.icloud.com`).
3. **Nextcloud / SabreDAV:** si well-known no responde, se prueba `{origen}/remote.php/dav/`.
4. Si la URL ya apunta a un path DAV (`/remote.php/dav/`, home-set, etc.), se usa como raíz.

Después: `current-user-principal` → `addressbook-home-set` → listado Depth 1 de addressbooks → GET de cada vCard (**un request en vuelo**). 429/503 honran `Retry-After` (segundos) con backoff.

## Write opt-in (PUT/DELETE)

- Exige `--confirm`. Sin el flag no hay HTTP de mutación.
- Actualización: `If-Match` con ETag concreto (`--etag`). **No** se envía `If-Match: *`.
- Alta: `--create` envía `If-None-Match: *` (falla si el href ya existe).
- HTTP **412** → conflicto reportado (`CardDavError::PreconditionFailed`); **nunca** overwrite silencioso.
- I7: el VCF local de `--input` es de solo lectura; la salida de `pull`/`watch -o` va a otra ruta.

```bash
zedazo carddav put --href https://cloud.example.test/remote.php/dav/addressbooks/users/ada/contacts/ada.vcf \
  --input ada.vcf --etag 'etag-1' --confirm
zedazo carddav delete --href …/ada.vcf --etag 'etag-1' --confirm
```

## Watch mode

`zedazo carddav watch` **no** se arranca al iniciar la CLI ni la GUI. Polling conservador (un PROPFIND de colección cada `--interval` segundos; por defecto 30).

Huella: `DAV:sync-token` si el servidor lo anuncia; si no, `CS:getctag`; si no, ETag de la colección. El primer sondeo es línea base (no se trata como cambio). Si hay `-o` y el token cambia, se hace pull (secuencial) al VCF indicado.

`--once` = línea base + un sondeo. `--interval 0` evita espera (útil en tests). Se sigue honrando `Retry-After` / backoff.

No hay watch de ficheros locales (evitaría una dep nueva `notify`).

## Filtros por categoría

`--category` (repetible, OR) aplica la taxonomía N1/N2 **ya existente** sobre el set sincronizado, en cliente: se leen las propiedades `CATEGORIES` del vCard.

- `--category PROF` incluye `PROF` y N2 hijas (`PROF-JUD`, …).
- `--category PROF-JUD` solo ese N2 (o N3 debajo).
- No se usa `addressbook-query` RFC 6352 (no es requisito del MVP).

Los vCards remotos sin `CATEGORIES` no pasan el filtro. Tras `cribar`, el VCF local sí lleva N1/N2; un `put` posterior las deja en el servidor.

## TLS

HTTPS obligatorio hacia hosts no-loopback (TLS 1.2+, rustls). Sin `insecure-skip-verify`. HTTP claro solo hacia `127.0.0.1`, `::1` o `localhost` (puente local). CA privada / skip-verify: PR posterior.

## Fuera de alcance (siguen fuera de #48)

- OAuth Google / People API
- Proveedor Proton de primera clase (sigue export VCF)
- Endpoints CardDAV en `zedazo-api` / GUI
- Sync automático al arrancar CLI o GUI
- `addressbook-query` server-side
