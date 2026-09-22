//! Sugestões conservadoras (E05-T02 / D49-D50).

use std::collections::BTreeSet;

use crate::graph::{Suggestion, extract};
use crate::schema::{EdgeKind, NoteType, id};

fn known(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|id| (*id).to_string()).collect()
}

fn references(target: &str, reason: &str) -> Suggestion {
    Suggestion {
        kind: EdgeKind::References,
        target: target.to_string(),
        reason: reason.to_string(),
    }
}

#[test]
fn bare_mention_suggests_references() {
    let target = id::note_id(NoteType::Fact, "Alvo");
    let set = known(&[&target]);
    let found = extract(&format!("veja {target} agora"), &set);
    assert_eq!(found, vec![references(&target, "menção")]);
}

#[test]
fn wikilink_suggests_references() {
    let target = id::note_id(NoteType::Fact, "Alvo");
    let set = known(&[&target]);
    let found = extract(&format!("veja [[{target}]]"), &set);
    assert_eq!(found, vec![references(&target, "wikilink")]);
}

#[test]
fn verb_selects_edge_kind() {
    let target = id::note_id(NoteType::Fact, "Alvo");
    let set = known(&[&target]);

    let depends = extract(&format!("isto depende de {target}"), &set);
    assert_eq!(
        depends.first().map(|item| item.kind),
        Some(EdgeKind::DependsOn)
    );

    let upper = extract(&format!("CONTRADIZ {target}"), &set);
    assert_eq!(
        upper.first().map(|item| item.kind),
        Some(EdgeKind::Contradicts)
    );

    let replaces = extract(&format!("substitui {target}"), &set);
    assert_eq!(
        replaces.first().map(|item| item.kind),
        Some(EdgeKind::Replaces)
    );
}

#[test]
fn unknown_target_is_ignored() {
    let target = id::note_id(NoteType::Fact, "Alvo");
    assert!(extract(&format!("veja {target}"), &BTreeSet::new()).is_empty());
}

#[test]
fn invalid_prefix_or_boundary_is_ignored() {
    let target = id::note_id(NoteType::Fact, "Alvo");
    let set = known(&[&target]);
    assert!(extract("foo_12345678", &set).is_empty());
    assert!(extract(&format!("x{target}"), &set).is_empty());
    assert!(extract(&format!("{target}9"), &set).is_empty());
}
