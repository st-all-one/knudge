//! Data contract por tipo no `write` (E19-T03/D191): slots soft, `--dry-run` e avisos.

use knudge_core::Result;
use knudge_core::schema::{NoteType, missing_slots};
use knudge_core::write::{Draft, propose};
use serde_json::json;

use crate::output::Output;
use crate::session::Session;

use super::decision_label;

/// Simula a gravação sem tocar no corpus, explicando os slots ausentes (D191).
pub(super) fn dry_run_note(session: &Session, draft: &Draft, missing: &[&str]) -> Result<Output> {
    let ctx = session.write_context()?;
    let proposal = propose(ctx.index(), draft, &session.thresholds()?)?;
    let note = draft.to_note(session.now_ms())?;
    let id = note.frontmatter.id().unwrap_or_default().to_string();
    let data = json!({
        "action": "dry_run",
        "id": id,
        "decision": decision_label(&proposal),
        "missing_slots": missing,
        "candidates": proposal.candidates.iter().map(|c| json!({
            "id": c.id, "score": c.score,
        })).collect::<Vec<_>>(),
    });
    let text = if missing.is_empty() {
        format!("dry-run: {id}")
    } else {
        format!("dry-run: {id} | slots ausentes: {}", missing.join(", "))
    };
    Ok(Output::new(text, data))
}

/// Anexa os avisos soft do contrato de dados (D191) e das âncoras (D230).
pub(super) fn with_slot_warnings(
    output: Output,
    note_type: NoteType,
    missing: &[&str],
    extra: Vec<String>,
) -> Output {
    let mut warnings = extra;
    if !missing.is_empty() {
        warnings.push(format!(
            "data contract de `{note_type}`: slots ausentes: {}",
            missing.join(", ")
        ));
    }
    output.with_warnings(warnings)
}

/// Slots mínimos ausentes no rascunho (E19-T03/D191) — aviso soft, erro só sob `strict`.
pub(super) fn draft_missing(draft: &Draft) -> Vec<&'static str> {
    missing_slots(
        draft.note_type,
        &draft.body,
        draft.anchors.len(),
        draft.edges.len(),
    )
}

/// Avisa sobre âncoras literais que não existem na árvore do projeto (D230).
///
/// Globs são ignorados: casá-los exigiria a varredura que o `prune` já faz off-path. A checagem
/// usa a porta `Fs`, mantendo o domínio puro; `strict` promove o aviso a erro na chamada.
pub(super) fn anchor_existence_warnings(session: &Session, anchors: &[String]) -> Vec<String> {
    let mut warnings = Vec::new();
    let root = session.project_root();
    for anchor in anchors {
        if anchor.chars().any(|ch| matches!(ch, '*' | '?' | '[')) {
            continue;
        }
        let path = root.join(anchor);
        if !session.fs_dyn().exists(&path) || session.fs_dyn().is_dir(&path) {
            warnings.push(format!("âncora inexistente: {anchor}"));
        }
    }
    warnings
}
