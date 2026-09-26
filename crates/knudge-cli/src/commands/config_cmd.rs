//! `kd config` — get/set/unset/list em dois níveis (E12-T01, D61).

use std::path::{Path, PathBuf};

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::config::{Config, global_config_path};
use serde_json::json;

use crate::cli::ConfigCommand;
use crate::output::Output;
use crate::session::Session;

/// Nível da configuração.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    /// Config global (`~/.config/knudge/config.toml`).
    Global,
    /// Config do projeto (`.knudge/config.toml`).
    Project,
}

impl Scope {
    /// Rótulo canônico.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Project => "project",
        }
    }
}

/// Executa `kd config <subcomando>`.
///
/// # Errors
/// Propaga erros de leitura/escrita/validação da config.
pub fn run(session: &Session, command: &ConfigCommand) -> Result<Output> {
    match command {
        ConfigCommand::Get { key, global } => get(
            session,
            key,
            if *global {
                Scope::Global
            } else {
                Scope::Project
            },
        ),
        ConfigCommand::Set { key, value, global } => set(
            session,
            key,
            value,
            if *global {
                Scope::Global
            } else {
                Scope::Project
            },
        ),
        ConfigCommand::Unset { key, global } => unset(
            session,
            key,
            if *global {
                Scope::Global
            } else {
                Scope::Project
            },
        ),
        ConfigCommand::List { global } => list(
            session,
            if *global {
                Scope::Global
            } else {
                Scope::Project
            },
        ),
    }
}

fn target(session: &Session, scope: Scope) -> Result<PathBuf> {
    match scope {
        Scope::Global => global_config_path(session.env()),
        Scope::Project => Ok(session.project().config_path()),
    }
}

fn load(session: &Session, path: &Path) -> Result<Config> {
    Ok(Config::load(session.fs_dyn(), path)?.unwrap_or_default())
}

fn get(session: &Session, key: &str, scope: Scope) -> Result<Output> {
    let path = target(session, scope)?;
    let config = load(session, &path)?;
    let entry = config.list().into_iter().find(|(name, _)| name == key);
    let Some((name, value)) = entry else {
        return Err(Error::not_found(format!("chave ausente: {key}")));
    };
    let data = json!({ "key": name, "value": value, "scope": scope.as_str() });
    Ok(Output::new(
        format!("{name} = {value} ({}) — {}", scope.as_str(), path.display()),
        data,
    ))
}

fn set(session: &Session, key: &str, value: &str, scope: Scope) -> Result<Output> {
    let path = target(session, scope)?;
    let mut config = load(session, &path)?;
    config.set_str(key, value)?;
    config.save(session.fs_dyn(), &path)?;
    tracing::info!(key, scope = scope.as_str(), "config atualizada");
    let data = json!({ "key": key, "value": value, "scope": scope.as_str() });
    Ok(Output::new(
        format!("{key} = {value} ({}) — {}", scope.as_str(), path.display()),
        data,
    ))
}

fn unset(session: &Session, key: &str, scope: Scope) -> Result<Output> {
    let path = target(session, scope)?;
    let mut config = load(session, &path)?;
    let removed = config.unset(key)?;
    if removed {
        config.save(session.fs_dyn(), &path)?;
    }
    let data = json!({ "key": key, "removed": removed, "scope": scope.as_str() });
    Ok(Output::new(
        if removed {
            format!("{key} removida ({}) — {}", scope.as_str(), path.display())
        } else {
            format!("{key} já ausente ({}) — {}", scope.as_str(), path.display())
        },
        data,
    ))
}

fn list(session: &Session, scope: Scope) -> Result<Output> {
    let path = target(session, scope)?;
    let config = load(session, &path)?;
    let entries = config.list();
    let body = if entries.is_empty() {
        "(vazio)".to_string()
    } else {
        entries
            .iter()
            .map(|(key, value)| format!("{key} = {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let text = format!(
        "{} ({}) — {}\n{body}",
        scope.as_str(),
        entries.len(),
        path.display()
    );
    let data = json!({
        "scope": scope.as_str(),
        "entries": entries.iter().map(|(key, value)| json!({
            "key": key, "value": value,
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}
