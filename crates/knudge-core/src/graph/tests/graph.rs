//! Projeção do grafo, `link` e `expand` (E05-T01/T04).

use crate::Result;
use crate::graph::{Graph, link};
use crate::schema::{EdgeKind, NoteType, id};

use super::{frontmatter, note};

#[test]
fn builds_projection_and_lists_targets() -> Result<()> {
    let a = note(NoteType::Fact, "A", &[])?;
    let a_id = a.id()?.to_string();
    let b = note(NoteType::Decision, "B", &[(EdgeKind::DependsOn, &a_id)])?;
    let b_id = b.id()?.to_string();

    let graph = Graph::from_notes(vec![a, b])?;

    assert!(graph.contains(&a_id));
    assert!(!graph.contains("fact_ffffffff"));
    assert_eq!(graph.note_type(&b_id), Some(NoteType::Decision));
    assert_eq!(graph.targets(&b_id, EdgeKind::DependsOn).to_vec(), [a_id]);
    assert!(graph.targets(&b_id, EdgeKind::Supports).is_empty());
    assert_eq!(graph.ids().len(), 2);
    Ok(())
}

#[test]
fn link_is_idempotent_and_validates() -> Result<()> {
    let mut fm = frontmatter(NoteType::Fact, "A")?;
    let own = fm.id()?.to_string();
    let target = id::note_id(NoteType::Fact, "B");

    assert!(link(&mut fm, EdgeKind::References, &target)?);
    assert!(
        !link(&mut fm, EdgeKind::References, &target)?,
        "não duplica"
    );
    assert_eq!(fm.string_list("references")?, [target.as_str()]);

    assert!(link(&mut fm, EdgeKind::References, "não-é-id").is_err());
    assert!(link(&mut fm, EdgeKind::References, &own).is_err());
    Ok(())
}

#[test]
fn edges_serialize_in_canonical_order() -> Result<()> {
    let a = id::note_id(NoteType::Fact, "A");
    let b = id::note_id(NoteType::Fact, "B");
    let mut fm = frontmatter(NoteType::Decision, "D")?;
    link(&mut fm, EdgeKind::Supports, &b)?;
    link(&mut fm, EdgeKind::DependsOn, &a)?;

    let keys: Vec<&str> = fm.keys();
    let depends = keys.iter().position(|key| *key == "depends_on");
    let supports = keys.iter().position(|key| *key == "supports");
    assert!(
        depends < supports,
        "ordem canônica: depends_on antes de supports"
    );

    let edges = fm.edges()?;
    assert_eq!(edges.len(), 2);
    assert_eq!(
        edges.first().map(|edge| edge.kind),
        Some(EdgeKind::DependsOn)
    );
    Ok(())
}

#[test]
fn expand_is_deterministic_bfs() -> Result<()> {
    let a = note(NoteType::Fact, "A", &[])?;
    let a_id = a.id()?.to_string();
    let b = note(NoteType::Fact, "B", &[(EdgeKind::DependsOn, &a_id)])?;
    let b_id = b.id()?.to_string();
    let c = note(NoteType::Fact, "C", &[(EdgeKind::DependsOn, &b_id)])?;
    let c_id = c.id()?.to_string();

    let graph = Graph::from_notes(vec![a, b, c])?;
    let expanded = graph.expand(&c_id, Some(EdgeKind::DependsOn), 2);
    let hits: Vec<(&str, EdgeKind, u32)> = expanded
        .iter()
        .map(|hit| (hit.id.as_str(), hit.kind, hit.depth))
        .collect();
    assert_eq!(
        hits,
        [
            (b_id.as_str(), EdgeKind::DependsOn, 1),
            (a_id.as_str(), EdgeKind::DependsOn, 2)
        ]
    );

    let other_kind = graph.expand(&c_id, Some(EdgeKind::Supports), 2);
    assert!(other_kind.is_empty());
    assert!(graph.expand("fact_00000000", None, 2).is_empty());
    Ok(())
}

#[test]
fn expand_never_follows_suggestions_only_explicit_edges() -> Result<()> {
    // `c` menciona `a` no corpo, mas não declara aresta: `expand` não vê.
    let a = note(NoteType::Fact, "A", &[])?;
    let a_id = a.id()?.to_string();
    let mut c = note(NoteType::Fact, "C", &[])?;
    c.body = format!("depende de {a_id}");
    let c_id = c.id()?.to_string();

    let graph = Graph::from_notes(vec![a, c])?;
    assert!(graph.expand(&c_id, None, 3).is_empty());
    Ok(())
}

#[test]
fn edge_keys_are_canonical_and_contiguous() {
    use crate::schema::{CANONICAL_KEYS, EDGE_KEYS, EdgeKind};

    let keys: Vec<&str> = EdgeKind::ALL.iter().map(|kind| kind.key()).collect();
    assert_eq!(keys, EDGE_KEYS.to_vec());

    let start = CANONICAL_KEYS
        .iter()
        .position(|key| *key == "superseded_by")
        .map(|index| index.saturating_add(1));
    let end = start.map(|index| index.saturating_add(EDGE_KEYS.len()));
    let window = start
        .zip(end)
        .and_then(|(start, end)| CANONICAL_KEYS.get(start..end));
    assert_eq!(window, Some(EDGE_KEYS.as_slice()));
}

#[test]
fn edge_kind_round_trips_and_rejects_unknown() {
    use std::str::FromStr;

    for kind in EdgeKind::ALL {
        assert_eq!(EdgeKind::from_str(kind.as_str()).ok(), Some(kind));
        assert_eq!(kind.to_string(), kind.as_str());
    }
    assert!(EdgeKind::from_str("dep").is_err());
    assert!(EdgeKind::from_str("unknown").is_err());
}
