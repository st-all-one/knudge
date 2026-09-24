//! Layout material por tipo (`notas/<tipo>/<id>.md` — D150).

use std::path::Path;

use crate::Result;
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use crate::store::Store;

use super::sample_note;

const ROOT: &str = "/p/.knudge";

#[test]
fn note_path_is_derived_from_type_prefix() {
    let fs = MemFs::new();
    let store = Store::new(&fs, ROOT);
    assert_eq!(
        store.note_path("fact_7a3c1b2d"),
        Path::new("/p/.knudge/notas/fact/fact_7a3c1b2d.md")
    );
    assert_eq!(
        store.note_path("epic_c9a3e2f4"),
        Path::new("/p/.knudge/notas/epic/epic_c9a3e2f4.md")
    );
    // Prefixo histórico (D149) vive em `epic/`.
    assert_eq!(
        store.note_path("container_c9a3e2f4"),
        Path::new("/p/.knudge/notas/epic/container_c9a3e2f4.md")
    );
}

#[test]
fn write_and_list_round_trip_in_type_dirs() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, ROOT);
    let note = sample_note("layout por tipo")?;
    let note_id = note.id()?.to_string();
    store.write(&note)?;
    assert!(fs.exists(&store.note_path(&note_id)));
    assert_eq!(store.list_ids()?, vec![note_id]);
    Ok(())
}

#[test]
fn flat_legacy_note_is_still_listed() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, ROOT);
    store.ensure_dirs()?;
    let note = sample_note("nota plana legada")?;
    let note_id = note.id()?.to_string();
    fs.insert(
        format!("{ROOT}/notas/{note_id}.md"),
        note.render().into_bytes(),
    );
    assert_eq!(store.list_ids()?, vec![note_id]);
    Ok(())
}

#[test]
fn map_md_at_root_is_not_a_note() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, ROOT);
    store.ensure_dirs()?;
    fs.insert(format!("{ROOT}/notas/MAP.md"), b"# Mapa".to_vec());
    assert!(store.list_ids()?.is_empty());
    Ok(())
}
