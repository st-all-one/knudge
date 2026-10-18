//! Testes do corpus de leitura única (E15-T02/O1).

use std::path::Path;

use crate::Result;
use crate::corpus::{Corpus, Source};
use crate::graph::Graph;
use crate::ports::Fs;
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

#[test]
fn load_notes_parallel_matches_sequential_order() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    store.ensure_dirs()?;
    // Acima de `PARALLEL_MIN_NOTES` (256): exercita a leitura paralela em máquina multicore.
    for i in 0..300u32 {
        store.write(&note(
            NoteType::Fact,
            &format!("afirmação sintética número {i}"),
        )?)?;
    }
    let ids = store.list_ids()?;
    assert_eq!(ids.len(), 300);
    let loaded = Corpus::load_notes(&store)?;
    let mut loaded_ids = Vec::with_capacity(loaded.len());
    for item in &loaded {
        loaded_ids.push(item.frontmatter.id()?.to_string());
    }
    // A ordem remontada é a de `list_ids`, idêntica à da leitura sequencial.
    assert_eq!(loaded_ids, ids);
    let mut sequential = Vec::with_capacity(ids.len());
    for id in &ids {
        if let Some(item) = store.read_optional(id)? {
            sequential.push(item.frontmatter.id()?.to_string());
        }
    }
    assert_eq!(loaded_ids, sequential);
    Ok(())
}

/// JSONL com as notas do store (o codec de `Note::to_jsonl`, D218).
fn jsonl_of(notes: &[Note]) -> Result<String> {
    let mut text = String::new();
    for note in notes {
        text.push_str(&note.to_jsonl()?);
        text.push('\n');
    }
    Ok(text)
}

#[test]
fn jsonl_source_reads_the_same_corpus_as_notas_dir() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    seed(&store)?;
    let notes = Corpus::load_notes(&store)?;
    let path = "/p/.agents/memory/notes.jsonl";
    fs.write_atomic(Path::new(path), jsonl_of(&notes)?.as_bytes())?;

    let (from_md, warnings_md) = Source::NotesDir.load(&store)?;
    let (from_jsonl, warnings_jsonl) = Source::jsonl(path).load(&store)?;
    assert!(warnings_md.is_empty());
    assert!(warnings_jsonl.is_empty());
    // Golden: a mesma memória em `notas/*.md` e em JSONL deriva o mesmo índice e grafo.
    assert_eq!(from_jsonl.index, from_md.index);
    assert_eq!(from_jsonl.graph.ids(), from_md.graph.ids());
    Ok(())
}

#[test]
fn jsonl_source_dedups_on_read_last_line_wins() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    let first = note(NoteType::Fact, "primeira afirmação")?;
    let second = note(NoteType::Fact, "segunda afirmação")?;
    let mut text = jsonl_of(&[first.clone(), second])?;
    // A primeira reaparece no fim: a última versão vence (D26).
    text.push_str(&first.to_jsonl()?);
    text.push('\n');
    let path = "/p/notes.jsonl";
    fs.write_atomic(Path::new(path), text.as_bytes())?;

    let (loaded, warnings) = Source::jsonl(path).load_notes(&store)?;
    assert!(warnings.is_empty());
    assert_eq!(loaded.len(), 2);
    let ids: Vec<&str> = loaded.iter().map(|item| item.id().unwrap_or("")).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "ordem determinística por id");
    Ok(())
}

#[test]
fn jsonl_source_skips_malformed_lines_with_warning() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    let good = note(NoteType::Fact, "afirmação válida")?;
    let text = format!("não é json\n{{}}\n{}\n", good.to_jsonl()?);
    let path = "/p/notes.jsonl";
    fs.write_atomic(Path::new(path), text.as_bytes())?;

    let (loaded, warnings) = Source::jsonl(path).load_notes(&store)?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(warnings.len(), 2);
    assert!(
        warnings
            .first()
            .is_some_and(|text| text.contains("linha 1"))
    );
    Ok(())
}

#[test]
fn jsonl_source_absent_file_is_an_empty_corpus() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    let (loaded, warnings) = Source::jsonl("/p/nada.jsonl").load_notes(&store)?;
    assert!(loaded.is_empty());
    assert!(warnings.is_empty());
    Ok(())
}
