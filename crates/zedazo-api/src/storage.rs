use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use sha2::{Digest, Sha256};

/// TTL por defecto para uploads sin trabajo asociado (horas).
pub const DEFAULT_UPLOAD_TTL_HOURS: u64 = 24;

#[derive(Clone)]
pub struct Storage {
    pub root: PathBuf,
}

impl Storage {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir_all(root.join("jobs"))?;
        fs::create_dir_all(root.join("tmp"))?;
        fs::create_dir_all(root.join("uploads"))?;
        Ok(Self { root })
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

    pub fn write_upload_stream(
        &self,
        upload_id: &str,
        mut reader: impl std::io::Read,
        max_bytes: usize,
    ) -> anyhow::Result<(PathBuf, u64, String)> {
        Self::validate_id(upload_id)?;
        let path = self.upload_path(upload_id);
        self.assert_under_root(&path)?;
        if path.exists() {
            bail!("upload_id ya existe");
        }
        let mut file = fs::File::create(&path)?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 64 * 1024];
        let mut total: u64 = 0;
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total as usize > max_bytes {
                drop(file);
                let _ = fs::remove_file(&path);
                bail!("upload excede límite");
            }
            hasher.update(&buf[..n]);
            use std::io::Write;
            file.write_all(&buf[..n])?;
        }
        let sha = hex::encode(hasher.finalize());
        // Sidecar con marca de tiempo para TTL de uploads huérfanos.
        let meta = serde_json::json!({
            "created_at": chrono::Utc::now().to_rfc3339(),
            "bytes": total,
            "sha256": sha,
        });
        let _ = Self::atomic_write(
            &self.upload_meta_path(upload_id),
            meta.to_string().as_bytes(),
        );
        Ok((path, total, sha))
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

    /// Escritura atómica: tmp en el mismo directorio + rename.
    pub fn atomic_write(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension(format!(
            "{}.tmp",
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("partial")
        ));
        // Preferir nombre único si .tmp colisiona
        let tmp = if tmp.exists() {
            path.with_extension(format!("tmp-{}", uuid::Uuid::now_v7()))
        } else {
            tmp
        };
        fs::write(&tmp, bytes).with_context(|| format!("escribir {}", tmp.display()))?;
        fs::rename(&tmp, path).with_context(|| format!("rename → {}", path.display()))?;
        Ok(())
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
