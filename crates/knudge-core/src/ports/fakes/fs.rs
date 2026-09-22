//! Fakes de sistema de arquivos: `MemFs` (determinístico) e `FaultyFs` (injeção de crash).

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::lock_or_recover;
use crate::ports::Fs;
use crate::{Error, Result};

/// Entrada de arquivo em memória.
#[derive(Debug, Clone)]
struct Entry {
    data: Vec<u8>,
    modified_ms: i64,
}

/// Sistema de arquivos em memória.
#[derive(Debug, Default)]
pub struct MemFs {
    /// Arquivos por caminho.
    files: Mutex<BTreeMap<PathBuf, Entry>>,
}

impl MemFs {
    /// Cria um FS vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insere um arquivo diretamente (atalho de teste), com mtime na época.
    pub fn insert(&self, path: impl Into<PathBuf>, data: impl Into<Vec<u8>>) {
        self.insert_at(path, data, 0);
    }

    /// Insere um arquivo com `mtime` explícito (testes de varredura).
    pub fn insert_at(&self, path: impl Into<PathBuf>, data: impl Into<Vec<u8>>, mtime_ms: i64) {
        lock_or_recover(&self.files).insert(
            path.into(),
            Entry {
                data: data.into(),
                modified_ms: mtime_ms,
            },
        );
    }

    /// Conteúdo do arquivo, se existir (atalho de teste).
    #[must_use]
    pub fn get(&self, path: &Path) -> Option<Vec<u8>> {
        lock_or_recover(&self.files)
            .get(path)
            .map(|e| e.data.clone())
    }

    /// Quantidade de arquivos.
    #[must_use]
    pub fn len(&self) -> usize {
        lock_or_recover(&self.files).len()
    }

    /// `true` se não há arquivos.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Fs for MemFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        lock_or_recover(&self.files)
            .get(path)
            .map(|e| e.data.clone())
            .ok_or_else(|| Error::not_found(path.display().to_string()))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        // Espelha o adaptador real: escreve em `*.tmp` e renomeia (D20).
        let staging = staging_path(path);
        self.insert(staging.clone(), data.to_vec());
        self.rename(&staging, path)
    }

    fn append(&self, path: &Path, data: &[u8]) -> Result<()> {
        let mut files = lock_or_recover(&self.files);
        let entry = files.entry(path.to_path_buf()).or_insert(Entry {
            data: Vec::new(),
            modified_ms: 0,
        });
        entry.data.extend_from_slice(data);
        Ok(())
    }

    fn create_exclusive(&self, path: &Path, data: &[u8]) -> Result<()> {
        let mut files = lock_or_recover(&self.files);
        if files.contains_key(path) {
            return Err(Error::conflict(format!("já existe: {}", path.display())));
        }
        files.insert(
            path.to_path_buf(),
            Entry {
                data: data.to_vec(),
                modified_ms: 0,
            },
        );
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let mut files = lock_or_recover(&self.files);
        if let Some(entry) = files.remove(from) {
            files.insert(to.to_path_buf(), entry);
            return Ok(());
        }
        // Move de diretório: reposiciona todas as chaves sob `from`.
        let keys: Vec<PathBuf> = files
            .keys()
            .filter(|path| path.starts_with(from))
            .cloned()
            .collect();
        if keys.is_empty() {
            return Err(Error::not_found(from.display().to_string()));
        }
        for key in keys {
            if let Some(entry) = files.remove(&key) {
                let rest = key.strip_prefix(from).unwrap_or(&key);
                files.insert(to.join(rest), entry);
            }
        }
        Ok(())
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let files = lock_or_recover(&self.files);
        let mut out: BTreeSet<PathBuf> = BTreeSet::new();
        for key in files.keys() {
            // Diretórios são implícitos: projeta o primeiro componente sob `path`.
            if let Ok(rest) = key.strip_prefix(path)
                && let Some(first) = rest.components().next()
            {
                out.insert(path.join(first));
            }
        }
        Ok(out.into_iter().collect())
    }

    fn exists(&self, path: &Path) -> bool {
        let files = lock_or_recover(&self.files);
        files.contains_key(path) || files.keys().any(|p| p.starts_with(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        let files = lock_or_recover(&self.files);
        files.keys().any(|p| p != path && p.starts_with(path))
    }

    fn create_dir_all(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        lock_or_recover(&self.files).remove(path);
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        lock_or_recover(&self.files).retain(|p, _| !p.starts_with(path));
        Ok(())
    }

    fn sync(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn modified_ms(&self, path: &Path) -> Result<Option<i64>> {
        Ok(lock_or_recover(&self.files)
            .get(path)
            .map(|e| e.modified_ms))
    }
}

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

/// Caminho `*.tmp` usado na escrita atômica (mesmo diretório do destino — D20).
fn staging_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(OsStr::to_os_string)
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}
