//! Configuración CardDAV: env `ZEDAZO_CARDDAV_*` y sección `[carddav]` de TOML.
//!
//! Nunca se lee `ZEDAZO_AUTH_TOKEN` (token de GUI, ADR-0016).

use std::path::{Path, PathBuf};

use reqwest::Url;
use serde::Deserialize;

use crate::error::{CardDavError, CardDavResult};
use crate::url_policy::parse_and_validate;

/// Claves de entorno del proveedor CardDAV (ADR-0018).
pub const ENV_URL: &str = "ZEDAZO_CARDDAV_URL";
pub const ENV_USERNAME: &str = "ZEDAZO_CARDDAV_USERNAME";
pub const ENV_PASSWORD: &str = "ZEDAZO_CARDDAV_PASSWORD";
pub const ENV_ADDRESSBOOK: &str = "ZEDAZO_CARDDAV_ADDRESSBOOK";

/// Token de GUI: **no** es credencial CardDAV.
pub const GUI_AUTH_TOKEN_ENV: &str = "ZEDAZO_AUTH_TOKEN";

/// Overrides explícitos (CLI). Tienen prioridad sobre env y TOML.
#[derive(Debug, Clone, Default)]
pub struct ConfigOverrides {
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub addressbook: Option<String>,
    pub config_path: Option<PathBuf>,
}

/// Credenciales y URL de una sola cuenta CardDAV.
#[derive(Clone)]
pub struct CardDavConfig {
    pub url: Url,
    pub username: String,
    pub password: String,
    pub addressbook: Option<Url>,
}

impl std::fmt::Debug for CardDavConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CardDavConfig")
            .field("url", &self.url.as_str())
            .field("username", &self.username)
            .field("password", &"***")
            .field("addressbook", &self.addressbook.as_ref().map(Url::as_str))
            .finish()
    }
}

#[derive(Debug, Deserialize, Default)]
struct FileToml {
    #[serde(default)]
    carddav: Option<CarddavToml>,
}

#[derive(Debug, Deserialize, Default)]
struct CarddavToml {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    addressbook: Option<String>,
}

/// Carga configuración: CLI > env `ZEDAZO_CARDDAV_*` > `[carddav]` en TOML.
pub fn load(overrides: ConfigOverrides) -> CardDavResult<CardDavConfig> {
    load_with_env(overrides, |k| std::env::var(k).ok())
}

/// Igual que [`load`], con un lector de entorno inyectable (tests).
pub fn load_with_env(
    overrides: ConfigOverrides,
    env_get: impl Fn(&str) -> Option<String>,
) -> CardDavResult<CardDavConfig> {
    let file = match overrides.config_path.as_deref() {
        Some(path) => load_toml(path)?,
        None => CarddavToml::default(),
    };

    let env_nonempty = |key: &str| env_get(key).and_then(|s| nonempty(Some(s)));

    let url_raw = first_nonempty([overrides.url, env_nonempty(ENV_URL), file.url])
        .ok_or(CardDavError::MissingUrl)?;

    let username = first_nonempty([
        overrides.username,
        env_nonempty(ENV_USERNAME),
        file.username,
    ])
    .ok_or(CardDavError::MissingUsername)?;

    let password = pick_password(overrides.password, env_nonempty, file.password)?;

    let addressbook_raw = first_nonempty([
        overrides.addressbook,
        env_nonempty(ENV_ADDRESSBOOK),
        file.addressbook,
    ]);

    let url = parse_and_validate(&url_raw)?;
    let addressbook = match addressbook_raw {
        Some(raw) => Some(parse_and_validate(&raw)?),
        None => None,
    };

    Ok(CardDavConfig {
        url,
        username,
        password,
        addressbook,
    })
}

/// Elige la contraseña de aplicación. **No** consulta `ZEDAZO_AUTH_TOKEN`.
pub fn pick_password(
    override_pw: Option<String>,
    env_get: impl Fn(&str) -> Option<String>,
    file_pw: Option<String>,
) -> CardDavResult<String> {
    if let Some(p) = nonempty(override_pw) {
        return Ok(p);
    }
    if let Some(p) = env_get(ENV_PASSWORD).and_then(|s| nonempty(Some(s))) {
        return Ok(p);
    }
    // Intencionalmente no se llama env_get(GUI_AUTH_TOKEN_ENV).
    if let Some(p) = nonempty(file_pw) {
        return Ok(p);
    }
    Err(CardDavError::MissingPassword)
}

fn load_toml(path: &Path) -> CardDavResult<CarddavToml> {
    let content = std::fs::read_to_string(path)?;
    let parsed: FileToml = toml::from_str(&content).map_err(|e| CardDavError::Config {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    Ok(parsed.carddav.unwrap_or_default())
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}

fn first_nonempty<const N: usize>(candidates: [Option<String>; N]) -> Option<String> {
    candidates.into_iter().find_map(nonempty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn password_ignores_gui_auth_token() {
        let err = pick_password(
            None,
            |k| {
                if k == GUI_AUTH_TOKEN_ENV {
                    Some("token-de-gui-no-servir".into())
                } else {
                    None
                }
            },
            None,
        )
        .unwrap_err();
        assert!(matches!(err, CardDavError::MissingPassword));
    }

    #[test]
    fn password_prefers_carddav_env_over_file() {
        let pw = pick_password(
            None,
            |k| {
                if k == ENV_PASSWORD {
                    Some("app-pass".into())
                } else if k == GUI_AUTH_TOKEN_ENV {
                    Some("gui".into())
                } else {
                    None
                }
            },
            Some("from-file".into()),
        )
        .unwrap();
        assert_eq!(pw, "app-pass");
    }

    #[test]
    fn load_from_toml_and_redacts_debug() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("zedazo.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(
            b"[zedazo]\nprefijo_pais = \"+34\"\n\n[carddav]\n\
              url = \"https://cloud.example.test/remote.php/dav/\"\n\
              username = \"ada\"\n\
              password = \"super-secreto\"\n",
        )
        .unwrap();

        let cfg = load_with_env(
            ConfigOverrides {
                config_path: Some(path),
                ..ConfigOverrides::default()
            },
            |_| None,
        )
        .unwrap();
        assert_eq!(cfg.username, "ada");
        assert_eq!(cfg.password, "super-secreto");
        let dbg = format!("{cfg:?}");
        assert!(
            !dbg.contains("super-secreto"),
            "debug no debe filtrar el secreto: {dbg}"
        );
        assert!(dbg.contains("***"));
    }

    #[test]
    fn load_cli_overrides_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("zedazo.toml");
        std::fs::write(
            &path,
            "[carddav]\nurl = \"https://from-file.example.test/\"\nusername = \"file\"\npassword = \"file-pass\"\n",
        )
        .unwrap();

        let cfg = load_with_env(
            ConfigOverrides {
                url: Some("https://from-cli.example.test/dav/".into()),
                username: Some("cli".into()),
                password: Some("cli-pass".into()),
                config_path: Some(path),
                ..ConfigOverrides::default()
            },
            |_| None,
        )
        .unwrap();
        assert_eq!(cfg.url.as_str(), "https://from-cli.example.test/dav/");
        assert_eq!(cfg.username, "cli");
        assert_eq!(cfg.password, "cli-pass");
    }
}
