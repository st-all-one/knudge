//! Fakes de sistema de arquivos: `MemFs` (determinístico) e `FaultyFs` (injeção de crash).

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::lock_or_recover;
use crate::ports::Fs;
use crate::{Error, Result};

mod faulty;

pub use faulty::FaultyFs;

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
    /// Diretórios criados explicitamente (diretórios vazios contam).
    dirs: Mutex<BTreeSet<PathBuf>>,
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
        {
            let mut dirs = lock_or_recover(&self.dirs);
            let keys: Vec<PathBuf> = dirs
                .iter()
                .filter(|path| *path == from || path.starts_with(from))
                .cloned()
                .collect();
            for key in keys {
                dirs.remove(&key);
                let rest = key.strip_prefix(from).unwrap_or(&key);
                dirs.insert(to.join(rest));
            }
        }
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
        let dirs = lock_or_recover(&self.dirs);
        let mut out: BTreeSet<PathBuf> = BTreeSet::new();
        // Diretórios são implícitos: projeta o primeiro componente sob `path`.
        for key in files.keys().chain(dirs.iter()) {
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
        files.contains_key(path)
            || files.keys().any(|p| p.starts_with(path))
            || lock_or_recover(&self.dirs).contains(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        let files = lock_or_recover(&self.files);
        files.keys().any(|p| p != path && p.starts_with(path))
            || lock_or_recover(&self.dirs).contains(path)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        let mut dirs = lock_or_recover(&self.dirs);
        let mut current = PathBuf::new();
        for component in path.components() {
            current.push(component);
            dirs.insert(current.clone());
        }
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        lock_or_recover(&self.files).remove(path);
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        lock_or_recover(&self.files).retain(|p, _| !p.starts_with(path));
        lock_or_recover(&self.dirs).retain(|p| !p.starts_with(path));
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

/// Caminho `*.tmp` usado na escrita atômica (mesmo diretório do destino — D20).
pub(super) fn staging_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(OsStr::to_os_string)
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}
