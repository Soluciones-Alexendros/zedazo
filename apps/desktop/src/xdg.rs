//! Rutas XDG (config / data / state) para el escritorio.

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;

/// Directorios de aplicación bajo el esquema XDG (Linux) / equivalentes.
#[derive(Debug, Clone)]
pub struct XdgPaths {
    pub config: PathBuf,
    pub data: PathBuf,
    pub state: PathBuf,
}

impl XdgPaths {
    /// Resuelve y crea config/data/state (`zedazo`).
    pub fn resolve() -> anyhow::Result<Self> {
        let dirs = ProjectDirs::from("dev", "alexendros", "zedazo")
            .ok_or_else(|| anyhow::anyhow!("no se pudieron resolver directorios XDG"))?;
        let paths = Self {
            config: dirs.config_dir().to_path_buf(),
            data: dirs.data_dir().to_path_buf(),
            state: dirs
                .state_dir()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| dirs.data_local_dir().join("state")),
        };
        paths.ensure()?;
        Ok(paths)
    }

    fn ensure(&self) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        for p in [&self.config, &self.data, &self.state] {
            zedazo_core::infrastructure::secure_fs::ensure_dir_0700(p)
                .map_err(|e| anyhow::anyhow!(e))?;
            if p.exists() {
                fs::set_permissions(p, fs::Permissions::from_mode(0o700))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_creates_dirs() {
        // En CI/XDG temporales ProjectDirs sigue siendo válido; solo comprobamos no panic.
        let x = XdgPaths::resolve().unwrap();
        assert!(x.config.exists());
        assert!(x.data.exists());
        assert!(x.state.exists());
        assert!(x.config.to_string_lossy().contains("zedazo"));
    }
}
