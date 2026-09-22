//! Testes do flush coalescido (E11-T06).

use crate::embeddings::FlushState;

#[test]
fn clean_state_never_flushes() {
    let mut flush = FlushState::new(1000);
    assert!(!flush.is_dirty());
    assert!(!flush.take(2000, 100));
}

#[test]
fn debounce_respects_interval() {
    let mut flush = FlushState::new(1000);
    flush.mark_dirty();
    assert!(!flush.should_flush(1050, 100));
    assert!(flush.should_flush(1100, 100));
    assert!(flush.take(1100, 100));
    assert!(!flush.is_dirty());
}

#[test]
fn force_flushes_immediately() {
    let mut flush = FlushState::new(1000);
    flush.mark_dirty();
    flush.force(1001);
    assert!(!flush.is_dirty());
    assert!(!flush.should_flush(1001, 0));
}

#[test]
fn burst_flushes_once() {
    let mut flush = FlushState::new(1000);
    for _ in 0..20 {
        flush.mark_dirty();
    }
    assert!(flush.take(1100, 100));
    assert!(!flush.take(1200, 100));
}
