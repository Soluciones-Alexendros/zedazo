//! Errores del cliente CardDAV.

use std::time::Duration;

/// Resultado de operaciones CardDAV.
pub type CardDavResult<T> = Result<T, CardDavError>;

/// Error de configuración, transporte o protocolo CardDAV.
#[derive(Debug, thiserror::Error)]
pub enum CardDavError {
    /// Falta `ZEDAZO_CARDDAV_URL` / `[carddav].url`.
    #[error(
        "falta la URL del servidor CardDAV (ZEDAZO_CARDDAV_URL, --url o [carddav].url en TOML)"
    )]
    MissingUrl,

    /// Falta usuario Basic.
    #[error("falta el usuario CardDAV (ZEDAZO_CARDDAV_USERNAME, --username o [carddav].username)")]
    MissingUsername,

    /// Falta contraseña de aplicación. Nunca se reutiliza `ZEDAZO_AUTH_TOKEN`.
    #[error(
        "falta la contraseña de aplicación CardDAV (ZEDAZO_CARDDAV_PASSWORD o [carddav].password). \
         No uses ZEDAZO_AUTH_TOKEN: ese secreto es de la GUI (ADR-0016/0018)"
    )]
    MissingPassword,

    /// Esquema no soportado.
    #[error("esquema de URL no soportado: {0} (usa https, o http solo hacia loopback)")]
    UnsupportedScheme(String),

    /// HTTP claro hacia un host que no es loopback (ADR-0018).
    #[error(
        "HTTP sin TLS solo está permitido hacia loopback (127.0.0.1, ::1, localhost); \
         el servidor remoto exige HTTPS (TLS 1.2+)"
    )]
    CleartextForbidden,

    /// URL inválida.
    #[error("URL CardDAV inválida: {0}")]
    InvalidUrl(String),

    /// Error de configuración TOML.
    #[error("error de configuración CardDAV en {path}: {reason}")]
    Config { path: String, reason: String },

    /// Fallo de transporte HTTP.
    #[error("error HTTP CardDAV: {0}")]
    Transport(String),

    /// El servidor respondió un estado inesperado.
    #[error("el servidor CardDAV respondió HTTP {status} en {url}: {detail}")]
    HttpStatus {
        status: u16,
        url: String,
        detail: String,
    },

    /// El servidor DAV devolvió un href fuera del origen.
    #[error(
        "href CardDAV fuera de origen ({from} → {to}); se ignora para no reenviar credenciales"
    )]
    CrossOrigin { from: String, to: String },

    /// XML DAV inválido o incompleto.
    #[error("respuesta DAV inválida: {0}")]
    Protocol(String),

    /// Autenticación Basic rechazada.
    #[error("autenticación CardDAV rechazada (HTTP 401) en {0}")]
    Unauthorized(String),

    /// Hay varios addressbooks y no se indicó cuál.
    #[error("hay {count} addressbooks; indica uno con --addressbook / ZEDAZO_CARDDAV_ADDRESSBOOK")]
    AmbiguousAddressbook { count: usize },

    /// No se encontró ningún addressbook.
    #[error("no se encontró ningún addressbook CardDAV en {0}")]
    NoAddressbook(String),

    /// Error de E/S local (escritura del VCF).
    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),

    /// Se agotaron los reintentos (429/503).
    #[error("el servidor CardDAV sigue ocupado tras {attempts} intentos (último Retry-After {retry_after:?})")]
    RetriesExhausted {
        attempts: u32,
        retry_after: Option<Duration>,
    },

    /// PUT/DELETE exige `If-Match` con ETag concreto; nunca `*` (ADR-0018).
    #[error(
        "escritura CardDAV sin ETag: PUT/DELETE exige --etag / If-Match (no se permite \
         If-Match: * ni overwrite silencioso)"
    )]
    MissingEtag,

    /// HTTP 412: el ETag remoto cambió; conflicto reportado, no overwrite.
    #[error(
        "conflicto CardDAV (HTTP 412 Precondition Failed) en {url}: el ETag remoto \
         no coincide con If-Match {etag:?}; no se sobrescribió"
    )]
    PreconditionFailed { url: String, etag: String },

    /// Filtro `--category` que no parece código N1/N2 de la taxonomía.
    #[error(
        "categoría CardDAV desconocida o mal formada: {0} (usa códigos N1/N2 como PROF, PROF-JUD, FIN-CRYPTO)"
    )]
    UnknownCategory(String),
}

impl From<reqwest::Error> for CardDavError {
    fn from(err: reqwest::Error) -> Self {
        CardDavError::Transport(err.to_string())
    }
}

impl From<reqwest::header::InvalidHeaderValue> for CardDavError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        CardDavError::Transport(err.to_string())
    }
}
