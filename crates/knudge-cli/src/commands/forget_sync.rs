//! `kd forget` (soft/restore/purge) e `kd sync` (E12-T01, D32/D48/D84).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::git::{Persistence, SyncReport, sync as git_sync};
use knudge_core::lifecycle::{Retention, due_for_purge, retirements};
use knudge_core::ports::Git;
use knudge_core::schema::Status;
use knudge_core::write::{forget as forget_note, restore};
use serde_json::json;

use crate::cli::{ForgetArgs, SyncArgs};
use crate::output::Output;
use crate::session::Session;

use super::hooks::{self, HookEvent};

/// Executa `kd forget` (soft por padrão; `--restore`/`--purge`).
///
/// # Errors
/// Propaga erros de leitura/escrita e retenção não vencida.
pub fn forget(session: &Session, args: &ForgetArgs) -> Result<Output> {
    if args.restore {
        let ctx = session.write_context()?;
        let revision = restore(&ctx, &args.id)?;
        let data = json!({ "id": args.id, "action": "restore", "revision": revision });
        return Ok(Output::new(
            format!("restore|{}|r{revision}", args.id),
            data,
        ));
    }
    if args.purge {
        let mode = if args.force {
            PurgeMode::Force
        } else {
            PurgeMode::Retention
        };
        return purge(session, &args.id, mode);
    }
    let ctx = session.write_context()?;
    let revision = forget_note(&ctx, &args.id, None)?;
    let data = json!({ "id": args.id, "action": "forget", "revision": revision });
    Ok(Output::new(format!("forget|{}|r{revision}", args.id), data))
}

/// Modo do `--purge`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PurgeMode {
    /// Exige a janela de retenção vencida (D84).
    Retention,
    /// Ignora a retenção, mas só purga estado aposentado.
    Force,
}

fn purge(session: &Session, id: &str, mode: PurgeMode) -> Result<Output> {
    let hook = hooks::run(session, HookEvent::PrePrune, &json!({ "id": id }))?;
    if hook.blocked {
        return Err(Error::invalid_input("hook `pre-prune` bloqueou"));
    }
    match mode {
        // `--force` só purga estado aposentado (`forgotten`/`superseded`), nunca nota viva.
        PurgeMode::Force => {
            let status = session.store().read(id)?.frontmatter.status()?;
            if !matches!(status, Status::Forgotten | Status::Superseded) {
                return Err(Error::invalid_input(format!(
                    "nota {id} está `{status}`; use `kd forget --id {id}` antes de `--purge --force`"
                )));
            }
        }
        PurgeMode::Retention => {
            let (events, _warnings) = session.events().read_all()?;
            let retention = Retention::from_config(session.config());
            let due = due_for_purge(&retirements(&events), session.now_ms(), retention);
            if !due.iter().any(|candidate| candidate == id) {
                return Err(Error::invalid_input(format!(
                    "nota {id} não está aposentada ou a retenção não venceu (D84); use `--force` para purgar um tombstone"
                )));
            }
        }
    }
    session.store().remove(id)?;
    let forced = mode == PurgeMode::Force;
    let data = json!({ "id": id, "action": "purge", "forced": forced });
    Ok(Output::new(format!("purge|{id}"), data))
}

/// Executa `kd sync`.
///
/// # Errors
/// Propaga erros de git e de persistência.
pub fn sync(session: &Session, args: &SyncArgs) -> Result<Output> {
    let persistence = if session
        .config()
        .get_bool("knowledge.persist_in_project")
        .unwrap_or(true)
    {
        Persistence::Versioned
    } else {
        Persistence::LocalOnly
    };
    let report: SyncReport = git_sync(
        session.fs_dyn(),
        session.git(),
        session.project(),
        persistence,
        args.message.as_deref(),
    )?;
    let (branch, hash) = if report.committed {
        let root = session.project_root().to_string_lossy().into_owned();
        (
            git_field(session, &["-C", &root, "rev-parse", "--abbrev-ref", "HEAD"]),
            git_field(session, &["-C", &root, "rev-parse", "--short", "HEAD"]),
        )
    } else {
        (None, None)
    };
    let text = if report.committed {
        let branch = branch.as_deref().unwrap_or("?");
        let short = hash
            .as_deref()
            .map(|value| format!(" ({value})"))
            .unwrap_or_default();
        format!(
            "sync {branch}: {} arquivo(s) commitados{short}",
            report.files.len()
        )
    } else if report.message == "sem mudanças" {
        "nada a sincronizar".to_string()
    } else {
        format!("sem commit: {}", report.message)
    };
    tracing::info!(
        committed = report.committed,
        files = report.files.len(),
        "sync concluído"
    );
    let data = json!({
        "committed": report.committed,
        "message": report.message,
        "files": report.files,
        "branch": branch,
        "hash": hash,
    });
    Ok(Output::new(text, data))
}

/// Campo de `git` (uma linha), tolerante a falha: `None` se o comando falhar/vazio.
fn git_field(session: &Session, args: &[&str]) -> Option<String> {
    let output = session.git().run(args).ok()?;
    if output.status != 0 {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}
