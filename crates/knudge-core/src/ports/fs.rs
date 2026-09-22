//! Porta de sistema de arquivos.
//!
//! O domínio nunca chama `std::fs` diretamente. A porta expõe só as primitivas de persistência
//! necessárias ao store: escrita atômica (tmp + rename — D20), criação exclusiva (lock — D23),
//! append (log de eventos — D26), `fsync` em batch (D22), rename, listagem e idade de arquivo
//! (varredura de resíduos — R10).

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

    /// Acrescenta bytes ao fim do arquivo, criando-o se não existir.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a escrita falhar.
    fn append(&self, path: &Path, data: &[u8]) -> Result<()>;

    /// Cria o arquivo **exclusivamente** (falha se já existir).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Conflict` se o arquivo existir e `ErrorKind::Io` em outras falhas.
    fn create_exclusive(&self, path: &Path, data: &[u8]) -> Result<()>;

    /// Renomeia/move um arquivo ou diretório.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho de origem se falhar.
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;

    /// Lista as entradas de um diretório.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a listagem falhar.
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;

    /// Verifica existência (arquivo ou diretório).
    fn exists(&self, path: &Path) -> bool;

    /// `true` se o caminho for um diretório.
    fn is_dir(&self, path: &Path) -> bool;

    /// Cria o diretório e seus ancestrais.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a criação falhar.
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Remove um arquivo (ausência não é erro).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a remoção falhar.
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// Remove um diretório e todo o seu conteúdo (ausência não é erro).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se a remoção falhar.
    fn remove_dir_all(&self, path: &Path) -> Result<()>;

    /// Força os bytes do arquivo/diretório para o disco (`fsync`).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se o `fsync` falhar.
    fn sync(&self, path: &Path) -> Result<()>;

    /// Idade do arquivo como `mtime` em milissegundos desde a época, se disponível.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` com o caminho se os metadados não puderem ser lidos.
    fn modified_ms(&self, path: &Path) -> Result<Option<i64>>;
}
