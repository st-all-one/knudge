//! `kd self` — version, completions, setup e upgrade (E12-T05, D69).

use std::fmt::Write as _;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::ports::Fs;
use serde_json::json;

use crate::cli::SelfCommand;
use crate::output::Output;
use crate::session::Session;

/// Versão do binário.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Clientes suportados por `kd self setup`.
const CLIENTS: [&str; 4] = ["claude", "cursor", "codex", "pi"];

/// `kd self version`.
#[must_use]
pub fn version() -> Output {
    Output::new(
        format!("kd {VERSION}"),
        json!({ "version": VERSION, "name": "knudge" }),
    )
}

/// `kd self completions <shell>`.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` para shell desconhecido.
pub fn completions(shell: &str) -> Result<Output> {
    let script = match shell {
        "bash" => bash(),
        "zsh" => zsh(),
        "fish" => fish(),
        other => {
            return Err(Error::invalid_input(format!(
                "shell desconhecido: {other:?} (use bash|zsh|fish)"
            )));
        }
    };
    let data = json!({ "shell": shell, "script": &script });
    Ok(Output::new(script, data))
}

/// `kd self setup <client>` e `kd self upgrade`.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` para cliente desconhecido.
pub fn setup(session: &Session, command: &SelfCommand) -> Result<Output> {
    match command {
        SelfCommand::Setup { client } => setup_client(session, client),
        SelfCommand::Upgrade => upgrade(),
        SelfCommand::Version => Ok(version()),
        SelfCommand::Completions { shell } => completions(shell),
    }
}

fn setup_client(session: &Session, client: &str) -> Result<Output> {
    if !CLIENTS.contains(&client) {
        return Err(Error::invalid_input(format!(
            "cliente desconhecido: {client:?} (use {})",
            CLIENTS.join("|")
        )));
    }
    let dir = session.knowledge_dir().join("setup");
    let path = dir.join(format!("{client}.json"));
    let recipe = json!({
        "knudge": {
            "version": VERSION,
            "client": client,
            "binary": "kd",
            "verbs": ["prime", "rewind", "ask", "write", "task", "maintenance"],
            "protocol": "AGENTS.md",
        }
    });
    let bytes = serde_json::to_vec_pretty(&recipe)
        .map_err(|error| Error::internal(format!("recipe inválida: {error}")))?;
    session.fs().create_dir_all(&dir)?;
    session.fs().write_atomic(&path, &bytes)?;
    let data = json!({ "client": client, "path": path });
    Ok(Output::new(
        format!("recipe {client} gravada em {}", path.display()),
        data,
    ))
}

fn upgrade() -> Result<Output> {
    Err(Error::invalid_input(
        "atualização automática não disponível; instale a nova versão pelo canal de origem",
    ))
}

fn verbs() -> &'static str {
    "init prime rewind ask write task maintenance config forget sync self"
}

fn bash() -> String {
    format!(
        "# bash completion for kd\n_kd() {{\n    local cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n    COMPREPLY=( $(compgen -W \"{}\" -- \"$cur\") )\n}}\ncomplete -F _kd kd\n",
        verbs()
    )
}

fn zsh() -> String {
    format!(
        "#compdef kd\n_kd() {{\n    compadd -- {}\n}}\ncompdef _kd kd\n",
        verbs()
    )
}

fn fish() -> String {
    let mut out = String::from("# fish completion for kd\n");
    for verb in verbs().split_whitespace() {
        let _ignored = writeln!(out, "complete -c kd -n '__fish_use_subcommand' -a {verb}");
    }
    out
}
