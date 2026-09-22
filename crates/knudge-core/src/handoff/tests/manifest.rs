//! Manifest e ranking por tier (E08-T01).

use crate::handoff::RewindMode;
use crate::handoff::manifest::{TrustTier, manifest_text, rank, tier_of, trust_score};
use crate::schema::{Classification, EdgeKind, NoteType, Scope};
use crate::write::Draft;
use crate::{Error, Result};

use super::{NOW, built, classified, confirmed, note};

#[test]
fn confirmed_note_is_star() -> Result<()> {
    let note = confirmed("confirmada")?;
    let (index, graph) = built(std::slice::from_ref(&note))?;
    let items = rank(&index, &graph, &RewindMode::Manifest);
    assert_eq!(items.first().map(|item| item.tier), Some(TrustTier::Star));
    let doc = index
        .docs
        .first()
        .ok_or_else(|| Error::internal("sem doc"))?;
    assert!(trust_score(&doc.meta) >= 100.0);
    assert_eq!(tier_of(&doc.meta), TrustTier::Star);
    Ok(())
}

#[test]
fn ranking_orders_by_tier() -> Result<()> {
    let star = confirmed("estrela")?;
    let foundational = classified(
        NoteType::Decision,
        "fundacional",
        Classification::Foundational,
    )?;
    let tactical = note(NoteType::Fact, "tatica")?;
    let observational = classified(
        NoteType::Fact,
        "observacional",
        Classification::Observational,
    )?;
    let notes = [star, foundational, tactical, observational];
    let (index, graph) = built(&notes)?;
    let items = rank(&index, &graph, &RewindMode::Manifest);
    let tiers: Vec<TrustTier> = items.iter().map(|item| item.tier).collect();
    assert_eq!(
        tiers,
        vec![
            TrustTier::Star,
            TrustTier::Foundational,
            TrustTier::Tactical,
            TrustTier::Observational,
        ]
    );
    Ok(())
}

#[test]
fn manifest_counts_and_dirty() -> Result<()> {
    let notes = [note(NoteType::Fact, "a")?, note(NoteType::Decision, "b")?];
    let (index, graph) = built(&notes)?;
    let text = manifest_text(&index, &graph, &["src/x.rs".to_string()]);
    assert!(text.starts_with("notes=2"));
    assert!(text.contains("dirty"));
    assert!(text.contains("recent:"));
    Ok(())
}

#[test]
fn files_mode_filters_by_anchor() -> Result<()> {
    let anchored = super::anchored("ancorada", &["src/**"])?;
    let other = note(NoteType::Fact, "solta")?;
    let notes = [anchored, other];
    let (index, graph) = built(&notes)?;
    let items = rank(
        &index,
        &graph,
        &RewindMode::Files(vec!["src/main.rs".to_string()]),
    );
    assert_eq!(items.len(), 1);
    assert_eq!(
        items.first().map(|item| item.statement.as_str()),
        Some("ancorada")
    );
    Ok(())
}

#[test]
fn files_mode_includes_program_subtree() -> Result<()> {
    let mut child = Draft::new(NoteType::Task, "story");
    child.scope = Some(Scope::Task);
    let child = child.to_note(NOW)?;
    let child_id = child.id()?.to_string();
    let mut epic = Draft::new(NoteType::Container, "programa");
    epic.scope = Some(Scope::Epic);
    epic.anchors = vec!["plan/foo.md".to_string()];
    epic.edges = vec![(EdgeKind::ResultsIn, child_id.clone())];
    let epic = epic.to_note(NOW)?;
    let epic_id = epic.id()?.to_string();

    let notes = [epic, child];
    let (index, graph) = built(&notes)?;
    let items = rank(
        &index,
        &graph,
        &RewindMode::Files(vec!["plan/foo.md".to_string()]),
    );
    let ids: Vec<&str> = items.iter().map(|item| item.id.as_str()).collect();
    assert!(ids.contains(&epic_id.as_str()), "sem o épico: {ids:?}");
    assert!(ids.contains(&child_id.as_str()), "sem a subárvore: {ids:?}");
    Ok(())
}
