//! Porta de log.
//!
//! **stdout = dados, stderr = logs** (R20). Esta porta é a única forma de o domínio registrar
//! eventos; a implementação real (`tracing`) fica no `knudge-cli`. A redação (R22) acontece na
//! implementação, usando [`crate::logging::Redactor`].

/// Nível de log, com semântica documentada (R21).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Falha que impede a operação.
    Error,
    /// Situação anômala, operação segue.
    Warn,
    /// Marco de progresso normal.
    Info,
    /// Detalhe de diagnóstico.
    Debug,
    /// Rastreamento fino.
    Trace,
}

/// Registro estruturado de log.
#[derive(Debug)]
pub struct LogRecord<'a> {
    /// Nível.
    pub level: Level,
    /// Mensagem (nunca contém corpo de nota nem segredo).
    pub message: &'a str,
    /// Campos estruturados `(chave, valor)`.
    pub fields: &'a [(&'a str, &'a str)],
}

/// Destino de log.
pub trait Logger: Send + Sync {
    /// Emite um registro. Nunca deve panicar nem escrever em stdout.
    fn log(&self, record: &LogRecord<'_>);
}
