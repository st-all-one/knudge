//! `kd forget` (soft/restore/purge) e `kd sync` (E12-T01, D32/D48/D84).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::git::{Persistence, SyncReport, sync as git_sync};
use knudge_core::lifecycle::{Retention, due_for_purge, retirements};
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
        return purge(session, &args.id);
    }
    let ctx = session.write_context()?;
    let revision = forget_note(&ctx, &args.id, None)?;
    let data = json!({ "id": args.id, "action": "forget", "revision": revision });
    Ok(Output::new(format!("forget|{}|r{revision}", args.id), data))
}

fn purge(session: &Session, id: &str) -> Result<Output> {
    let hook = hooks::run(session, HookEvent::PrePrune, &json!({ "id": id }))?;
    if hook.blocked {
        return Err(Error::invalid_input("hook `pre-prune` bloqueou"));
    }
    let (events, _warnings) = session.events().read_all()?;
    let retention = Retention::from_config(session.config());
    let due = due_for_purge(&retirements(&events), session.now_ms(), retention);
    if !due.iter().any(|candidate| candidate == id) {
        return Err(Error::invalid_input(format!(
            "nota {id} não está aposentada ou a retenção não venceu (D84)"
        )));
    }
    session.store().remove(id)?;
    let data = json!({ "id": id, "action": "purge" });
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
    let data = json!({
        "committed": report.committed,
        "message": report.message,
        "files": report.files,
    });
    Ok(Output::new(
        format!(
            "{}: {}",
            if report.committed {
                "commit"
            } else {
                "sem commit"
            },
            report.message
        ),
        data,
    ))
}
