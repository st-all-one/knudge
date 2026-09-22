//! Testes da varredura de resíduos (E03-T08 / R10).

use std::path::Path;

use crate::Result;
use crate::ports::Fs;
use crate::ports::fakes::{MemFs, RecordingLogger};
use crate::store::sweep_residues;

#[test]
fn sweep_removes_only_old_residues() -> Result<()> {
    let fs = MemFs::new();
    fs.insert_at("/p/.knudge/notas/x.md.tmp", b"x", 0);
    fs.insert_at("/p/.knudge/.locks/a.lock", b"a", 8_000);
    fs.insert_at("/p/.knudge/eventos/events.jsonl", b"e", 0);
    fs.insert_at("/p/.knudge/.idx/index.json", b"{}", 0);

    let logger = RecordingLogger::default();
    let removed = sweep_residues(&fs, Path::new("/p/.knudge"), 10_000, 5_000, &logger)?;

    assert_eq!(removed, 1);
    assert!(!fs.exists(Path::new("/p/.knudge/notas/x.md.tmp")));
    assert!(
        fs.exists(Path::new("/p/.knudge/.locks/a.lock")),
        "lock fresco fica"
    );
    assert!(fs.exists(Path::new("/p/.knudge/eventos/events.jsonl")));
    assert!(logger.contains("resíduo"));
    Ok(())
}
