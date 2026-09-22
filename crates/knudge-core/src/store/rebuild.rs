//! Rebuild double-buffer do índice derivado (D27).
//!
//! O índice vive em `<dir>` (ex.: `.idx/`). A reconstrução escreve em `<dir>.new/` e troca os
//! diretórios por `rename` (atômico): um leitor concorrente vê o índice antigo **ou** o novo,
//! nunca pela metade.

use std::ffi::OsStr;
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
    /// Força os bytes novos ao disco **antes** do rename (D22).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se algum `fsync`/`rename` falhar.
    pub fn commit(self) -> Result<()> {
        sync_tree(self.fs, &self.new_dir)?;
        let old_dir = with_suffix(&self.dir, ".old");
        self.fs.remove_dir_all(&old_dir)?;
        if self.fs.exists(&self.dir) {
            self.fs.rename(&self.dir, &old_dir)?;
        }
        self.fs.rename(&self.new_dir, &self.dir)?;
        self.fs.remove_dir_all(&old_dir)
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
    let mut name = dir.file_name().map(OsStr::to_os_string).unwrap_or_default();
    name.push(suffix);
    dir.with_file_name(name)
}
