//! Modelo de erro do núcleo (R30–R35).
//!
//! Regras:
//! - `enum Error` fechado, `#[non_exhaustive]`, `Send + Sync + 'static`.
//! - Nada de `Box<dyn Error>` na API pública do core (R30).
//! - Todo erro de I/O carrega `path` (R34).
//! - `ErrorKind` é o contrato de máquina; o mapa código→exit é estável (R31/R35).

use std::path::{Path, PathBuf};

/// Taxonomia estável de erros — o contrato de máquina (R31).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Recurso (nota, arquivo, id) não encontrado.
    NotFound,
    /// Entrada inválida (argumento, schema, formato).
    InvalidInput,
    /// Conflito (lock, dedup, revisão concorrente).
    Conflict,
    /// Falha de I/O.
    Io,
    /// Tempo esgotado em operação externa.
    Timeout,
    /// Erro de configuração.
    Config,
    /// Violação do schema canônico.
    Schema,
    /// Operação bloqueada pela política de `unsafe`/segurança.
    UnsafeBlocked,
    /// Falha interna inesperada.
    Internal,
}

impl ErrorKind {
    /// Código estável de máquina (nunca traduzido).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::InvalidInput => "invalid_input",
            Self::Conflict => "conflict",
            Self::Io => "io",
            Self::Timeout => "timeout",
            Self::Config => "config",
            Self::Schema => "schema",
            Self::UnsafeBlocked => "unsafe_blocked",
            Self::Internal => "internal",
        }
    }

    /// Exit code do processo associado ao código (R35).
    ///
    /// `101` é reservado a panic (não é um `ErrorKind`) e `0` a sucesso.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::NotFound => 3,
            Self::InvalidInput => 2,
            Self::Conflict => 4,
            Self::Io => 5,
            Self::Timeout => 6,
            Self::Config => 7,
            Self::Schema => 8,
            Self::UnsafeBlocked => 9,
            Self::Internal => 70,
        }
    }
}

/// Erro do núcleo.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Recurso não encontrado.
    #[error("não encontrado: {0}")]
    NotFound(String),
    /// Entrada inválida.
    #[error("entrada inválida: {0}")]
    InvalidInput(String),
    /// Conflito.
    #[error("conflito: {0}")]
    Conflict(String),
    /// Falha de I/O com o caminho de origem.
    #[error("erro de I/O em {path}: {source}")]
    Io {
        /// Caminho do arquivo/diretório.
        path: PathBuf,
        /// Causa de baixo nível.
        #[source]
        source: std::io::Error,
    },
    /// Tempo esgotado.
    #[error("tempo esgotado: {0}")]
    Timeout(String),
    /// Erro de configuração.
    #[error("erro de configuração: {0}")]
    Config(String),
    /// Violação de schema.
    #[error("erro de schema: {0}")]
    Schema(String),
    /// Operação bloqueada por segurança.
    #[error("operação bloqueada: {0}")]
    UnsafeBlocked(String),
    /// Falha interna.
    #[error("erro interno: {0}")]
    Internal(String),
}

impl Error {
    /// Classifica o erro.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::InvalidInput(_) => ErrorKind::InvalidInput,
            Self::Conflict(_) => ErrorKind::Conflict,
            Self::Io { .. } => ErrorKind::Io,
            Self::Timeout(_) => ErrorKind::Timeout,
            Self::Config(_) => ErrorKind::Config,
            Self::Schema(_) => ErrorKind::Schema,
            Self::UnsafeBlocked(_) => ErrorKind::UnsafeBlocked,
            Self::Internal(_) => ErrorKind::Internal,
        }
    }

    /// Indica se a operação pode ser repetida com segurança (R31).
    #[must_use]
    pub const fn retryable(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// Cria um [`Error::NotFound`].
    #[must_use]
    #[cold]
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Cria um [`Error::InvalidInput`].
    #[must_use]
    #[cold]
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Cria um [`Error::Conflict`].
    #[must_use]
    #[cold]
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    /// Cria um [`Error::Timeout`].
    #[must_use]
    #[cold]
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Cria um [`Error::Config`].
    #[must_use]
    #[cold]
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Cria um [`Error::Schema`].
    #[must_use]
    #[cold]
    pub fn schema(msg: impl Into<String>) -> Self {
        Self::Schema(msg.into())
    }

    /// Cria um [`Error::Internal`].
    #[must_use]
    #[cold]
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Cria um erro de I/O carregando o caminho (R34).
    #[must_use]
    #[cold]
    pub fn io(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

/// `Result` do núcleo.
pub type Result<T> = std::result::Result<T, Error>;

/// Adquire um `MutexGuard` sem `unwrap`/`expect` (R32).
///
/// Em caso de envenenamento, recupera o estado interno em vez de panicar.
pub fn lock_or_recover<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_and_code_are_consistent() {
        let err = Error::not_found("nota");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert_eq!(err.kind().code(), "not_found");
        assert_eq!(err.kind().exit_code(), 3);
        assert!(!err.retryable());
    }

    #[test]
    fn timeout_is_retryable() {
        assert!(Error::timeout("http").retryable());
    }

    #[test]
    fn io_error_chains_source_and_keeps_path() {
        let err = Error::io("/tmp/x.md", std::io::Error::other("boom"));
        let text = err.to_string();
        assert!(
            text.contains("/tmp/x.md"),
            "mensagem deve conter o path: {text}"
        );
        let source = std::error::Error::source(&err);
        assert!(source.is_some(), "source() deve encadear");
    }

    #[test]
    fn error_is_send_sync_static() {
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_send_sync_static::<Error>();
    }

    #[test]
    fn lock_or_recover_survives_poison() {
        let m = std::sync::Mutex::new(7_u32);
        // Envenena o mutex sem usar `panic!` (proibido por D92/clippy).
        let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = lock_or_recover(&m);
            std::panic::resume_unwind(Box::new("poison"));
        }));
        assert!(poisoned.is_err());
        let value = *lock_or_recover(&m);
        assert_eq!(value, 7);
    }
}
