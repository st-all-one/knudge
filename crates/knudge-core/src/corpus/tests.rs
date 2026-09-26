//! Testes do corpus de leitura única (E15-T02/O1).

use crate::Result;
use crate::corpus::Corpus;
use crate::graph::Graph;
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::NoteType;
use crate::store::{Note, Store};
use crate::write::Draft;

const NOW: i64 = 1_700_000_000_000;

fn note(note_type: NoteType, statement: &str) -> Result<Note> {
    Draft::new(note_type, statement).to_note(NOW)
}

/// Semeia o store com um corpus pequeno e variado.
fn seed(store: &Store<'_>) -> Result<()> {
    store.ensure_dirs()?;
    for note in [
        note(NoteType::Fact, "o parser TOON é byte exato")?,
        note(NoteType::Decision, "adotamos BTreeMap para determinismo")?,
        note(NoteType::Fact, "otimizar o corpus de leitura")?,
    ] {
        store.write(&note)?;
    }
    Ok(())
}

#[test]
fn load_matches_separate_reads() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    seed(&store)?;
    let corpus = Corpus::load(&store)?;
    let index = Index::from_store(&store)?;
    let graph = Graph::build(&store)?;
    assert_eq!(corpus.index, index);
    assert_eq!(corpus.graph.ids(), graph.ids());
    assert_eq!(corpus.notes.len(), corpus.index.docs.len());
    Ok(())
}

#[test]
fn from_notes_ref_equals_from_notes() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    seed(&store)?;
    let notes = Corpus::load(&store)?.notes;
    let borrowed = Graph::from_notes_ref(&notes)?;
    let owned = Graph::from_notes(notes)?;
    assert_eq!(borrowed.ids(), owned.ids());
    Ok(())
}

#[test]
fn from_notes_derives_same_corpus() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    seed(&store)?;
    let loaded = Corpus::load(&store)?;
    let rebuilt = Corpus::from_notes(loaded.notes.clone())?;
    assert_eq!(rebuilt.index, loaded.index);
    assert_eq!(rebuilt.graph.ids(), loaded.graph.ids());
    Ok(())
}

#[test]
fn load_fresh_rebuilds_and_then_reuses() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    seed(&store)?;
    let (first, warnings) = Corpus::load_fresh(&store, &fs, store.root())?;
    assert!(
        warnings.iter().any(|w| w.contains("reconstruído")),
        "avisos: {warnings:?}"
    );
    let (again, warnings) = Corpus::load_fresh(&store, &fs, store.root())?;
    assert_eq!(again.index, first.index);
    assert_eq!(again.notes.len(), first.notes.len());
    assert!(warnings.is_empty(), "avisos: {warnings:?}");
    Ok(())
}

#[test]
fn load_fresh_rebuilds_when_a_note_is_newer() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    seed(&store)?;
    let (first, _) = Corpus::load_fresh(&store, &fs, store.root())?;
    // Nota nova com `mtime` posterior ao índice ⇒ o índice persistido fica obsoleto.
    let extra = note(NoteType::Fact, "nota posterior ao indice")?;
    let path = store.note_path(extra.id()?);
    store.write(&extra)?;
    let Some(data) = fs.get(&path) else {
        return Ok(());
    };
    fs.insert_at(path, data, 10_000);
    let (again, _) = Corpus::load_fresh(&store, &fs, store.root())?;
    assert_eq!(again.notes.len(), first.notes.len().saturating_add(1));
    assert!(again.index.docs.len() > first.index.docs.len());
    Ok(())
}

#[test]
fn load_fresh_rebuilds_on_corrupt_index() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    seed(&store)?;
    let (first, _) = Corpus::load_fresh(&store, &fs, store.root())?;
    fs.insert_at(Index::path(store.root()), b"{nao eh json".to_vec(), 10_000);
    let (again, warnings) = Corpus::load_fresh(&store, &fs, store.root())?;
    assert_eq!(again.index, first.index);
    assert!(
        warnings.iter().any(|w| w.contains("ilegível")),
        "avisos: {warnings:?}"
    );
    Ok(())
}
