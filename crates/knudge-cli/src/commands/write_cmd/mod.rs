//! `kd write` — toda escrita: create, update (`--update`) e link (`--link`) (E12-T01).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::jsonl;
use knudge_core::write::{
    BatchMode, DedupDecision, OutcomeStatus, Patch, UpdateOutcome, WriteAction, WriteProposal,
    link, outcome, update, write,
};
use serde_json::json;

use crate::cli::WriteArgs;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

use super::gate;
use super::hooks::{self, HookEvent};
use super::input;

mod args;
mod contract;
use args::{draft_of, patch_of};
use contract::{anchor_existence_warnings, draft_missing, dry_run_note, with_slot_warnings};

/// Executa `kd write` no modo adequado (link > update > create).
///
/// # Errors
/// Propaga erros de validação, dedup e I/O do domínio.
pub fn run(session: &Session, args: &WriteArgs) -> Result<Output> {
    if let Some(id) = &args.update {
        return update_note(session, id, args);
    }
    if let Some(params) = &args.params {
        let mode = if args.dry_run {
            BatchMode::DryRun
        } else {
            BatchMode::Apply
        };
        return super::write_batch::params_note(session, params, mode);
    }
    if let Some(source) = &args.batch {
        let mode = if args.dry_run {
            BatchMode::DryRun
        } else {
            BatchMode::Apply
        };
        return super::write_batch::batch_note(session, source, mode);
    }
    if args.outcome.is_some() {
        return outcome_note(session, args);
    }
    if let Some(spec) = &args.link {
        return link_edge(session, spec);
    }
    create_note(session, args)
}

fn outcome_note(session: &Session, args: &WriteArgs) -> Result<Output> {
    if args.update.is_some() || args.link.is_some() || args.dry_run {
        return Err(Error::invalid_input(
            "`--outcome` não combina com `--update`, `--link` ou `--dry-run`",
        ));
    }
    let Some(id) = args.id.as_deref() else {
        return Err(Error::invalid_input("`--outcome` exige `--id`"));
    };
    let status = args
        .outcome
        .as_deref()
        .unwrap_or_default()
        .parse::<OutcomeStatus>()?;
    let ctx = session.write_context()?;
    let revision = outcome(&ctx, id, status, args.note.as_deref())?;
    let data = json!({
        "action": "outcome",
        "id": id,
        "outcome": status.as_str(),
        "revision": revision,
    });
    Ok(Output::new(
        format!("outcome|{id}|{}|r{revision}", status.as_str()),
        data,
    ))
}

fn link_edge(session: &Session, spec: &str) -> Result<Output> {
    let (from, kind, to) = parse::triple(spec)?;
    let ctx = session.write_context()?;
    let created = link(&ctx, &from, kind, &to)?;
    let text = format!(
        "{from} {} {to} {}",
        kind.as_str(),
        if created { "created" } else { "unchanged" }
    );
    let data = json!({
        "action": if created { "linked" } else { "unchanged" },
        "from": from,
        "kind": kind.as_str(),
        "to": to,
    });
    Ok(Output::new(text, data))
}

#[allow(
    clippy::too_many_lines,
    reason = "valida hooks, contrato de dados, âncoras e portão antes de gravar (D230)"
)]
fn create_note(session: &Session, args: &WriteArgs) -> Result<Output> {
    let mut draft = draft_of(args)?;
    let payload = json!({
        "statement": &draft.statement,
        "body": &draft.body,
        "type": draft.note_type.as_str(),
        "tags": &draft.tags,
        "anchors": &draft.anchors,
    });
    let hook = hooks::run(session, HookEvent::PreRecord, &payload)?;
    if hook.blocked {
        return Err(Error::conflict(format!(
            "hook `{}` bloqueou a gravação",
            HookEvent::PreRecord.as_str()
        )));
    }
    hooks::apply_to_draft(&mut draft, &hook.payload);
    let anchor_warnings = anchor_existence_warnings(session, &draft.anchors);
    let missing = draft_missing(&draft);
    if args.dry_run {
        return Ok(dry_run_note(session, &draft, &missing)?.with_warnings(anchor_warnings));
    }
    if !anchor_warnings.is_empty() && session.config().strict() {
        return Err(Error::invalid_input(format!(
            "âncora inexistente: {}",
            anchor_warnings.join(", ")
        )));
    }
    if !missing.is_empty() && session.config().strict() {
        return Err(Error::invalid_input(format!(
            "data contract de `{}`: slots ausentes: {}",
            draft.note_type,
            missing.join(", ")
        )));
    }
    if gate::enforced(session) {
        let after = json!({
            "statement": &draft.statement,
            "body": &draft.body,
            "type": draft.note_type.as_str(),
            "tags": &draft.tags,
            "anchors": &draft.anchors,
        });
        let report = gate::evaluate(session, "create", None, &after)?;
        if !report.passed() {
            let failed = report.failed().join(", ");
            return Err(Error::conflict(format!(
                "portão de evidência reprovou a gravação: {failed}"
            )));
        }
    }
    let ctx = session.write_context()?;
    let outcome = write(&ctx, &draft, &session.thresholds()?)?;
    let post = json!({
        "action": outcome.action.as_str(),
        "id": outcome.id,
        "revision": outcome.revision,
    });
    let _post = hooks::run(session, HookEvent::PostRecord, &post)?;
    Ok(with_slot_warnings(
        outcome_output(outcome.action, &outcome.id, outcome.revision),
        draft.note_type,
        &missing,
        anchor_warnings,
    ))
}

fn update_note(session: &Session, id: &str, args: &WriteArgs) -> Result<Output> {
    let mut patch = match &args.params {
        Some(params) => {
            let text = if params == "-" {
                input::read_stdin()?
            } else {
                params.clone()
            };
            let value = jsonl::decode(&text)?;
            Patch::from_value(&value)?
        }
        None => patch_of(args)?,
    };
    if let Some(expected) = &args.expect_body_hash {
        patch.expected_body_hash = Some(expected.clone());
    }
    let anchor_warnings = patch
        .anchors
        .as_deref()
        .map_or_default(|anchors| anchor_existence_warnings(session, anchors));
    if !anchor_warnings.is_empty() && session.config().strict() {
        return Err(Error::invalid_input(format!(
            "âncora inexistente: {}",
            anchor_warnings.join(", ")
        )));
    }
    let ctx = session.write_context()?;
    let outcome = update(&ctx, id, &patch)?;
    let action = match outcome {
        UpdateOutcome::Revised { .. } => WriteAction::Updated,
        UpdateOutcome::Superseded { .. } => WriteAction::Created,
    };
    Ok(
        outcome_output(action, outcome.id(), Some(outcome.revision()))
            .with_warnings(anchor_warnings),
    )
}

fn outcome_output(action: WriteAction, id: &str, revision: Option<u32>) -> Output {
    let text = match revision {
        Some(rev) => format!("{}|{id}|r{rev}", action.as_str()),
        None => format!("{}|{id}", action.as_str()),
    };
    let data = json!({
        "action": action.as_str(),
        "id": id,
        "revision": revision,
    });
    Output::new(text, data)
}

pub(super) fn decision_label(proposal: &WriteProposal) -> &'static str {
    match proposal.decision {
        DedupDecision::Create => "create",
        DedupDecision::Merge { .. } => "merge",
        DedupDecision::Reject { .. } => "reject",
    }
}
