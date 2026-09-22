//! `context_id` e handoff 1:1 (E08-T04).

use crate::Result;
use crate::handoff::context::{ContextStore, derive_id, is_valid_context_id};
use crate::ports::fakes::MemFs;

#[test]
fn derive_id_is_deterministic() {
    assert_eq!(derive_id("hello"), derive_id("hello"));
    assert_ne!(derive_id("hello"), derive_id("world"));
    assert!(is_valid_context_id(&derive_id("hello")));
    assert!(!is_valid_context_id("ctx_short"));
    assert!(!is_valid_context_id("nope"));
}

#[test]
fn save_and_load_roundtrip() -> Result<()> {
    let fs = MemFs::new();
    let store = ContextStore::new(&fs, "/p/.knudge");
    let id = derive_id("texto");
    assert_eq!(store.load(&id)?, None);
    store.save(&id, "texto")?;
    assert_eq!(store.load(&id)?, Some("texto".to_string()));
    assert!(store.path(&id).starts_with("/p/.knudge/.idx/contexts"));
    Ok(())
}
