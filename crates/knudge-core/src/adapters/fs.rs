//! Sistema de arquivos real, com escrita atômica (D20) e sem seguir symlink (R05).

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::ports::Fs;
use crate::{Error, Result};

/// Implementação de [`Fs`] sobre `std::fs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdFs;

impl StdFs {
    /// Cria o adaptador.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Rejeita symlink no caminho (R05): nada em `.knudge/` deve escapar do projeto.
    fn reject_symlink(path: &Path) -> Result<()> {
        match path.symlink_metadata() {
            Ok(meta) if meta.file_type().is_symlink() => Err(Error::UnsafeBlocked(format!(
                "symlink não permitido em .knudge/: {}",
                path.display()
            ))),
            _ => Ok(()),
        }
    }

    /// Nome do staging `*.tmp` ao lado do destino (mesmo diretório — D20).
    fn staging_path(path: &Path) -> Result<PathBuf> {
        let file_name = path
            .file_name()
            .ok_or_else(|| Error::invalid_input(format!("caminho sem nome: {}", path.display())))?;
        let mut staging_name = file_name.to_os_string();
        staging_name.push(".tmp");
        Ok(path.with_file_name(staging_name))
    }
}

impl Fs for StdFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        Self::reject_symlink(path)?;
        std::fs::read(path).map_err(|e| match e.kind() {
            // Ausência e diretório equivalem a "sem conteúdo legível" (D86): o verify-on-hit
            // das âncoras degrada para `Missing` em vez de derrubar o comando.
            std::io::ErrorKind::NotFound | std::io::ErrorKind::IsADirectory => {
                Error::not_found(path.display().to_string())
            }
            _ => Error::io(path, e),
        })
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        Self::reject_symlink(path)?;
        let staging = Self::staging_path(path)?;
        Self::reject_symlink(&staging)?;
        std::fs::write(&staging, data).map_err(|e| Error::io(&staging, e))?;
        std::fs::rename(&staging, path).map_err(|e| Error::io(path, e))
    }

    fn append(&self, path: &Path, data: &[u8]) -> Result<()> {
        Self::reject_symlink(path)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| Error::io(path, e))?;
        file.write_all(data).map_err(|e| Error::io(path, e))
    }

    fn create_exclusive(&self, path: &Path, data: &[u8]) -> Result<()> {
        Self::reject_symlink(path)?;
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(mut file) => file.write_all(data).map_err(|e| Error::io(path, e)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(Error::conflict(format!("já existe: {}", path.display())))
            }
            Err(e) => Err(Error::io(path, e)),
        }
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        std::fs::rename(from, to).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Error::not_found(format!(
                "rename {} -> {}: origem ausente",
                from.display(),
                to.display()
            )),
            _ => Error::io(from, e),
        })
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = std::fs::read_dir(path).map_err(|e| Error::io(path, e))?;
        let mut out = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| Error::io(path, e))?;
            out.push(entry.path());
        }
        out.sort();
        Ok(out)
    }

    fn exists(&self, path: &Path) -> bool {
        // `symlink_metadata` não segue o link: um symlink em `.knudge/` conta como presente,
        // mas a leitura/escrita real deve rejeitá-lo (R05).
        path.symlink_metadata().is_ok()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path).map_err(|e| Error::io(path, e))
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Error::io(path, e)),
        }
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        match std::fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Error::io(path, e)),
        }
    }

    fn sync(&self, path: &Path) -> Result<()> {
        let file = std::fs::File::open(path).map_err(|e| Error::io(path, e))?;
        file.sync_all().map_err(|e| Error::io(path, e))
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador de FS real; lê mtime do arquivo (a porta é a única a tocar o SO)"
    )]
    fn modified_ms(&self, path: &Path) -> Result<Option<i64>> {
        let meta = path.symlink_metadata().map_err(|e| Error::io(path, e))?;
        let Ok(modified) = meta.modified() else {
            return Ok(None);
        };
        let Ok(elapsed) = modified.duration_since(std::time::UNIX_EPOCH) else {
            return Ok(None);
        };
        Ok(Some(i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn rejects_symlink() -> Result<()> {
        use std::os::unix::fs::symlink;

        let dir = std::env::temp_dir().join("knudge-fs-symlink-test");
        std::fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
        let target = dir.join("target.md");
        let link = dir.join("link.md");
        let _ignored = std::fs::remove_file(&link);
        std::fs::write(&target, b"x").map_err(|e| Error::io(&target, e))?;
        symlink(&target, &link).map_err(|e| Error::io(&link, e))?;

        let fs = StdFs::new();
        assert!(matches!(fs.read(&link), Err(Error::UnsafeBlocked(_))));

        let _ignored = std::fs::remove_file(&link);
        let _ignored = std::fs::remove_file(&target);
        Ok(())
    }

    #[test]
    fn missing_file_reads_as_not_found() -> Result<()> {
        let dir = std::env::temp_dir().join("knudge-fs-missing-test");
        std::fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
        let path = dir.join("nao-existe.md");
        let _ignored = std::fs::remove_file(&path);

        let fs = StdFs::new();
        assert!(matches!(fs.read(&path), Err(Error::NotFound(_))));
        // Diretório também não tem conteúdo legível: mesma degradação para `NotFound`.
        assert!(matches!(fs.read(&dir), Err(Error::NotFound(_))));
        Ok(())
    }

    #[test]
    fn exclusive_create_conflicts() -> Result<()> {
        let dir = std::env::temp_dir().join("knudge-fs-exclusive-test");
        std::fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
        let path = dir.join("lock");
        let _ignored = std::fs::remove_file(&path);

        let fs = StdFs::new();
        fs.create_exclusive(&path, b"1")?;
        let second = fs.create_exclusive(&path, b"2");
        assert!(matches!(second, Err(Error::Conflict(_))));
        let _ignored = std::fs::remove_file(&path);
        Ok(())
    }
}
