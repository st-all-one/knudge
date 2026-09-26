//! Adaptador `tracing` do port `Logger`, com redação (R21/R22) e escrita em **stderr** (R20).

use knudge_core::logging::Redactor;
use knudge_core::ports::{Level, LogRecord, Logger};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;

use crate::cli::Cli;

/// Logger que emite em stderr via `tracing`, redigindo segredos.
#[derive(Debug)]
pub struct TracingLogger {
    /// Redator aplicado a mensagens e campos.
    redactor: Redactor,
}

impl TracingLogger {
    /// Cria o logger com um redator.
    #[must_use]
    pub fn new(redactor: Redactor) -> Self {
        Self { redactor }
    }
}

impl Logger for TracingLogger {
    fn log(&self, record: &LogRecord<'_>) {
        let message = self.redactor.redact(record.message);
        let mut fields = String::new();
        for &(key, value) in record.fields {
            fields.push(' ');
            fields.push_str(key);
            fields.push('=');
            fields.push_str(&self.redactor.redact(value));
        }
        let line = if fields.is_empty() {
            message.into_owned()
        } else {
            format!("{message}{fields}")
        };
        match record.level {
            Level::Error => tracing::error!("{line}"),
            Level::Warn => tracing::warn!("{line}"),
            Level::Info => tracing::info!("{line}"),
            Level::Debug => tracing::debug!("{line}"),
            Level::Trace => tracing::trace!("{line}"),
        }
    }
}

/// Inicializa o subscriber global (stderr).
///
/// `--quiet`/`off` **não instalam** subscriber (nenhum evento é construído). Nível simples usa
/// `LevelFilter` (sem parsear diretivas de `EnvFilter`); o `EnvFilter` fica para alvos.
pub fn init(cli: &Cli) {
    let level = if cli.quiet {
        "off"
    } else {
        cli.log_level.as_str()
    };
    if level.eq_ignore_ascii_case("off") {
        return;
    }
    let builder = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false);
    if let Ok(filter) = level.parse::<LevelFilter>() {
        let _ignored = builder.with_max_level(filter).try_init();
    } else {
        let filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("warn"));
        let _ignored = builder.with_env_filter(filter).try_init();
    }
}
