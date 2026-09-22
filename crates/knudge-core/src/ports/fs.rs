//! Porta de sistema de arquivos.
//!
//! O domínio nunca chama `std::fs` diretamente. `write_atomic` deve ser atômico
//! (tmp + rename no mesmo diretório — D20).

use std::path::{Path, PathBuf};

use crate::Result;

/// Operações de arquivo usadas pelo domínio.
pub trait Fs: Send + Sync {
    /// Lê o arquivo inteiro.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a leitura falhar.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;

    /// Escreve de forma atômica (tmp + rename no mesmo diretório).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a escrita falhar.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;

    /// Lista as entradas de um diretório.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a listagem falhar.
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;

    /// Verifica existência (arquivo ou diretório).
    fn exists(&self, path: &Path) -> bool;

    /// Cria o diretório e seus ancestrais.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a criação falhar.
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Remove um arquivo.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a remoção falhar.
    fn remove_file(&self, path: &Path) -> Result<()>;
}
