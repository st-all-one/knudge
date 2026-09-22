//! `kd write` — toda escrita: create, update (`--update`) e link (`--link`) (E12-T01).

use std::io::Read;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::schema::NoteType;
use knudge_core::write::{
    DedupDecision, Draft, OutcomeStatus, Patch, UpdateOutcome, WriteAction, WriteProposal, link,
    outcome, propose, update, write,
};
use serde_json::json;

use crate::cli::WriteArgs;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

use super::hooks::{self, HookEvent};

/// Executa `kd write` no modo adequado (link > update > create).
///
/// # Errors
/// Propaga erros de validação, dedup e I/O do domínio.
pub fn run(session: &Session, args: &WriteArgs) -> Result<Output> {
    if args.outcome.is_some() {
        return outcome_note(session, args);
    }
    if let Some(spec) = &args.link {
        return link_edge(session, spec);
    }
    if let Some(id) = &args.update {
        return update_note(session, id, args);
    }
    create_note(session, args)
}

fn outcome_note(session: &Session, args: &WriteArgs) -> Result<Output> {
    if args.update.is_some() || args.link.is_some() || args.dry_run {
        return Err(Error::invalid_input(
            "`--outcome` não combina com `--update`, `--link` ou `--dry-run`",
        ));
    }
    let Some(id) = args.statement.first() else {
        return Err(Error::invalid_input("`--outcome` exige um id (STATEMENT)"));
    };
    if args.statement.len() != 1 {
        return Err(Error::invalid_input(
            "`--outcome` exige exatamente um id (STATEMENT)",
        ));
    }
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
    let patch = patch_of(args)?;
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
    let note_type: NoteType = match &args.note_type {
        Some(value) => value.parse()?,
        None => NoteType::Fact,
    };
    let mut draft = Draft::new(note_type, args.statement.join(" "));
    draft.body = read_body(args.body.as_deref())?;
    draft.confidence = args.confidence.unwrap_or(0.7);
    draft.tags.clone_from(&args.tags);
    draft.anchors.clone_from(&args.anchors);
    draft.checks.clone_from(&args.checks);
    draft.source.clone_from(&args.source);
    if let Some(class) = &args.class {
        draft.classification = Some(class.parse()?);
    }
    if let Some(status) = &args.status {
        draft.status = Some(status.parse()?);
    }
    draft.expires_at = parse::timestamp_opt(args.expires_at.as_ref())?;
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
    if !args.statement.is_empty() {
        patch.statement = Some(args.statement.join(" "));
    }
    if args.body.is_some() {
        patch.body = Some(read_body(args.body.as_deref())?);
    }
    patch.confidence = args.confidence;
    if !args.tags.is_empty() {
        patch.tags = Some(args.tags.clone());
    }
    if !args.anchors.is_empty() {
        patch.anchors = Some(args.anchors.clone());
    }
    if let Some(class) = &args.class {
        patch.classification = Some(class.parse()?);
    }
    if let Some(status) = &args.status {
        patch.status = Some(status.parse()?);
    }
    patch.source.clone_from(&args.source);
    patch.expires_at = parse::timestamp_opt(args.expires_at.as_ref())?;
    Ok(patch)
}

fn read_body(value: Option<&str>) -> Result<String> {
    match value {
        Some("-") => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|error| Error::io("stdin", error))?;
            Ok(buffer)
        }
        Some(text) => Ok(text.to_string()),
        None => Ok(String::new()),
    }
}

fn decision_label(proposal: &WriteProposal) -> &'static str {
    match proposal.decision {
        DedupDecision::Create => "create",
        DedupDecision::Merge { .. } => "merge",
        DedupDecision::Reject { .. } => "reject",
    }
}
