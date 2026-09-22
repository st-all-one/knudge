//! `diff` sobre eventos (E08-T05).

use crate::graph::Graph;
use crate::maintenance::diff;
use crate::store::Event;

#[test]
fn filters_by_interval_and_orders() {
    let events = vec![
        Event::new("write", 100).with_note_id("fact_00000001"),
        Event::new("update", 300).with_note_id("fact_00000002"),
        Event::new("write", 200).with_note_id("fact_00000003"),
    ];
    let graph = Graph::default();
    let entries = diff(&events, Some(150), Some(350), None, &graph);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries.first().map(|entry| entry.at), Some(200));
    assert_eq!(entries.last().map(|entry| entry.at), Some(300));
}

#[test]
fn events_without_note_are_ignored() {
    let events = vec![Event::new("maintenance", 100)];
    let graph = Graph::default();
    assert!(diff(&events, None, None, None, &graph).is_empty());
}

#[test]
fn no_bounds_returns_everything_sorted() {
    let events = vec![
        Event::new("b", 2).with_note_id("fact_00000001"),
        Event::new("a", 1).with_note_id("fact_00000001"),
    ];
    let graph = Graph::default();
    let entries = diff(&events, None, None, None, &graph);
    assert_eq!(entries.first().map(|entry| entry.op.as_str()), Some("a"));
}
