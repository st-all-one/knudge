//! Testes do drift de âncoras (E19/T01b/D203).

use std::collections::BTreeMap;

use crate::Result;
use crate::lifecycle::decay::AnchorValidity;
use crate::lifecycle::drift::{DriftEntry, DriftIndex, DriftStore, entries_from_validity};
use crate::ports::fakes::MemFs;

use super::ROOT;

#[test]
fn entries_from_validity_maps_broken_fraction() {
    let mut validity: BTreeMap<String, AnchorValidity> = BTreeMap::new();
    let _ignored = validity.insert(
        "a".to_string(),
        AnchorValidity {
            total: 4,
            valid: 1,
            broken: 3,
        },
    );
    let _ignored = validity.insert("b".to_string(), AnchorValidity::default());
    let entries = entries_from_validity(&validity);
    let index = DriftIndex::new(&entries);
    assert!((index.get("a") - 0.75).abs() < 1e-9);
    assert!(index.get("b").abs() < 1e-9, "sem âncoras ⇒ sem drift");
    assert!(
        index.get("ausente").abs() < 1e-9,
        "ausente ⇒ 0 (degradação graciosa)"
    );
}

#[test]
fn store_round_trips_and_absent_is_noop() -> Result<()> {
    let fs = MemFs::default();
    let store = DriftStore::new(&fs, ROOT);
    assert!(store.index()?.is_empty(), "arquivo ausente ⇒ índice vazio");
    let entries = vec![
        DriftEntry {
            id: "a".to_string(),
            drift: 0.5,
        },
        DriftEntry {
            id: "b".to_string(),
            drift: 1.0,
        },
    ];
    store.persist(&entries)?;
    let index = store.index()?;
    assert!((index.get("a") - 0.5).abs() < 1e-9);
    assert!(
        (index.get("b") - 1.0).abs() < 1e-9,
        "1.0 sobrevive ao round-trip"
    );
    Ok(())
}
