//! Cliente CardDAV para Zedazo (RFC 6352).
//!
//! Adaptador de infraestructura (`publish = false`) según [ADR-0018](https://github.com/Iniciativas-Alexendros/zedazo/blob/main/DECISIONS.md).
//! Lo consume `zedazo-cli`. **`zedazo-core` no tiene HTTP.**
//!
//! # Alcance de este crate
//!
//! - Descubrimiento `/.well-known/carddav` (RFC 6764) y URL de addressbook explícita.
//! - Perfil Nextcloud `/remote.php/dav/` como fallback documentado.
//! - Auth HTTP Basic + contraseña de aplicación (`ZEDAZO_CARDDAV_*`).
//! - Pull (PROPFIND + GET), write opt-in (PUT/DELETE + `If-Match`), watch (CTag / sync-token)
//!   y filtro client-side por categorías N1/N2.
//!
//! TLS 1.2+ con rustls; sin `insecure-skip-verify`. HTTP claro solo a loopback.

pub mod client;
pub mod config;
pub mod error;
pub mod filter;
pub mod url_policy;
pub mod xml;

pub use client::{
    Addressbook, CardDavClient, CardObject, Discovery, PullResult, SyncState, WriteResult,
};
pub use config::{
    load, CardDavConfig, ConfigOverrides, ENV_ADDRESSBOOK, ENV_PASSWORD, ENV_URL, ENV_USERNAME,
};
pub use error::{CardDavError, CardDavResult};
pub use reqwest::Url;
pub use url_policy::{parse_and_validate, validate_server_url};
