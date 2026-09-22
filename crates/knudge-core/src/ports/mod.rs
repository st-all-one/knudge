//! Portas determinísticas (D65).
//!
//! O domínio só conhece estas *traits*. As implementações reais ficam em `crate::adapters`; os
//! fakes, em [`fakes`], permitem testes reprodutíveis sem tocar o sistema operacional.

pub mod embedder;
pub mod fakes;
pub mod fs;
pub mod logger;

pub use embedder::Embedder;
pub use fs::Fs;
pub use logger::{Level, LogRecord, Logger};

use std::path::PathBuf;

use crate::Result;
use crate::time::Timestamp;

/// Fonte de tempo (injetada; nunca `SystemTime::now` no domínio).
pub trait Clock: Send + Sync {
    /// Instante atual em UTC.
    fn now(&self) -> Timestamp;
}

/// Fonte de aleatoriedade (injetada; usada só para jitter, nunca para IDs — D01).
pub trait Rng: Send + Sync {
    /// Próximo valor pseudoaleatório.
    fn next_u64(&mut self) -> u64;
}

/// Acesso ao ambiente (variáveis e argumentos) sem ler o processo diretamente.
pub trait Env: Send + Sync {
    /// Lê uma variável de ambiente.
    fn var(&self, key: &str) -> Option<String>;
    /// Argumentos de linha de comando (sem o nome do programa).
    fn args(&self) -> Vec<String>;
    /// Diretório de trabalho atual.
    ///
    /// # Errors
    /// Retorna erro se o diretório atual não puder ser lido.
    fn current_dir(&self) -> Result<PathBuf>;
}

/// Saída de um comando `git`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitOutput {
    /// Código de saída (`-1` quando morto por sinal).
    pub status: i32,
    /// Bytes de stdout.
    pub stdout: Vec<u8>,
    /// Bytes de stderr.
    pub stderr: Vec<u8>,
}

impl GitOutput {
    /// `stdout` como texto, com espaços das pontas removidos.
    #[must_use]
    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).trim().to_string()
    }
}

/// Consultas de Git usadas pelas regras de worktree/persistência (E04).
pub trait Git: Send + Sync {
    /// `true` se o diretório atual está dentro de um repositório git.
    fn is_repo(&self) -> bool;
    /// Caminho de `git rev-parse --git-common-dir`, quando houver.
    fn common_dir(&self) -> Option<PathBuf>;
    /// `git rev-parse --show-toplevel` do worktree atual, quando houver.
    fn top_level(&self) -> Option<PathBuf>;
    /// Raiz do superprojeto (`git rev-parse --show-superproject-working-tree`), se houver.
    fn superproject_root(&self) -> Option<PathBuf>;
    /// Saída de `git status --porcelain` (uma linha por arquivo).
    ///
    /// # Errors
    /// Retorna erro se o comando `git` não puder ser executado.
    fn status_porcelain(&self) -> Result<Vec<String>>;
    /// Executa `git <args>` no diretório do adaptador (sem shell — R12).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se o processo não puder ser iniciado.
    fn run(&self, args: &[&str]) -> Result<GitOutput>;
}

/// Resultado da execução de um hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookOutput {
    /// Código de saída do processo.
    pub status: i32,
    /// Bytes escritos em stdout.
    pub stdout: Vec<u8>,
    /// Bytes escritos em stderr.
    pub stderr: Vec<u8>,
}

/// Executor de hooks externos (opcional; sem shell — R12).
pub trait HookRunner: Send + Sync {
    /// Executa um hook pelo nome, passando `input` em stdin.
    ///
    /// # Errors
    /// Retorna erro se o processo não puder ser iniciado ou exceder o timeout.
    fn run(&self, hook: &str, input: &[u8]) -> Result<HookOutput>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traits_are_object_safe() {
        fn assert_send_sync<T: ?Sized + Send + Sync>() {}
        assert_send_sync::<dyn Clock>();
        assert_send_sync::<dyn Rng>();
        assert_send_sync::<dyn Env>();
        assert_send_sync::<dyn Git>();
        assert_send_sync::<dyn HookRunner>();
        assert_send_sync::<dyn Fs>();
        assert_send_sync::<dyn Logger>();
        assert_send_sync::<dyn Embedder>();
    }
}
