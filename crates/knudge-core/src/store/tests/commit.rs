//! Testes de ordem de commit e crash-injection (E03-T02).

use std::path::Path;

use super::sample_note;
use crate::Result;
use crate::ports::Fs;
use crate::ports::fakes::{FaultyFs, MemFs};
use crate::store::{Event, EventLog, Store, commit};

#[test]
fn crash_between_note_and_event_leaves_note_without_event() -> Result<()> {
    let fs = FaultyFs::new(MemFs::new());
    let root = "/p/.knudge";
    let store = Store::new(&fs, root);
    let events = EventLog::new(&fs, root, EventLog::DEFAULT_MAX_BYTES);
    let note = sample_note("nota antes do evento")?;
    let event = Event::new("write", 1).with_note_id(note.id()?);

    // Injeta falha na escrita do evento (mas não da nota).
    fs.fail_writes_containing("/eventos/");
    let failed = commit(&store, &events, &note, &event);
    assert!(failed.is_err());
    assert!(store.exists(note.id()?), "a nota deve ter sido gravada");
    assert!(
        events.read_all()?.0.is_empty(),
        "nenhum evento deve existir"
    );

    // Sem a falha, o commit completa e o evento aponta para uma nota existente.
    fs.clear();
    commit(&store, &events, &note, &event)?;
    let (recorded, warnings) = events.read_all()?;
    assert!(warnings.is_empty());
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        recorded.first().and_then(|e| e.note_id.as_deref()),
        Some(note.id()?)
    );
    Ok(())
}

#[test]
fn crashed_atomic_write_keeps_old_file() -> Result<()> {
    let fs = FaultyFs::new(MemFs::new());
    let path = Path::new("/p/.knudge/notas/x.md");
    fs.inner().insert(path, b"old");

    // Falha entre a escrita do `*.tmp` e o `rename`: o arquivo antigo permanece.
    fs.fail_writes_containing(".tmp");
    assert!(fs.write_atomic(path, b"new").is_err());
    assert_eq!(fs.read(path)?.as_slice(), b"old".as_slice());

    fs.clear();
    fs.write_atomic(path, b"new")?;
    assert_eq!(fs.read(path)?.as_slice(), b"new".as_slice());
    Ok(())
}
