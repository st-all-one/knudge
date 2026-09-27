//! Testes da escrita (E07).

mod batch;
mod dedup;
mod idempotent;
mod lifecycle;
mod lsh;
mod outcome;
mod reconcile;
mod strict;
mod update;

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::{Claim, NoteType, claims};
use crate::store::{EventLog, Note, Store};
use crate::write::{Draft, WriteContext, merge_into};

/// Instante fixo dos testes.
pub(super) const NOW: i64 = 1_700_000_000_000;

/// Nota pronta para semear o store.
pub(super) fn note(note_type: NoteType, statement: &str, body: &str) -> Result<Note> {
    Draft::new(note_type, statement)
        .with_body(body)
        .to_note(NOW)
}

/// Contexto de escrita sobre as notas semeadas.
pub(super) fn seeded<'a>(fs: &'a MemFs, notes: &[Note]) -> Result<WriteContext<'a>> {
    let store = Store::new(fs, "/p/.knudge");
    store.ensure_dirs()?;
    for note in notes {
        store.write(note)?;
    }
    let events = EventLog::new(fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    let index = Index::build(notes)?;
    Ok(WriteContext::new(store, events, index, NOW))
}

#[test]
fn merge_unions_claims_without_duplicating() -> Result<()> {
    let fs = MemFs::new();
    let mut first = Draft::new(
        NoteType::Fact,
        "servidor de embeddings escuta na porta 8889",
    );
    first.claims = vec![Claim::new("embeddings", "porta", "8889")];
    let seeded_note = first.to_note(NOW)?;
    let id = seeded_note.id()?.to_string();
    let ctx = seeded(&fs, &[seeded_note])?;

    let mut second = Draft::new(NoteType::Fact, "outro fato sobre embeddings");
    second.claims = vec![
        Claim::new("embeddings", "porta", "8889"),
        Claim::new("embeddings", "porta", "9999"),
    ];
    let incoming = second.to_note(NOW)?;
    let _revision = merge_into(&ctx, &id, &incoming)?;

    let merged = ctx.store().read(&id)?;
    let mut claims = claims(&merged.frontmatter)?;
    claims.sort();
    assert_eq!(claims.len(), 2, "tripla repetida não duplica: {claims:?}");
    let objects: Vec<&str> = claims.iter().map(|claim| claim.object.as_str()).collect();
    assert_eq!(objects, vec!["8889", "9999"]);
    Ok(())
}
