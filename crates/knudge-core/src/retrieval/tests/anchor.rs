//! Canal de âncoras e globs (D81/D86).

use std::collections::BTreeSet;

use proptest::prelude::*;

use crate::Result;
use crate::retrieval::anchor::{glob_match, match_note, rank};
use crate::retrieval::{Index, Meta};
use crate::schema::{Classification, NoteType, Status};

use super::{anchored, note};

#[test]
fn glob_semantics() {
    assert!(glob_match("V2/**", "V2/a/b.rs"));
    assert!(glob_match("src/*.rs", "src/main.rs"));
    assert!(!glob_match("src/*.rs", "src/a/b.rs"));
    assert!(glob_match("a?c", "abc"));
    assert!(!glob_match("a?c", "a/c"));
    assert!(glob_match("**", "qualquer/coisa"));
    assert!(glob_match("fact_00000001", "fact_00000001"));
}

#[test]
fn glob_edge_cases() {
    assert!(glob_match("**", ""));
    assert!(glob_match("*", ""));
    assert!(glob_match("", ""));
    assert!(!glob_match("", "a"));
    assert!(glob_match("a**b", "a/x/b"));
    assert!(!glob_match("a*b", "a/x/b"));
    assert!(glob_match("**/x", "a/x"));
    assert!(!glob_match("a/**", "a"));
    assert!(glob_match("a/*", "a/b"));
    assert!(!glob_match("a/*", "a/b/c"));
    assert!(glob_match("**/*.rs", "src/a/b.rs"));
    assert!(!glob_match("**/*.rs", "src/a/b.md"));
}

/// Oráculo independente (recursivo) para o proptest de equivalência.
fn reference_glob(pattern: &str, text: &str) -> bool {
    fn go(pattern: &[u8], text: &[u8]) -> bool {
        match pattern.first() {
            None => text.is_empty(),
            Some(b'*') if pattern.get(1) == Some(&b'*') => {
                go(pattern.get(2..).unwrap_or_default(), text)
                    || (!text.is_empty() && go(pattern, text.get(1..).unwrap_or_default()))
            }
            Some(b'*') => {
                go(pattern.get(1..).unwrap_or_default(), text)
                    || (!text.is_empty()
                        && text.first() != Some(&b'/')
                        && go(pattern, text.get(1..).unwrap_or_default()))
            }
            Some(b'?') => {
                !text.is_empty()
                    && text.first() != Some(&b'/')
                    && go(
                        pattern.get(1..).unwrap_or_default(),
                        text.get(1..).unwrap_or_default(),
                    )
            }
            Some(byte) => {
                !text.is_empty()
                    && text.first() == Some(byte)
                    && go(
                        pattern.get(1..).unwrap_or_default(),
                        text.get(1..).unwrap_or_default(),
                    )
            }
        }
    }
    go(pattern.as_bytes(), text.as_bytes())
}

proptest! {
    #[test]
    fn glob_matches_reference(
        pattern in "[a-z*/?]{0,6}",
        text in "[a-z/]{0,8}",
    ) {
        prop_assert_eq!(glob_match(&pattern, &text), reference_glob(&pattern, &text));
    }
}

#[test]
fn anchored_note_is_ranked_by_anchor_match() -> Result<()> {
    let anchored = anchored(NoteType::Fact, "sem termos da consulta", &["V2/**"])?;
    let anchored_id = anchored.id()?.to_string();
    let plain = note(NoteType::Fact, "outra nota", "")?;
    let index = Index::build(&[anchored, plain])?;

    let allowed: BTreeSet<String> = index.docs.iter().map(|doc| doc.meta.id.clone()).collect();
    let paths = vec!["V2/Modules/Noticias/foo.rs".to_string()];
    let ranked = rank(&index, &allowed, &paths, &[]);
    assert_eq!(
        ranked.first().map(String::as_str),
        Some(anchored_id.as_str())
    );
    Ok(())
}

#[test]
fn match_note_distinguishes_file_and_id() {
    let meta = Meta {
        id: "fact_00000001".to_string(),
        note_type: NoteType::Fact,
        scope: None,
        classification: Classification::Tactical,
        status: Status::Active,
        tags: Vec::new(),
        anchors: vec!["V2/**".to_string(), "fact_00000009".to_string()],
        created_ms: 0,
        confirmation: 0.0,
    };
    let paths = vec!["V2/x.rs".to_string()];
    let ids = vec!["fact_00000009".to_string()];
    let matched = match_note(&meta, &paths, &ids);
    assert!(matched.file);
    assert!(matched.id);
    assert_eq!(matched.count, 2);
}
