//! `FaultyFs`: injeta falhas de escrita para testes de crash (E03).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::lock_or_recover;
use crate::ports::Fs;
use crate::{Error, Result};

use super::{MemFs, staging_path};

/// Wrapper que injeta falhas de escrita em caminhos que contêm um marcador (crash-injection).
#[derive(Debug, Default)]
pub struct FaultyFs {
    /// FS subjacente.
    inner: MemFs,
    /// Marcador; escritas cujo caminho o contém falham enquanto definido.
    needle: Mutex<Option<String>>,
}

impl FaultyFs {
    /// Cria sobre um [`MemFs`] existente.
    #[must_use]
    pub const fn new(inner: MemFs) -> Self {
        Self {
            inner,
            needle: Mutex::new(None),
        }
    }

    /// Passa a falhar escritas cujo caminho contenha `needle`.
    pub fn fail_writes_containing(&self, needle: impl Into<String>) {
        *lock_or_recover(&self.needle) = Some(needle.into());
    }

    /// Volta a permitir escritas.
    pub fn clear(&self) {
        *lock_or_recover(&self.needle) = None;
    }

    /// Acesso ao FS subjacente (inspeção em teste).
    #[must_use]
    pub const fn inner(&self) -> &MemFs {
        &self.inner
    }

    fn fails(&self, path: &Path) -> bool {
        lock_or_recover(&self.needle)
            .as_ref()
            .is_some_and(|needle| path.to_string_lossy().contains(needle.as_str()))
    }
}

impl Fs for FaultyFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.inner.read(path)
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        let staging = staging_path(path);
        if self.fails(path) || self.fails(&staging) {
            return Err(Error::io(path, std::io::Error::other("falha injetada")));
        }
        self.inner.write_atomic(path, data)
    }

    fn append(&self, path: &Path, data: &[u8]) -> Result<()> {
        if self.fails(path) {
            return Err(Error::io(path, std::io::Error::other("falha injetada")));
        }
        self.inner.append(path, data)
    }

    fn create_exclusive(&self, path: &Path, data: &[u8]) -> Result<()> {
        if self.fails(path) {
            return Err(Error::io(path, std::io::Error::other("falha injetada")));
        }
        self.inner.create_exclusive(path, data)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        self.inner.rename(from, to)
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        self.inner.list_dir(path)
    }

    fn exists(&self, path: &Path) -> bool {
        self.inner.exists(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.inner.is_dir(path)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.inner.create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        self.inner.remove_file(path)
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        self.inner.remove_dir_all(path)
    }

    fn sync(&self, path: &Path) -> Result<()> {
        self.inner.sync(path)
    }

    fn modified_ms(&self, path: &Path) -> Result<Option<i64>> {
        self.inner.modified_ms(path)
    }
}
