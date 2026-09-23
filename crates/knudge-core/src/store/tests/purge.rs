//! Testes da purga do derivado em remoção (E03-T07 / D84).

use std::path::Path;

use super::sample_note;
use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::Value;
use crate::store::{Store, purge_derived};

#[test]
fn purge_removes_id_from_jsonl_and_index() -> Result<()> {
    let fs = MemFs::new();
    let id = "fact_00000001";
    let other = "fact_00000002";
    fs.insert(
        "/p/.knudge/.idx/embeddings.jsonl",
        format!("{{\"id\":\"{id}\",\"v\":[1]}}\n{{\"id\":\"{other}\",\"v\":[2]}}\n"),
    );
    fs.insert(
        "/p/.knudge/.idx/index.json",
        format!(
            "{{\"forward\":{{\"{id}\":{{}},\"{other}\":{{}}}},\"postings\":{{\"rust\":[\"{id}\",\"{other}\"]}}}}"
        ),
    );

    purge_derived(&fs, Path::new("/p/.knudge"), id)?;

    let jsonl = read(&fs, "/p/.knudge/.idx/embeddings.jsonl");
    assert!(!jsonl.contains(id));
    assert!(jsonl.contains(other));

    let index = read(&fs, "/p/.knudge/.idx/index.json");
    assert!(!index.contains(id));
    assert!(index.contains(other));
    Ok(())
}

#[test]
fn store_remove_purges_derived() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    let note = sample_note("nota purgada")?;
    store.write(&note)?;
    fs.insert(
        "/p/.knudge/.idx/embeddings.jsonl",
        format!("{{\"id\":\"{}\",\"v\":[1]}}\n", note.id()?),
    );

    store.remove(note.id()?)?;

    let jsonl = read(&fs, "/p/.knudge/.idx/embeddings.jsonl");
    assert!(!jsonl.contains(note.id()?));
    Ok(())
}

#[test]
fn store_remove_detaches_incoming_edges() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    let target = sample_note("alvo")?;
    let target_id = target.id()?.to_string();
    let mut referrer = sample_note("referente")?;
    let list =
        |ids: &[&str]| Value::List(ids.iter().map(|id| Value::Str((*id).to_string())).collect());
    referrer
        .frontmatter
        .set("depends_on", list(&[target_id.as_str()]))?;
    referrer
        .frontmatter
        .set("results_in", list(&[target_id.as_str()]))?;
    referrer
        .frontmatter
        .set("superseded_by", Value::Str(target_id.clone()))?;
    referrer.frontmatter.validate()?;
    store.write(&target)?;
    store.write(&referrer)?;

    store.remove(&target_id)?;

    let after = store.read(referrer.id()?)?;
    assert!(after.frontmatter.get("depends_on").is_none());
    assert!(after.frontmatter.get("results_in").is_none());
    assert!(after.frontmatter.get("superseded_by").is_none());
    assert_eq!(after.revision(), 2);
    Ok(())
}

fn read(fs: &MemFs, path: &str) -> String {
    String::from_utf8(fs.get(Path::new(path)).unwrap_or_default()).unwrap_or_default()
}

#[test]
fn purge_prunes_ids_inside_jsonl_lists() -> Result<()> {
    let fs = MemFs::new();
    let removed = "fact_00000001";
    let kept = "fact_00000002";
    fs.insert(
        "/p/.knudge/.idx/suggestions.jsonl",
        format!("{{\"id\":\"{kept}\",\"targets\":[\"{removed}\",\"{kept}\"]}}\n"),
    );

    purge_derived(&fs, Path::new("/p/.knudge"), removed)?;

    let jsonl = read(&fs, "/p/.knudge/.idx/suggestions.jsonl");
    assert!(jsonl.contains(kept));
    assert!(!jsonl.contains(removed));
    Ok(())
}
