//! Testes da leitura tolerante (E09-T05).

use crate::Result;
use crate::graph::Graph;
use crate::health::read_tolerant;
use crate::ports::fakes::MemFs;
use crate::retrieval::{Index, RecallQuery, recall};
use crate::schema::{NoteType, Value};
use crate::store::{Note, Store};
use crate::toon;

use super::{ROOT, note};

fn store_with<'a>(fs: &'a MemFs, notes: &[Note]) -> Result<Store<'a>> {
    let store = Store::new(fs, ROOT);
    store.ensure_dirs()?;
    for note in notes {
        store.write(note)?;
    }
    Ok(store)
}

fn insert_raw(fs: &MemFs, id: &str, frontmatter: &str, body: &str) {
    let text = format!("---\n{frontmatter}---\n{body}");
    let prefix = id.split_once('_').map_or(id, |(prefix, _)| prefix);
    fs.insert(format!("{ROOT}/notas/{prefix}/{id}.md"), text.into_bytes());
}

#[test]
fn unknown_key_warns_but_reads() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, ROOT);
    store.ensure_dirs()?;
    let note = note(NoteType::Fact, "tolerante", "")?;
    let id = note.id()?.to_string();
    let mut value = note.frontmatter.to_value();
    if let Value::Map(map) = &mut value {
        map.insert("extra_field".to_string(), Value::Str("x".to_string()));
    }
    insert_raw(&fs, &id, &toon::emit(&value), "");

    let read = read_tolerant(&store)?;
    assert_eq!(read.notes.len(), 1);
    assert!(read.skipped.is_empty());
    assert!(
        read.warnings
            .iter()
            .any(|warning| warning.contains("extra_field"))
    );
    Ok(())
}

#[test]
fn malformed_note_is_skipped_with_guidance() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, ROOT);
    store.ensure_dirs()?;
    fs.insert(
        format!("{ROOT}/notas/bad/bad_00000000.md"),
        b"nao e uma nota".to_vec(),
    );

    let read = read_tolerant(&store)?;
    assert!(read.notes.is_empty());
    assert_eq!(read.skipped.len(), 1);
    assert!(
        read.skipped
            .first()
            .is_some_and(|note| !note.guidance.is_empty())
    );
    Ok(())
}

#[test]
fn unknown_type_is_skipped_per_note() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, ROOT);
    store.ensure_dirs()?;
    let front = "id: alien_00000000\ntype: alien\nstatement: hmm\ncreated_at: 2023-11-14T22:13:20.000Z\nbody_hash: 00000000\nschema_version: 1\n";
    insert_raw(&fs, "alien_00000000", front, "");

    let read = read_tolerant(&store)?;
    assert!(read.notes.is_empty());
    assert_eq!(read.skipped.len(), 1);
    assert!(
        read.skipped
            .first()
            .is_some_and(|note| note.reason.contains("tipo desconhecido"))
    );
    Ok(())
}

#[test]
fn git_conflict_markers_are_skipped() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, ROOT);
    store.ensure_dirs()?;
    // Mesma `statement` em dois clones: o git deixa marcadores de conflito (D153).
    fs.insert(
        format!("{ROOT}/notas/fact/fact_00000001.md"),
        b"<<<<<<< HEAD\n---\nid: fact_00000001\n---\ncorpo A\n=======\ncorpo B\n>>>>>>> other\n"
            .to_vec(),
    );

    let read = read_tolerant(&store)?;
    assert!(read.notes.is_empty());
    assert_eq!(read.skipped.len(), 1, "nota em conflito devia ser pulada");
    assert!(
        read.warnings
            .iter()
            .any(|warning| warning.contains("nota pulada"))
    );
    Ok(())
}

#[test]
fn recall_survives_bad_note() -> Result<()> {
    let fs = MemFs::new();
    let store = store_with(
        &fs,
        &[
            note(NoteType::Fact, "alpha importante", "")?,
            note(NoteType::Fact, "beta relevante", "")?,
        ],
    )?;
    fs.insert(
        format!("{ROOT}/notas/broken/broken_00000000.md"),
        b"quebrada".to_vec(),
    );

    let read = read_tolerant(&store)?;
    assert_eq!(read.notes.len(), 2);
    assert_eq!(read.skipped.len(), 1);

    let index = Index::build(&read.notes)?;
    let graph = Graph::from_notes(read.notes)?;
    let output = recall(&index, &graph, &RecallQuery::new("alpha"))?;
    assert_eq!(output.hits.len(), 1);
    Ok(())
}
