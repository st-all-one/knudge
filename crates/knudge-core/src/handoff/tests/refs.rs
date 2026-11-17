//! Testes das refs versionadas de handoff (D227).

use std::collections::BTreeMap;
use std::path::Path;

use proptest::prelude::*;

use crate::Result;
use crate::handoff::{RefKind, RefStatus, annotate_refs, parse_ref, render_ref, verify_refs};
use crate::health::{AnchorStore, refresh};
use crate::ports::fakes::MemFs;
use crate::schema::{NoteType, Status, Value};
use crate::store::Store;
use crate::write::Draft;

use super::NOW;

const ROOT: &str = "/p/.knudge";
const PROJECT: &str = "/p";

proptest! {
    #[test]
    fn parse_ref_roundtrips_generated_names_and_hashes(
        id in "[a-z]{3,8}_[a-f0-9]{8}",
        hash in "[a-f0-9]{8}"
    ) {
        let rendered = render_ref(&id, &hash);
        prop_assert_eq!(parse_ref(&rendered), Some((id.as_str(), hash.as_str())));
    }
}

#[test]
fn annotate_refs_leaves_default_text_unchanged() {
    let text = "fact_aaaaaaaa|afirmação|tactical\n";
    assert_eq!(
        annotate_refs(text, &BTreeMap::new(), &BTreeMap::new()),
        text
    );
}

#[test]
fn annotate_refs_adds_hashes_and_anchor_lines() {
    let mut bodies = BTreeMap::new();
    bodies.insert("fact_aaaaaaaa".to_string(), "deadbeef".to_string());
    let mut anchors = BTreeMap::new();
    anchors.insert("src/main.rs".to_string(), "cafebabe".to_string());
    let out = annotate_refs("fact_aaaaaaaa|afirmação|tactical\n", &bodies, &anchors);
    assert_eq!(
        out,
        "fact_aaaaaaaa@deadbeef|afirmação|tactical\nanchor|src/main.rs@cafebabe\n"
    );
}

#[test]
fn verify_refs_is_silent_when_everything_matches() -> Result<()> {
    let fs = MemFs::new();
    fs.insert("/p/src/main.rs", b"fn main() {}".to_vec());
    let (id, hash) = seeded_note(&fs)?;
    let store = Store::new(&fs, ROOT);
    let anchors = AnchorStore::new(&fs, ROOT);
    let text = format!("{id}@{hash}|afirmação|tactical\n");
    assert!(verify_refs(&store, &anchors, &fs, Path::new(PROJECT), &text)?.is_empty());
    Ok(())
}

#[test]
fn verify_refs_reports_changed_superseded_retired_and_dangling() -> Result<()> {
    let fs = MemFs::new();
    fs.insert("/p/src/main.rs", b"fn main() {}".to_vec());
    let (id, hash) = seeded_note(&fs)?;
    let store = Store::new(&fs, ROOT);
    let anchors = AnchorStore::new(&fs, ROOT);
    let text = format!(
        "{id}@{hash}|afirmação|tactical\n\
         fact_ffffffff@deadbeef|outra|tactical\n"
    );

    let mut note = store.read(&id)?;
    note.body = "corpo novo".to_string();
    note.refresh_body_hash()?;
    store.write(&note)?;
    let findings = verify_refs(&store, &anchors, &fs, Path::new(PROJECT), &text)?;
    assert_eq!(findings.len(), 2);
    let changed = findings
        .iter()
        .find(|finding| finding.status == RefStatus::Changed)
        .ok_or_else(|| crate::Error::internal("changed não reportado"))?;
    assert_eq!(changed.kind, RefKind::Note);
    assert_eq!(
        changed.render(),
        format!(
            "changed {id}@{hash} -> {}",
            changed.new_hash.as_deref().unwrap_or("")
        ),
        "render não pode duplicar o separador da ref"
    );
    assert!(findings.iter().any(|f| f.status == RefStatus::Dangling));

    let mut superseded = store.read(&id)?;
    superseded.frontmatter.set(
        "status",
        Value::Str(Status::Superseded.as_str().to_string()),
    )?;
    superseded
        .frontmatter
        .set("superseded_by", Value::Str("fact_11111111".to_string()))?;
    store.write(&superseded)?;
    let findings = verify_refs(&store, &anchors, &fs, Path::new(PROJECT), &text)?;
    assert!(
        findings
            .iter()
            .any(|finding| finding.status == RefStatus::Superseded
                && finding.detail.as_deref() == Some("fact_11111111"))
    );

    let mut forgotten = store.read(&id)?;
    forgotten
        .frontmatter
        .set("status", Value::Str(Status::Forgotten.as_str().to_string()))?;
    store.write(&forgotten)?;
    let findings = verify_refs(&store, &anchors, &fs, Path::new(PROJECT), &text)?;
    assert!(
        findings
            .iter()
            .any(|finding| finding.status == RefStatus::Retired)
    );
    Ok(())
}

#[test]
fn verify_refs_detects_changed_anchor() -> Result<()> {
    let fs = MemFs::new();
    fs.insert("/p/src/main.rs", b"fn main() {}".to_vec());
    let (id, hash) = seeded_note(&fs)?;
    let store = Store::new(&fs, ROOT);
    let anchors = AnchorStore::new(&fs, ROOT);
    let _changed_count = refresh(&fs, Path::new(PROJECT), &store, &anchors)?;
    let records = anchors.list()?;
    let record = records
        .first()
        .ok_or_else(|| crate::Error::internal("âncora não registrada"))?;
    let text = format!(
        "{id}@{hash}|afirmação|tactical\nanchor|{}@{}\n",
        record.path, record.content_hash
    );
    assert!(verify_refs(&store, &anchors, &fs, Path::new(PROJECT), &text)?.is_empty());

    fs.insert("/p/src/main.rs", b"fn main() { changed }".to_vec());
    let findings = verify_refs(&store, &anchors, &fs, Path::new(PROJECT), &text)?;
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == RefKind::Anchor && finding.status == RefStatus::Changed)
    );
    Ok(())
}

fn seeded_note(fs: &MemFs) -> Result<(String, String)> {
    let mut draft = Draft::new(NoteType::Fact, "afirmação").with_body("corpo");
    draft.anchors = vec!["src/main.rs".to_string()];
    let note = draft.to_note(NOW)?;
    let id = note.frontmatter.id()?.to_string();
    let hash = note
        .frontmatter
        .get("body_hash")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let store = Store::new(fs, ROOT);
    store.ensure_dirs()?;
    store.write(&note)?;
    Ok((id, hash))
}
