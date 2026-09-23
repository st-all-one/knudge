//! Escrita em lote a partir de JSONL (K4/D110).

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::NoteType;
use crate::write::{BatchMode, DedupThresholds, WriteAction, batch_jsonl};

use super::{NOW, seeded};

fn thresholds() -> Result<DedupThresholds> {
    DedupThresholds::new(0.75, 0.92)
}

#[test]
fn batch_creates_valid_items_in_order() -> Result<()> {
    let fs = MemFs::new();
    let ctx = seeded(&fs, &[])?;
    let source = concat!(
        "{\"type\":\"fact\",\"statement\":\"primeira nota\",\"tags\":[\"a\"]}\n",
        "{\"type\":\"decision\",\"statement\":\"segunda nota\",\"anchors\":[\"src/x.ts\"]}\n",
    );

    let out = batch_jsonl(&ctx, source, &thresholds()?, BatchMode::Apply);
    assert!(out.warnings.is_empty(), "avisos: {:?}", out.warnings);
    assert_eq!(out.items.len(), 2);
    assert_eq!(
        out.items.first().map(|item| item.action),
        Some(WriteAction::Created)
    );
    assert!(
        out.items
            .first()
            .is_some_and(|item| item.id.starts_with("fact_"))
    );
    assert!(
        out.items
            .get(1)
            .is_some_and(|item| item.id.starts_with("decision_"))
    );
    Ok(())
}

#[test]
fn batch_skips_invalid_item_and_continues() -> Result<()> {
    let fs = MemFs::new();
    let ctx = seeded(&fs, &[])?;
    let source = concat!(
        "{\"type\":\"fact\",\"statement\":\"válida\"}\n",
        "não é json\n",
        "{\"type\":\"inexistente\",\"statement\":\"tipo ruim\"}\n",
        "{\"type\":\"fact\",\"statement\":\"outra válida\"}\n",
    );

    let out = batch_jsonl(&ctx, source, &thresholds()?, BatchMode::Apply);
    assert_eq!(out.items.len(), 2, "itens: {:?}", out.items);
    assert_eq!(out.warnings.len(), 2, "avisos: {:?}", out.warnings);
    assert!(
        out.warnings
            .first()
            .is_some_and(|w| w.starts_with("linha 2:"))
    );
    assert!(
        out.warnings
            .get(1)
            .is_some_and(|w| w.starts_with("linha 3:"))
    );
    Ok(())
}

#[test]
fn batch_dry_run_does_not_write() -> Result<()> {
    let fs = MemFs::new();
    let ctx = seeded(&fs, &[])?;
    let source = "{\"type\":\"fact\",\"statement\":\"efêmera\"}\n";

    let out = batch_jsonl(&ctx, source, &thresholds()?, BatchMode::DryRun);
    assert_eq!(out.items.len(), 1);
    assert_eq!(
        out.items.first().map(|item| item.action),
        Some(WriteAction::Created)
    );
    let id = out
        .items
        .first()
        .map(|item| item.id.clone())
        .unwrap_or_default();
    assert!(!ctx.store().exists(&id), "dry-run gravou {id}");
    Ok(())
}

#[test]
fn batch_dry_run_reports_unchanged_for_existing() -> Result<()> {
    let existing = super::note(NoteType::Fact, "já existe", "")?;
    let id = existing.id()?.to_string();
    let fs = MemFs::new();
    let ctx = seeded(&fs, &[existing])?;
    let source = "{\"type\":\"fact\",\"statement\":\"já existe\"}\n";

    let out = batch_jsonl(&ctx, source, &thresholds()?, BatchMode::DryRun);
    assert_eq!(
        out.items.first().map(|item| item.action),
        Some(WriteAction::Unchanged)
    );
    assert_eq!(
        out.items.first().map(|item| item.id.as_str()),
        Some(id.as_str())
    );
    assert_eq!(NOW, ctx.now_ms());
    Ok(())
}
