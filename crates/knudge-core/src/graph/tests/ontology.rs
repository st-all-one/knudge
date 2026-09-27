//! Testes da inferência de ontologia leve (E19-T09/D207).

use super::{frontmatter, note};
use crate::Result;
use crate::graph::{
    ClaimConflict, Graph, broader_ancestors, claim_conflicts, equivalence_classes,
    has_hierarchy_cycle, narrower_descendants,
};
use crate::schema::{Claim, EdgeKind, NoteType, claims_to_value, id};
use crate::store::Note;

fn node_id(statement: &str) -> String {
    id::note_id(NoteType::Fact, statement)
}

/// Copia e ordena ids para comparar com as clausuras (que saem em ordem canônica).
fn sorted(ids: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = ids.iter().map(|id| (*id).to_string()).collect();
    out.sort();
    out
}

#[test]
fn same_as_forms_equivalence_classes() -> Result<()> {
    let a = node_id("entidade a");
    let b = node_id("entidade b");
    let c = node_id("entidade c");
    let notes = vec![
        note(NoteType::Fact, "entidade a", &[(EdgeKind::SameAs, &b)])?,
        note(NoteType::Fact, "entidade b", &[(EdgeKind::SameAs, &c)])?,
        note(NoteType::Fact, "entidade c", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    let classes = equivalence_classes(&graph);
    let representative = classes.get(&a).cloned();
    assert_eq!(representative, classes.get(&b).cloned());
    assert_eq!(representative, classes.get(&c).cloned());
    // Representante = menor id da classe.
    let smallest = [a.as_str(), b.as_str(), c.as_str()].into_iter().min();
    assert_eq!(representative.as_deref(), smallest);
    Ok(())
}

#[test]
fn broader_is_transitive_and_narrower_is_its_inverse() -> Result<()> {
    let a = node_id("conceito a");
    let b = node_id("conceito b");
    let c = node_id("conceito c");
    // a ⊂ b ⊂ c  (a é mais estreito).
    let notes = vec![
        note(NoteType::Fact, "conceito a", &[(EdgeKind::Broader, &b)])?,
        note(NoteType::Fact, "conceito b", &[(EdgeKind::Broader, &c)])?,
        note(NoteType::Fact, "conceito c", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    assert_eq!(broader_ancestors(&graph, &a), sorted(&[&b, &c]));
    assert_eq!(narrower_descendants(&graph, &c), sorted(&[&a, &b]));
    assert!(broader_ancestors(&graph, &c).is_empty());
    assert!(!has_hierarchy_cycle(&graph));
    Ok(())
}

#[test]
fn narrower_declaration_feeds_broader_ancestors() -> Result<()> {
    let a = node_id("geral a");
    let b = node_id("especifico b");
    // a --narrower--> b  ⇒  a é ancestral de b.
    let notes = vec![
        note(NoteType::Fact, "geral a", &[(EdgeKind::Narrower, &b)])?,
        note(NoteType::Fact, "especifico b", &[])?,
    ];
    let graph = Graph::from_notes(notes)?;
    assert_eq!(broader_ancestors(&graph, &b), sorted(&[&a]));
    assert_eq!(narrower_descendants(&graph, &a), sorted(&[&b]));
    Ok(())
}

#[test]
fn hierarchy_cycle_is_detected() -> Result<()> {
    let a = node_id("ciclo a");
    let b = node_id("ciclo b");
    let notes = vec![
        note(NoteType::Fact, "ciclo a", &[(EdgeKind::Broader, &b)])?,
        note(NoteType::Fact, "ciclo b", &[(EdgeKind::Broader, &a)])?,
    ];
    let graph = Graph::from_notes(notes)?;
    assert!(has_hierarchy_cycle(&graph));
    Ok(())
}

#[test]
fn claim_conflicts_find_divergent_objects() -> Result<()> {
    let mut a = frontmatter(NoteType::Fact, "o servidor usa 8889")?;
    a.set(
        "claims",
        claims_to_value(&[Claim::new("embeddings", "porta", "8889")]),
    )?;
    let mut b = frontmatter(NoteType::Fact, "o servidor usa 9999")?;
    b.set(
        "claims",
        claims_to_value(&[Claim::new("embeddings", "porta", "9999")]),
    )?;
    let mut c = frontmatter(NoteType::Fact, "confirma a porta")?;
    c.set(
        "claims",
        claims_to_value(&[Claim::new("embeddings", "porta", "8889")]),
    )?;
    let notes = vec![Note::new(a, ""), Note::new(b, ""), Note::new(c, "")];
    let conflicts = claim_conflicts(&notes)?;
    assert_eq!(
        conflicts,
        vec![ClaimConflict {
            subject: "embeddings".to_string(),
            relation: "porta".to_string(),
            objects: vec!["8889".to_string(), "9999".to_string()],
        }]
    );
    Ok(())
}

#[test]
fn claims_agreeing_do_not_conflict() -> Result<()> {
    let mut a = frontmatter(NoteType::Fact, "usa 8889")?;
    a.set(
        "claims",
        claims_to_value(&[Claim::new("porta", "valor", "8889")]),
    )?;
    let mut b = frontmatter(NoteType::Fact, "tambem 8889")?;
    b.set(
        "claims",
        claims_to_value(&[Claim::new("porta", "valor", "8889")]),
    )?;
    let notes = vec![Note::new(a, ""), Note::new(b, "")];
    assert!(claim_conflicts(&notes)?.is_empty());
    Ok(())
}

#[test]
fn ontology_edges_do_not_disturb_a_plain_graph() -> Result<()> {
    let fm = frontmatter(NoteType::Fact, "sozinha")?;
    let only = fm.id()?.to_string();
    let graph = Graph::from_notes(vec![Note::new(fm, "")])?;
    let classes = equivalence_classes(&graph);
    assert_eq!(classes.get(&only).map(String::as_str), Some(only.as_str()));
    assert!(broader_ancestors(&graph, &only).is_empty());
    assert!(!has_hierarchy_cycle(&graph));
    Ok(())
}
