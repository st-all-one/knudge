//! Check de corpo/lastro e slots mínimos das notas (D162/D191).

use std::collections::BTreeMap;

use crate::Result;
use crate::schema::{NoteType, Value, missing_slots};
use crate::store::Note;

use super::{CheckId, DoctorCheck};

/// Notas de conhecimento ativas sem corpo/lastro (D162) ou com slots ausentes (D191).
/// Advisório — não bloqueia `healthy`.
pub(super) fn body_check(notes: &[Note]) -> Result<DoctorCheck> {
    let mut without_body = 0_usize;
    let mut without_evidence = 0_usize;
    let mut incomplete = 0_usize;
    let mut missing_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for note in notes {
        let note_type = note.frontmatter.note_type()?;
        if matches!(note_type, NoteType::Task | NoteType::Epic) {
            continue;
        }
        if note.frontmatter.status()?.is_deprecated() {
            continue;
        }
        let anchors = note.frontmatter.string_list("anchors")?;
        let edges = note.frontmatter.edges()?.len();
        let missing = missing_slots(note_type, &note.body, anchors.len(), edges);
        if !missing.is_empty() {
            incomplete = incomplete.saturating_add(1);
            for slot in missing {
                let count = missing_counts.entry(slot).or_insert(0);
                *count = count.saturating_add(1);
            }
        }
        if !note.body.trim().is_empty() {
            continue;
        }
        without_body = without_body.saturating_add(1);
        let outcomes = match note.frontmatter.get("outcomes") {
            Some(Value::List(items)) => items.len(),
            _ => 0,
        };
        if anchors.is_empty() && outcomes == 0 {
            without_evidence = without_evidence.saturating_add(1);
        }
    }
    let ok = without_body == 0 && incomplete == 0;
    let detail = if ok {
        "todas as notas de conhecimento têm corpo e slots mínimos".to_string()
    } else {
        let top = missing_counts
            .iter()
            .max_by_key(|(_, count)| **count)
            .map_or(String::new(), |(slot, count)| {
                format!(" (slot mais ausente: {slot} em {count})")
            });
        format!(
            "{without_body} sem corpo; {without_evidence} sem lastro (corpo+outcome+âncora); \
             {incomplete} com slots ausentes{top}"
        )
    };
    Ok(DoctorCheck {
        id: CheckId::Body,
        ok,
        detail,
        fixable: false,
    })
}
