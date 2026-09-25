//! Check de corpo/lastro das notas (D162).

use crate::Result;
use crate::schema::{NoteType, Status, Value};
use crate::store::Note;

use super::{CheckId, DoctorCheck};

/// Notas de conhecimento ativas sem corpo e sem lastro (D162). Advisório — não bloqueia `healthy`.
pub(super) fn body_check(notes: &[Note]) -> Result<DoctorCheck> {
    let mut without_body = 0_usize;
    let mut without_evidence = 0_usize;
    for note in notes {
        if matches!(
            note.frontmatter.note_type()?,
            NoteType::Task | NoteType::Epic
        ) {
            continue;
        }
        if matches!(
            note.frontmatter.status()?,
            Status::Forgotten | Status::Superseded
        ) {
            continue;
        }
        if !note.body.trim().is_empty() {
            continue;
        }
        without_body = without_body.saturating_add(1);
        let anchors = note
            .frontmatter
            .string_list("anchors")
            .map_or(0, |items| items.len());
        let outcomes = match note.frontmatter.get("outcomes") {
            Some(Value::List(items)) => items.len(),
            _ => 0,
        };
        if anchors == 0 && outcomes == 0 {
            without_evidence = without_evidence.saturating_add(1);
        }
    }
    let ok = without_body == 0;
    Ok(DoctorCheck {
        id: CheckId::Body,
        ok,
        detail: if ok {
            "todas as notas de conhecimento têm corpo".to_string()
        } else {
            format!(
                "{without_body} sem corpo; {without_evidence} sem lastro (corpo+outcome+âncora)"
            )
        },
        fixable: false,
    })
}
