//! Testes da promoção de regras governadas (D157).

use std::path::Path;

use crate::Result;
use crate::graph::Graph;
use crate::knowledge::promote::{
    Candidate, RulesPolicy, apply_block, parse_block, recommend, render_block,
};
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use crate::schema::{Classification, NoteType};
use crate::store::Note;
use crate::write::Draft;

const NOW: i64 = 1_700_000_000_000;

fn note(
    note_type: NoteType,
    statement: &str,
    classification: Classification,
    anchors: &[&str],
) -> Result<Note> {
    let mut draft = Draft::new(note_type, statement);
    draft.classification = Some(classification);
    draft.anchors = anchors.iter().map(|a| (*a).to_string()).collect();
    draft.to_note(NOW)
}

#[test]
fn recommends_only_foundational_meta_or_decision() -> Result<()> {
    let notes = vec![
        note(
            NoteType::Meta,
            "sempre rodar make check",
            Classification::Foundational,
            &["a", "b", "c"],
        )?,
        note(
            NoteType::Meta,
            "rascunho",
            Classification::Observational,
            &["a", "b", "c"],
        )?,
        note(
            NoteType::Fact,
            "um fato",
            Classification::Foundational,
            &["a", "b", "c"],
        )?,
    ];
    let graph = Graph::from_notes(notes.clone())?;
    let policy = RulesPolicy {
        enabled: true,
        max_promoted: 15,
        min_confidence: 0.5,
    };
    let found = recommend(&notes, &graph, &policy, NOW)?;
    assert_eq!(found.len(), 1);
    assert_eq!(
        found.first().map(|c| c.statement.as_str()),
        Some("sempre rodar make check")
    );
    Ok(())
}

#[test]
fn ranking_is_deterministic_by_confidence_then_id() -> Result<()> {
    let notes = vec![
        note(
            NoteType::Decision,
            "decisão a",
            Classification::Foundational,
            &["a"],
        )?,
        note(
            NoteType::Decision,
            "decisão b",
            Classification::Foundational,
            &["a", "b", "c", "d", "e"],
        )?,
    ];
    let graph = Graph::from_notes(notes.clone())?;
    let policy = RulesPolicy {
        enabled: true,
        max_promoted: 15,
        min_confidence: 0.0,
    };
    let found = recommend(&notes, &graph, &policy, NOW)?;
    assert_eq!(found.len(), 2);
    let first = found.first().map(|c| c.confidence);
    let second = found.get(1).map(|c| c.confidence);
    assert!(first >= second);
    Ok(())
}

#[test]
fn render_and_parse_round_trip() {
    let approved = vec![Candidate {
        id: "dec_abc12345".to_string(),
        statement: "sempre rodar make check".to_string(),
        confidence: 0.9,
        reason: String::new(),
    }];
    let block = render_block(&approved);
    assert_eq!(parse_block(&block), vec!["dec_abc12345".to_string()]);
}

#[test]
fn apply_block_is_idempotent_and_preserves_user_text() -> Result<()> {
    let fs = MemFs::new();
    let root = Path::new("/proj");
    fs.write_atomic(&root.join("AGENTS.md"), b"user content\n")?;
    let approved = vec![Candidate {
        id: "meta_1".to_string(),
        statement: "regra".to_string(),
        confidence: 0.8,
        reason: String::new(),
    }];
    assert!(apply_block(&fs, root, &approved)?);
    assert!(!apply_block(&fs, root, &approved)?);
    let text = String::from_utf8(fs.read(&root.join("AGENTS.md"))?).unwrap_or_default();
    assert!(text.contains("user content"));
    assert!(text.contains("[meta_1] regra"));
    assert!(apply_block(&fs, root, &[])?);
    let text = String::from_utf8(fs.read(&root.join("AGENTS.md"))?).unwrap_or_default();
    assert!(!text.contains("[meta_1]"));
    Ok(())
}
