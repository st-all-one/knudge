//! Testes de âncoras com hash e verify-on-hit (E09-T06).

use crate::Result;
use crate::health::{AnchorRole, AnchorStore, StaleReason, invalidated_notes, refresh, verify};
use crate::ports::Fs;
use crate::ports::fakes::MemFs;

use super::{PROJECT, ROOT, anchored, seeded};

#[test]
fn refresh_records_hashes_and_verify_is_clean() -> Result<()> {
    let fs = MemFs::new();
    fs.insert("/p/src/main.rs", b"fn main() {}".to_vec());
    let note = anchored("cita main", &["src/main.rs"], "veja src/main.rs")?;
    let ctx = seeded(&fs, &[note])?;
    let store = AnchorStore::new(&fs, ROOT);

    let changed = refresh(&fs, std::path::Path::new(PROJECT), ctx.store(), &store)?;
    assert_eq!(changed, 1);
    assert!(verify(&fs, std::path::Path::new(PROJECT), &store)?.is_empty());
    Ok(())
}

#[test]
fn cited_anchor_invalidates_when_content_changes() -> Result<()> {
    let fs = MemFs::new();
    fs.insert("/p/src/main.rs", b"fn main() {}".to_vec());
    let cited = anchored("cita main", &["src/main.rs"], "veja src/main.rs")?;
    let context = anchored("contexto main", &["src/main.rs"], "")?;
    let ctx = seeded(&fs, &[cited, context])?;
    let store = AnchorStore::new(&fs, ROOT);
    let _changed = refresh(&fs, std::path::Path::new(PROJECT), ctx.store(), &store)?;

    fs.insert("/p/src/main.rs", b"fn main() { changed }".to_vec());
    let stale = verify(&fs, std::path::Path::new(PROJECT), &store)?;
    assert_eq!(stale.len(), 2);
    assert!(stale.iter().any(|a| a.role == AnchorRole::Cited));
    assert!(stale.iter().any(|a| a.role == AnchorRole::Context));
    // Só a citada invalida a nota.
    assert_eq!(invalidated_notes(&stale).len(), 1);
    Ok(())
}

#[test]
fn missing_file_is_stale() -> Result<()> {
    let fs = MemFs::new();
    fs.insert("/p/src/gone.rs", b"x".to_vec());
    let note = anchored("cita gone", &["src/gone.rs"], "veja src/gone.rs")?;
    let ctx = seeded(&fs, &[note])?;
    let store = AnchorStore::new(&fs, ROOT);
    let _changed = refresh(&fs, std::path::Path::new(PROJECT), ctx.store(), &store)?;

    fs.remove_file(std::path::Path::new("/p/src/gone.rs"))?;
    let stale = verify(&fs, std::path::Path::new(PROJECT), &store)?;
    assert_eq!(stale.first().map(|a| a.reason), Some(StaleReason::Missing));
    Ok(())
}

#[test]
fn glob_anchor_is_context() -> Result<()> {
    let note = anchored("glob", &["src/**"], "")?;
    assert_eq!(
        super::super::anchor_role(&note, "src/**"),
        AnchorRole::Context
    );
    Ok(())
}
