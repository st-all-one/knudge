//! Testes da nota, listagem e `revision` (E03-T01/T05).

use super::sample_note;
use crate::Result;
use crate::ports::fakes::MemFs;
use crate::store::{Note, Store};

#[test]
fn note_round_trips_byte_exact() -> Result<()> {
    let note = sample_note("Rust é seguro")?;
    let rendered = note.render();
    assert!(rendered.starts_with("---\n"));
    assert!(rendered.contains("\n---\n"));
    let parsed = Note::parse(rendered.as_bytes())?;
    assert_eq!(parsed, note);
    Ok(())
}

#[test]
fn write_read_list_and_remove() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    let first = sample_note("primeira")?;
    let second = sample_note("segunda")?;
    store.write(&first)?;
    store.write(&second)?;

    assert_eq!(store.read(first.id()?)?.id()?, first.id()?);
    assert_eq!(store.list_ids()?.len(), 2);

    store.remove(first.id()?)?;
    assert!(!store.exists(first.id()?));
    assert_eq!(store.list_ids()?, vec![second.id()?.to_string()]);
    Ok(())
}

#[test]
fn update_increments_revision() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    let note = sample_note("contador de versões")?;
    store.write(&note)?;
    assert_eq!(store.read(note.id()?)?.revision(), 1);

    let first = store.update(note.id()?, |item| {
        item.body.push('!');
        Ok(())
    })?;
    let second = store.update(note.id()?, |item| {
        item.body.push('!');
        Ok(())
    })?;
    assert_eq!(first, 2);
    assert_eq!(second, 3);
    Ok(())
}
