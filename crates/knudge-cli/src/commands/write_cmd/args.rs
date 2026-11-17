//! Conversão dos argumentos de `kd write` em `Draft`/`Patch` (D147/D230).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::schema::{Classification, NoteType, Provenance};
use knudge_core::write::{Draft, Patch};

use crate::cli::WriteArgs;
use crate::commands::{input, parse};
pub(super) fn draft_of(args: &WriteArgs) -> Result<Draft> {
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
    if draft.classification.is_none()
        && matches!(
            note_type,
            NoteType::Error | NoteType::Question | NoteType::Snippet
        )
    {
        draft.classification = Some(Classification::Observational);
    }
    if let Some(status) = &args.status {
        draft.status = Some(status.parse()?);
    }
    draft.edges = args
        .edge
        .iter()
        .map(|spec| parse::edge(spec))
        .collect::<Result<Vec<_>>>()?;
    draft.claims = args
        .claim
        .iter()
        .map(|spec| parse::claim(spec))
        .collect::<Result<Vec<_>>>()?;
    draft.provenance = Provenance {
        entity: None,
        activity: args.activity.clone(),
        agent: args.agent.clone(),
        ..Provenance::default()
    };
    Ok(draft)
}

pub(super) fn patch_of(args: &WriteArgs) -> Result<Patch> {
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
