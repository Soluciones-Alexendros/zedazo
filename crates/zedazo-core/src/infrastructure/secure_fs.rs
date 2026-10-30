//! Escritura segura: I7 (same-file), sin symlinks, atómica, modos 0600/0700 (S-02/S-06).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;

use crate::error::CribaError;

/// Opciones de escritura segura.
#[derive(Debug, Clone, Copy)]
pub struct WriteOpts<'a> {
    /// Si es false, no sobrescribe un fichero existente.
    pub force: bool,
    /// Si está definido, rechaza escribir si la salida es el mismo inodo/dispositivo.
    pub protect_input: Option<&'a Path>,
}

impl<'a> WriteOpts<'a> {
    pub fn new(force: bool) -> Self {
        Self {
            force,
            protect_input: None,
        }
    }

    pub fn protect(mut self, input: &'a Path) -> Self {
        self.protect_input = Some(input);
        self
    }
}

/// Crea un directorio (y padres) con modo 0700 en Unix.
pub fn ensure_dir_0700(path: &Path) -> Result<(), CribaError> {
    if path.as_os_str().is_empty() || path == Path::new(".") {
        return Ok(());
    }
    if path.exists() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)?;
    }
    #[cfg(not(unix))]
    {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Rechaza si `path` es un symlink.
pub fn reject_symlink(path: &Path) -> Result<(), CribaError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Err(CribaError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("ruta es un symlink: {}", path.display()),
        ))),
        Ok(_) | Err(_) => Ok(()),
    }
}

/// True si ambas rutas existen y son el mismo fichero (dispositivo + inodo).
pub fn paths_same_file(a: &Path, b: &Path) -> Result<bool, CribaError> {
    if !a.exists() || !b.exists() {
        return Ok(false);
    }
    Ok(same_file::is_same_file(a, b)?)
}

/// Compara rutas canónicas cuando el destino aún no existe.
fn resolved_targets_equal(input: &Path, output: &Path) -> Result<bool, CribaError> {
    let in_canon = fs::canonicalize(input)?;
    if output.exists() {
        let out_canon = fs::canonicalize(output)?;
        return Ok(in_canon == out_canon);
    }
    // Destino ausente: resolver padre + nombre.
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let parent_canon = if parent.as_os_str().is_empty() {
        fs::canonicalize(Path::new("."))?
    } else if parent.exists() {
        fs::canonicalize(parent)?
    } else {
        // Padre inexistente → no puede ser el mismo fichero que input.
        return Ok(false);
    };
    let name = match output.file_name() {
        Some(n) => n,
        None => return Ok(false),
    };
    Ok(parent_canon.join(name) == in_canon)
}

fn guard_output(path: &Path, opts: WriteOpts<'_>) -> Result<(), CribaError> {
    reject_symlink(path)?;
    if let Some(input) = opts.protect_input {
        reject_symlink(input)?;
        if paths_same_file(input, path)? || resolved_targets_equal(input, path)? {
            return Err(CribaError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "I7: la ruta de salida coincide con la de entrada ({} → {})",
                    input.display(),
                    path.display()
                ),
            )));
        }
    }
    if path.exists() && !opts.force {
        return Err(CribaError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("no se sobrescribe sin --force: {}", path.display()),
        )));
    }
    Ok(())
}

/// Escribe `data` de forma atómica en `path` (NamedTempFile + persist), modo 0600.
pub fn write_bytes(path: &Path, data: &[u8], opts: WriteOpts<'_>) -> Result<(), CribaError> {
    guard_output(path, opts)?;

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    ensure_dir_0700(parent)?;

    let mut tmp = NamedTempFile::new_in(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(tmp.path(), perms)?;
    }
    tmp.write_all(data)?;
    tmp.flush()?;
    tmp.persist(path).map_err(|e| {
        CribaError::Io(std::io::Error::other(format!(
            "persist {}: {}",
            path.display(),
            e.error
        )))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// Copia bytes de `src` a `dst` con las mismas garantías (API / resultados).
pub fn copy_file(src: &Path, dst: &Path, opts: WriteOpts<'_>) -> Result<(), CribaError> {
    let data = fs::read(src)?;
    write_bytes(dst, &data, opts)
}

/// Contexto de parseo sin valor: nombre de propiedad (≤32 ASCII) y longitud.
pub fn redact_parse_context(line: &str) -> String {
    let header = line.split_once(':').map(|(h, _)| h).unwrap_or(line);
    let name = header.split(';').next().unwrap_or(header).trim();
    let name_ascii: String = name.chars().filter(|c| c.is_ascii()).take(32).collect();
    let value_len = line
        .find(':')
        .map(|i| line.len().saturating_sub(i + 1))
        .unwrap_or(0);
    format!("prop={name_ascii} value_len={value_len}")
}

/// Ruta de cuarentena derivada (misma lógica que writer).
pub fn quarantine_sibling(path: &Path) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    path.with_file_name(format!("{stem}_cuarentena{ext}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn redact_omits_tel_digits() {
        let ctx = redact_parse_context("TEL;X-+34600000000");
        assert!(!ctx.contains("34600000000"));
        assert!(ctx.contains("prop=TEL"));
        assert!(ctx.contains("value_len=0"));
    }

    #[test]
    fn redact_value_len_only() {
        let ctx = redact_parse_context("TEL:+34600000000");
        assert!(!ctx.contains("34600000000"));
        assert!(ctx.contains("value_len=12"));
    }

    #[test]
    fn write_bytes_mode_0600() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.vcf");
        write_bytes(path.as_path(), b"hello", WriteOpts::new(false)).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0);
    }

    #[test]
    fn reject_same_as_input() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.vcf");
        fs::write(&input, b"x").unwrap();
        let err = write_bytes(&input, b"y", WriteOpts::new(true).protect(&input)).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("I7"), "{msg}");
        assert_eq!(fs::read(&input).unwrap(), b"x");
    }
}
