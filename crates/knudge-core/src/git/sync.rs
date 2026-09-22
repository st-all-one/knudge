//! `sync()`: commit de `notas/` + `eventos/` no worktree principal (E04-T05, D32).

use crate::ports::{Fs, Git, GitOutput};
use crate::store::EventLog;
use crate::{Error, Result};

use super::persistence::Persistence;
use super::project::Project;

/// Relatório de `sync`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncReport {
    /// `true` se um commit foi criado.
    pub committed: bool,
    /// Mensagem de commit (gerada do evento quando não fornecida).
    pub message: String,
    /// Arquivos vistos como modificados.
    pub files: Vec<String>,
}

/// Commita `notas/` + `eventos/` no worktree principal.
///
/// O guard de worktree usa `git -C <raiz>`, garantindo que o commit aconteça **no worktree
/// certo**, mesmo quando `sync` é chamado de um worktree ligado.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` fora de um repositório e `ErrorKind::Internal` se o `git`
/// falhar.
pub fn sync(
    fs: &dyn Fs,
    git: &dyn Git,
    project: &Project,
    persistence: Persistence,
    message: Option<&str>,
) -> Result<SyncReport> {
    if !project.in_repo() {
        return Err(Error::invalid_input("fora de um repositório git"));
    }
    if !persistence.is_versioned() {
        return Ok(SyncReport {
            committed: false,
            message: "persist_in_project=false".to_string(),
            files: Vec::new(),
        });
    }

    let root = project.root().to_string_lossy().into_owned();
    let paths = [".knudge/notas", ".knudge/eventos"];
    let mut status_args = vec!["-C", root.as_str(), "status", "--porcelain", "--"];
    status_args.extend(paths);

    let status = git.run(&status_args)?;
    ensure_ok("status", &status)?;
    let files: Vec<String> = String::from_utf8_lossy(&status.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    if files.is_empty() {
        return Ok(SyncReport {
            committed: false,
            message: "sem mudanças".to_string(),
            files: Vec::new(),
        });
    }

    let message = match message {
        Some(given) => given.to_string(),
        None => generate_message(fs, project, files.len())?,
    };

    let mut add_args = vec!["-C", root.as_str(), "add", "--"];
    add_args.extend(paths);
    ensure_ok("add", &git.run(&add_args)?)?;

    let commit = git.run(&["-C", root.as_str(), "commit", "-m", message.as_str()])?;
    ensure_ok("commit", &commit)?;

    Ok(SyncReport {
        committed: true,
        message,
        files,
    })
}

/// Gera uma mensagem determinística a partir do último evento do log.
fn generate_message(fs: &dyn Fs, project: &Project, changed: usize) -> Result<String> {
    let log = EventLog::new(fs, project.knowledge_dir(), EventLog::DEFAULT_MAX_BYTES);
    let (events, _warnings) = log.read_all()?;
    let head = match events.last() {
        Some(event) => match &event.note_id {
            Some(id) => format!("knudge: {} {id}", event.op),
            None => format!("knudge: {}", event.op),
        },
        None => "knudge: sync".to_string(),
    };
    Ok(if changed > 1 {
        format!("{head} (+{changed} arquivos)")
    } else {
        head
    })
}

fn ensure_ok(step: &str, output: &GitOutput) -> Result<()> {
    if output.status == 0 {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(Error::internal(format!(
        "git {step} falhou (status {}): {}",
        output.status,
        stderr.trim()
    )))
}
