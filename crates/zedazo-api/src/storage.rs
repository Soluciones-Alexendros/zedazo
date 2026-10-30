use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

/// TTL por defecto para uploads sin trabajo asociado (horas).
pub const DEFAULT_UPLOAD_TTL_HOURS: u64 = 24;

#[derive(Clone)]
pub struct Storage {
    pub root: PathBuf,
}

impl Storage {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        use zedazo_core::infrastructure::secure_fs;
        for sub in ["jobs", "tmp", "uploads"] {
            secure_fs::ensure_dir_0700(&root.join(sub))?;
        }
        let s = Self { root };
        s.harden_tree_permissions()?;
        Ok(s)
    }

    /// Corrige modos del árbol de datos (dirs 0700, ficheros 0600) — S-06.
    pub fn harden_tree_permissions(&self) -> anyhow::Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fn walk(path: &Path) -> std::io::Result<()> {
                let meta = fs::symlink_metadata(path)?;
                if meta.file_type().is_symlink() {
                    return Ok(());
                }
                if meta.is_dir() {
                    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
                    for entry in fs::read_dir(path)? {
                        walk(&entry?.path())?;
                    }
                } else if meta.is_file() {
                    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
                }
                Ok(())
            }
            if self.root.exists() {
                walk(&self.root)?;
            }
        }
        Ok(())
    }

    /// Solo UUID (v4/v7 hex con guiones). Rechaza path traversal.
    pub fn validate_id(id: &str) -> anyhow::Result<()> {
        if id.is_empty() || id.len() > 64 {
            bail!("id inválido");
        }
        if id.contains("..") || id.contains('/') || id.contains('\\') {
            bail!("id con separadores de ruta");
        }
        let ok = id.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
        if !ok {
            bail!("id no es UUID");
        }
        // UUID canónico: 8-4-4-4-12
        let parts: Vec<_> = id.split('-').collect();
        if parts.len() != 5
            || parts[0].len() != 8
            || parts[1].len() != 4
            || parts[2].len() != 4
            || parts[3].len() != 4
            || parts[4].len() != 12
        {
            bail!("id no tiene forma UUID");
        }
        Ok(())
    }

    pub fn upload_path(&self, upload_id: &str) -> PathBuf {
        self.root.join("uploads").join(upload_id)
    }

    pub fn upload_meta_path(&self, upload_id: &str) -> PathBuf {
        self.root
            .join("uploads")
            .join(format!("{upload_id}.meta.json"))
    }

    pub fn job_dir(&self, job_id: &str) -> PathBuf {
        self.root.join("jobs").join(job_id)
    }

    pub fn pending_deletes_path(&self) -> PathBuf {
        self.root.join("pending_deletes.json")
    }

    pub fn wipe_epoch_path(&self) -> PathBuf {
        self.root.join("wipe_epoch")
    }

    pub fn ensure_job_dir(&self, job_id: &str) -> anyhow::Result<PathBuf> {
        Self::validate_id(job_id)?;
        let d = self.job_dir(job_id);
        self.assert_under_root(&d)?;
        fs::create_dir_all(&d)?;
        Ok(d)
    }

    pub fn delete_job_dir(&self, job_id: &str) -> anyhow::Result<()> {
        Self::validate_id(job_id)?;
        let d = self.job_dir(job_id);
        self.assert_under_root(&d)?;
        if d.exists() {
            fs::remove_dir_all(&d).with_context(|| format!("borrar {}", d.display()))?;
        }
        Ok(())
    }

    pub fn delete_upload(&self, upload_id: &str) -> anyhow::Result<()> {
        Self::validate_id(upload_id)?;
        let path = self.upload_path(upload_id);
        self.assert_under_root(&path)?;
        if path.exists() {
            fs::remove_file(&path).with_context(|| format!("borrar upload {}", path.display()))?;
        }
        let meta = self.upload_meta_path(upload_id);
        if meta.exists() {
            let _ = fs::remove_file(&meta);
        }
        Ok(())
    }

    /// Borra todos los jobs, uploads y tmp bajo el data dir.
    pub fn wipe_all(&self) -> anyhow::Result<()> {
        for sub in ["jobs", "uploads", "tmp"] {
            let p = self.root.join(sub);
            if p.exists() {
                fs::remove_dir_all(&p).with_context(|| format!("wipe {}", p.display()))?;
            }
            fs::create_dir_all(&p)?;
        }
        let _ = fs::remove_file(self.pending_deletes_path());
        Ok(())
    }

    pub fn read_wipe_epoch(&self) -> u64 {
        fs::read_to_string(self.wipe_epoch_path())
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    pub fn bump_wipe_epoch(&self) -> anyhow::Result<u64> {
        let next = self.read_wipe_epoch().saturating_add(1);
        Self::atomic_write(&self.wipe_epoch_path(), next.to_string().as_bytes())?;
        Ok(next)
    }

    pub fn load_pending_deletes(&self) -> Vec<PendingDelete> {
        let path = self.pending_deletes_path();
        let Ok(raw) = fs::read_to_string(&path) else {
            return Vec::new();
        };
        serde_json::from_str(&raw).unwrap_or_default()
    }

    pub fn save_pending_deletes(&self, items: &[PendingDelete]) -> anyhow::Result<()> {
        let raw = serde_json::to_string_pretty(items)?;
        Self::atomic_write(&self.pending_deletes_path(), raw.as_bytes())
    }

    pub fn enqueue_pending_delete(&self, kind: PendingKind, id: &str) -> anyhow::Result<()> {
        let mut items = self.load_pending_deletes();
        if !items.iter().any(|p| p.kind == kind && p.id == id) {
            items.push(PendingDelete {
                kind,
                id: id.to_string(),
                enqueued_at: chrono::Utc::now().to_rfc3339(),
            });
            self.save_pending_deletes(&items)?;
        }
        Ok(())
    }

    /// Escritura atómica vía `secure_fs` (0600 / persist).
    pub fn atomic_write(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
        use zedazo_core::infrastructure::secure_fs::{self, WriteOpts};
        secure_fs::write_bytes(path, bytes, WriteOpts::new(true))
            .map_err(|e| anyhow::anyhow!(e))
            .with_context(|| format!("escribir {}", path.display()))
    }

    pub fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .take(200)
            .collect()
    }

    pub fn assert_under_root(&self, path: &Path) -> anyhow::Result<()> {
        let canon_root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        // Si el path aún no existe, comprobar el padre canónico + nombre.
        let canon = if path.exists() {
            path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
        } else if let Some(parent) = path.parent() {
            let parent_c = parent
                .canonicalize()
                .unwrap_or_else(|_| parent.to_path_buf());
            parent_c.join(path.file_name().unwrap_or_default())
        } else {
            path.to_path_buf()
        };
        if !canon.starts_with(&canon_root) {
            bail!("ruta fuera del data dir");
        }
        Ok(())
    }

    pub fn list_job_dirs(&self) -> anyhow::Result<Vec<PathBuf>> {
        let jobs = self.root.join("jobs");
        let mut out = Vec::new();
        if !jobs.exists() {
            return Ok(out);
        }
        for entry in fs::read_dir(&jobs)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                out.push(entry.path());
            }
        }
        Ok(out)
    }

    pub fn list_upload_ids(&self) -> anyhow::Result<Vec<String>> {
        let uploads = self.root.join("uploads");
        let mut out = Vec::new();
        if !uploads.exists() {
            return Ok(out);
        }
        for entry in fs::read_dir(&uploads)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".meta.json") {
                    continue;
                }
                out.push(name);
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingKind {
    Job,
    Upload,
    JobDir,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingDelete {
    pub kind: PendingKind,
    pub id: String,
    pub enqueued_at: String,
}
