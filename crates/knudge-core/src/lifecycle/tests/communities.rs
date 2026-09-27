//! Testes de comunidades (`GraphRAG` — E19-T05/D193).

use crate::Result;
use crate::lifecycle::{communities, communities_filtered, global_terms};
use crate::retrieval::Filter;
use crate::schema::{EdgeKind, NoteType, id};
use crate::store::Note;
use crate::write::Draft;

use super::{NOW, built};

/// Nota com arestas explícitas.
fn linked(statement: &str, edges: &[(EdgeKind, &str)]) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    for (kind, target) in edges {
        draft.edges.push((*kind, (*target).to_string()));
    }
    draft.to_note(NOW)
}

/// Nota com tags.
fn tagged(statement: &str, tags: &[&str]) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    draft.tags = tags.iter().map(|tag| (*tag).to_string()).collect();
    draft.to_note(NOW)
}

/// Nota com âncoras.
fn anchored(statement: &str, anchors: &[&str]) -> Result<Note> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    draft.anchors = anchors.iter().map(|anchor| (*anchor).to_string()).collect();
    draft.to_note(NOW)
}

/// Três notas em triângulo (`a→b→c→a`) + ponte para o segundo triângulo.
fn two_triangles() -> Result<Vec<Note>> {
    let a = id::note_id(NoteType::Fact, "alpha");
    let b = id::note_id(NoteType::Fact, "beta");
    let c = id::note_id(NoteType::Fact, "gamma");
    let d = id::note_id(NoteType::Fact, "delta");
    let e = id::note_id(NoteType::Fact, "epsilon");
    let f = id::note_id(NoteType::Fact, "zeta");
    Ok(vec![
        linked("alpha", &[(EdgeKind::References, &b)])?,
        linked("beta", &[(EdgeKind::References, &c)])?,
        linked(
            "gamma",
            &[(EdgeKind::References, &a), (EdgeKind::References, &d)],
        )?,
        linked("delta", &[(EdgeKind::References, &e)])?,
        linked("epsilon", &[(EdgeKind::References, &f)])?,
        linked("zeta", &[(EdgeKind::References, &d)])?,
    ])
}

#[test]
fn explicit_edges_form_two_communities() -> Result<()> {
    let (index, graph) = built(&two_triangles()?)?;
    let result = communities(&index, &graph);
    assert_eq!(result.len(), 2, "esperava 2 comunidades: {result:?}");
    assert!(result.iter().all(|community| community.members.len() == 3));
    Ok(())
}

#[test]
fn the_detection_is_deterministic() -> Result<()> {
    let (index, graph) = built(&two_triangles()?)?;
    assert_eq!(communities(&index, &graph), communities(&index, &graph));
    Ok(())
}

#[test]
fn shared_anchors_form_a_community() -> Result<()> {
    let notes = vec![
        anchored("um", &["src/a.rs"])?,
        anchored("dois", &["src/a.rs"])?,
        anchored("três", &["src/a.rs"])?,
    ];
    let (index, graph) = built(&notes)?;
    let result = communities(&index, &graph);
    assert_eq!(result.len(), 1, "âncora comum devia unir: {result:?}");
    assert_eq!(
        result.first().map(|community| community.members.len()),
        Some(3)
    );
    Ok(())
}

#[test]
fn unrelated_notes_are_singletons() -> Result<()> {
    let notes = vec![tagged("um", &["x"])?, tagged("dois", &["y"])?];
    let (index, graph) = built(&notes)?;
    let result = communities(&index, &graph);
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|community| community.members.len() == 1));
    Ok(())
}

#[test]
fn the_filter_restricts_the_partition() -> Result<()> {
    let notes = vec![
        tagged("alpha", &["x"])?,
        tagged("beta", &["x"])?,
        tagged("gamma", &["y"])?,
    ];
    let (index, graph) = built(&notes)?;
    let filter = Filter {
        tags: vec!["x".to_string()],
        ..Filter::default()
    };
    let result = communities_filtered(&index, &graph, &filter);
    let members: usize = result.iter().map(|community| community.members.len()).sum();
    assert_eq!(members, 2, "só as notas com tag `x`: {result:?}");
    Ok(())
}

#[test]
fn the_summary_ranks_repeated_terms() -> Result<()> {
    let notes = vec![
        anchored("cache de índice", &["src/a.rs"])?,
        anchored("índice de cache", &["src/a.rs"])?,
    ];
    let (index, graph) = built(&notes)?;
    let result = communities(&index, &graph);
    assert_eq!(result.len(), 1);
    let Some(community) = result.first() else {
        return Ok(());
    };
    let terms = &community.terms;
    assert!(
        terms.iter().any(|term| term == "cache" || term == "indice"),
        "resumo sem termo esperado: {terms:?}"
    );
    Ok(())
}

#[test]
fn global_terms_rank_by_frequency() -> Result<()> {
    let notes = vec![
        tagged("cache lru", &["x"])?,
        tagged("cache lfu", &["x"])?,
        tagged("fila", &["x"])?,
    ];
    let (index, _graph) = built(&notes)?;
    let terms = global_terms(&index, &Filter::new());
    assert_eq!(
        terms.first().map(String::as_str),
        Some("cache"),
        "{terms:?}"
    );
    Ok(())
}
