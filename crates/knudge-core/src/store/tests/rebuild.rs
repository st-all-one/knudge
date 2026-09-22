//! Testes do rebuild double-buffer (E03-T06).

use std::path::Path;

use crate::Result;
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use crate::store::Staging;

fn read_index(fs: &MemFs) -> Option<Vec<u8>> {
    fs.get(Path::new("/p/.knudge/.idx/index.json"))
}

#[test]
fn reader_sees_old_or_new_never_partial() -> Result<()> {
    let fs = MemFs::new();
    fs.insert("/p/.knudge/.idx/index.json", b"old");
    assert_eq!(read_index(&fs).as_deref(), Some(b"old".as_slice()));

    let staging = Staging::begin(&fs, "/p/.knudge/.idx")?;
    staging.write("index.json", b"new")?;
    // Antes do commit, o leitor ainda vê o índice antigo.
    assert_eq!(read_index(&fs).as_deref(), Some(b"old".as_slice()));

    staging.commit()?;
    assert_eq!(read_index(&fs).as_deref(), Some(b"new".as_slice()));
    // O buffer novo e o antigo não devem sobrar.
    assert!(!fs.exists(Path::new("/p/.knudge/.idx.new")));
    assert!(!fs.exists(Path::new("/p/.knudge/.idx.old")));
    Ok(())
}

#[test]
fn crash_during_rebuild_keeps_old_index() -> Result<()> {
    use crate::ports::fakes::FaultyFs;

    let fs = FaultyFs::new(MemFs::new());
    fs.inner().insert("/p/.knudge/.idx/index.json", b"old");
    fs.fail_writes_containing(".idx.new");

    let staging = Staging::begin(&fs, "/p/.knudge/.idx")?;
    let failed = staging.write("index.json", b"new");
    assert!(failed.is_err(), "a escrita no buffer novo deve falhar");
    drop(staging);

    // Um crash no meio do rebuild preserva o índice canônico antigo.
    assert_eq!(read_index(fs.inner()).as_deref(), Some(b"old".as_slice()));
    Ok(())
}
