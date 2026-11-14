//! Rebuild double-buffer do índice derivado (D27).
//!
//! O índice vive em `<dir>` (ex.: `.idx/`). A reconstrução escreve em `<dir>.new/` e troca os
//! diretórios por `rename` (atômico): um leitor concorrente vê o índice antigo **ou** o novo,
//! nunca pela metade.

use std::ffi::OsStr;
use std::mem::DropGuard;
use std::path::{Path, PathBuf};

use crate::Result;
use crate::ports::Fs;

/// Staging do rebuild: escreve no buffer novo e troca no `commit`.
pub struct Staging<'a> {
    fs: &'a dyn Fs,
    dir: PathBuf,
    new_dir: PathBuf,
}

impl<'a> Staging<'a> {
    /// Inicia o staging, limpando um `.new` remanescente.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se criação/limpeza falharem.
    pub fn begin(fs: &'a dyn Fs, dir: impl Into<PathBuf>) -> Result<Self> {
        let dir = dir.into();
        let new_dir = with_suffix(&dir, ".new");
        fs.remove_dir_all(&new_dir)?;
        fs.create_dir_all(&new_dir)?;
        Ok(Self { fs, dir, new_dir })
    }

    /// Caminho do buffer novo (para escrita).
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.new_dir
    }

    /// Caminho final do índice.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Escreve um arquivo relativo dentro do buffer novo.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a escrita falhar.
    pub fn write(&self, relative: &str, data: &[u8]) -> Result<()> {
        let path = self.new_dir.join(relative);
        if let Some(parent) = path.parent() {
            self.fs.create_dir_all(parent)?;
        }
        self.fs.write_atomic(&path, data)
    }

    /// Troca o buffer novo pelo índice atual (rename + limpeza do antigo).
    ///
    /// Força os bytes novos ao disco **antes** do rename (D22). Se o rename do buffer novo
    /// falhar, o índice antigo é restaurado de `.old` (rollback via [`DropGuard`]).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se algum `fsync`/`rename` falhar; o índice canônico permanece
    /// o antigo.
    pub fn commit(self) -> Result<()> {
        let Self { fs, dir, new_dir } = self;
        sync_tree(fs, &new_dir)?;
        let old_dir = with_suffix(&dir, ".old");
        fs.remove_dir_all(&old_dir)?;
        let had_old = fs.exists(&dir);
        if had_old {
            fs.rename(&dir, &old_dir)?;
        }
        let guard = DropGuard::new(had_old, |had_old| {
            if had_old {
                let _ignored = fs.rename(&old_dir, &dir);
            }
        });
        fs.rename(&new_dir, &dir)?;
        DropGuard::dismiss(guard);
        fs.remove_dir_all(&old_dir)
    }
}

/// `fsync` recursivo de arquivos e diretórios (D22).
fn sync_tree(fs: &dyn Fs, dir: &Path) -> Result<()> {
    for path in fs.list_dir(dir)? {
        if fs.is_dir(&path) {
            sync_tree(fs, &path)?;
        } else {
            fs.sync(&path)?;
        }
    }
    fs.sync(dir)
}

fn with_suffix(dir: &Path, suffix: &str) -> PathBuf {
    let mut name = dir.file_name().map_or_default(OsStr::to_os_string);
    name.push(suffix);
    dir.with_file_name(name)
}
