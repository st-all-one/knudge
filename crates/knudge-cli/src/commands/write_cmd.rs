//! `kd write` — toda escrita: create, update (`--update`) e link (`--link`) (E12-T01).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::jsonl;
use knudge_core::schema::NoteType;
use knudge_core::write::{
    BatchMode, DedupDecision, Draft, OutcomeStatus, Patch, UpdateOutcome, WriteAction,
    WriteProposal, link, outcome, propose, update, write,
};
use serde_json::json;

use crate::cli::WriteArgs;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

use super::hooks::{self, HookEvent};
use super::input;

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
    if args.dry_run {
        let ctx = session.write_context()?;
        let proposal = propose(ctx.index(), &draft, &session.thresholds()?)?;
        let note = draft.to_note(session.now_ms())?;
        let id = note.frontmatter.id().unwrap_or_default().to_string();
        let data = json!({
            "action": "dry_run",
            "id": id,
            "decision": decision_label(&proposal),
            "candidates": proposal.candidates.iter().map(|c| json!({
                "id": c.id, "score": c.score,
            })).collect::<Vec<_>>(),
        });
        return Ok(Output::new(format!("dry-run: {id}"), data));
    }
    let ctx = session.write_context()?;
    let outcome = write(&ctx, &draft, &session.thresholds()?)?;
    let post = json!({
        "action": outcome.action.as_str(),
        "id": outcome.id,
        "revision": outcome.revision,
    });
    let _post = hooks::run(session, HookEvent::PostRecord, &post)?;
    Ok(outcome_output(
        outcome.action,
        &outcome.id,
        outcome.revision,
    ))
}

fn update_note(session: &Session, id: &str, args: &WriteArgs) -> Result<Output> {
    let patch = match &args.params {
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
    let ctx = session.write_context()?;
    let outcome = update(&ctx, id, &patch)?;
    let action = match outcome {
        UpdateOutcome::Revised { .. } => WriteAction::Updated,
        UpdateOutcome::Superseded { .. } => WriteAction::Created,
    };
    Ok(outcome_output(
        action,
        outcome.id(),
        Some(outcome.revision()),
    ))
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

fn draft_of(args: &WriteArgs) -> Result<Draft> {
    if args.clear_anchors {
        return Err(Error::invalid_input(
            "`--clear-anchors` só vale com `--update`",
        ));
    }
    let note_type: NoteType = match &args.note_type {
        Some(value) => value.parse()?,
        None => NoteType::Fact,
    };
    let summary = args
        .summary
        .clone()
        .ok_or_else(|| Error::invalid_input("`kd write` exige `--summary`"))?;
    let mut draft = Draft::new(note_type, summary);
    draft.body = input::content(&args.body)?;
    draft.tags.clone_from(&args.tags);
    draft.anchors.clone_from(&args.anchors);
    if let Some(class) = &args.class {
        draft.classification = Some(class.parse()?);
    }
    if let Some(status) = &args.status {
        draft.status = Some(status.parse()?);
    }
    draft.edges = args
        .edge
        .iter()
        .map(|spec| parse::edge(spec))
        .collect::<Result<Vec<_>>>()?;
    Ok(draft)
}

fn patch_of(args: &WriteArgs) -> Result<Patch> {
    let mut patch = Patch::default();
    if let Some(value) = &args.note_type {
        patch.note_type = Some(value.parse()?);
    }
    if let Some(summary) = &args.summary {
        patch.statement = Some(summary.clone());
    }
    if !args.body.is_empty() {
        patch.body = Some(input::content(&args.body)?);
    }
    if !args.tags.is_empty() {
        patch.tags = Some(args.tags.clone());
    }
    if args.clear_anchors {
        patch.anchors = Some(Vec::new());
    } else if !args.anchors.is_empty() {
        patch.anchors = Some(args.anchors.clone());
    }
    if let Some(class) = &args.class {
        patch.classification = Some(class.parse()?);
    }
    if let Some(status) = &args.status {
        patch.status = Some(status.parse()?);
    }
    Ok(patch)
}

fn decision_label(proposal: &WriteProposal) -> &'static str {
    match proposal.decision {
        DedupDecision::Create => "create",
        DedupDecision::Merge { .. } => "merge",
        DedupDecision::Reject { .. } => "reject",
    }
}
