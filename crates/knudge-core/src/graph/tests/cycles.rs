//! Ciclos: Kosaraju e supersessão protegida (E05-T04 / D45).

use std::collections::{BTreeMap, BTreeSet};

use crate::Result;
use crate::graph::{Graph, cyclic_components, link};
use crate::schema::{EdgeKind, NoteType, id};
use crate::store::Note;

use super::{frontmatter, note, with_superseded};

#[test]
fn kosaraju_finds_two_node_cycle_and_self_loop() {
    let nodes: BTreeSet<String> = ["a", "b", "c", "d", "e"]
        .iter()
        .map(|node| (*node).to_string())
        .collect();
    let adjacency = BTreeMap::from([
        ("a".to_string(), vec!["b".to_string()]),
        ("b".to_string(), vec!["a".to_string()]),
        ("c".to_string(), vec!["c".to_string()]),
        ("d".to_string(), vec!["e".to_string()]),
        ("e".to_string(), vec![]),
    ]);

    let cycles = cyclic_components(&nodes, &adjacency);
    assert_eq!(
        cycles,
        vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()]
        ]
    );
}

#[test]
fn supersession_cycle_protects_members() -> Result<()> {
    let a_id = id::note_id(NoteType::Fact, "A");
    let b_id = id::note_id(NoteType::Fact, "B");

    let mut a = frontmatter(NoteType::Fact, "A")?;
    link(&mut a, EdgeKind::Replaces, &b_id)?;
    let a = with_superseded(a, &b_id)?;
    let mut b = frontmatter(NoteType::Fact, "B")?;
    link(&mut b, EdgeKind::Replaces, &a_id)?;
    let b = with_superseded(b, &a_id)?;

    let graph = Graph::from_notes(vec![Note::new(a, ""), Note::new(b, "")])?;
    let mut expected = vec![a_id.clone(), b_id.clone()];
    expected.sort();
    assert_eq!(graph.supersession_cycles(), vec![expected]);

    let members = graph.cycle_members();
    assert!(members.contains(&a_id));
    assert!(members.contains(&b_id));
    Ok(())
}

#[test]
fn supersession_chain_is_not_a_cycle() -> Result<()> {
    let a_id = id::note_id(NoteType::Fact, "A");
    let b_id = id::note_id(NoteType::Fact, "B");
    let c_id = id::note_id(NoteType::Fact, "C");

    let a = with_superseded(frontmatter(NoteType::Fact, "A")?, &b_id)?;
    let mut b = frontmatter(NoteType::Fact, "B")?;
    link(&mut b, EdgeKind::Replaces, &a_id)?;
    let b = with_superseded(b, &c_id)?;
    let mut c = frontmatter(NoteType::Fact, "C")?;
    link(&mut c, EdgeKind::Replaces, &b_id)?;

    let graph = Graph::from_notes(vec![Note::new(a, ""), Note::new(b, ""), Note::new(c, "")])?;
    assert!(graph.supersession_cycles().is_empty());
    assert!(graph.cycle_members().is_empty());
    Ok(())
}

#[test]
fn dependency_cycle_detected() -> Result<()> {
    let a_id = id::note_id(NoteType::Fact, "A");
    let b_id = id::note_id(NoteType::Fact, "B");

    let a = note(NoteType::Fact, "A", &[(EdgeKind::DependsOn, &b_id)])?;
    let b = note(NoteType::Fact, "B", &[(EdgeKind::DependsOn, &a_id)])?;

    let graph = Graph::from_notes(vec![a, b])?;
    let mut expected = vec![a_id, b_id];
    expected.sort();
    assert_eq!(graph.dependency_cycles(), vec![expected.clone()]);
    assert_eq!(
        graph.cycle_members().into_iter().collect::<Vec<_>>(),
        expected
    );
    Ok(())
}
