//! Escrita estrita e omissão de opcionais (E07-T05).

use crate::Result;
use crate::schema::{Classification, Frontmatter, NoteType, Scope, Value};
use crate::write::Draft;

use super::NOW;

#[test]
fn unknown_key_is_rejected_with_frozen_message() {
    let mut frontmatter = Frontmatter::new();
    let error = frontmatter.set("bogus", Value::Str("x".to_string()));
    assert!(error.is_err());
    let message = error.err().map(|error| error.to_string());
    assert_eq!(
        message.as_deref(),
        Some("erro de schema: chave desconhecida: \"bogus\"")
    );
}

#[test]
fn unknown_type_is_rejected() {
    assert!("bogus".parse::<NoteType>().is_err());
    assert_eq!(
        "bogus"
            .parse::<NoteType>()
            .err()
            .map(|error| error.to_string()),
        Some("erro de schema: tipo desconhecido: \"bogus\"".to_string())
    );
}

#[test]
fn missing_required_key_is_rejected() {
    let error = Frontmatter::new().validate();
    assert!(error.is_err());
    assert_eq!(
        error.err().map(|error| error.to_string()),
        Some("erro de schema: campo obrigatório ausente: id".to_string())
    );
}

#[test]
fn empty_optionals_are_omitted() -> Result<()> {
    let draft = Draft {
        classification: Some(Classification::Foundational),
        ..Draft::new(NoteType::Fact, "alpha")
    };
    let note = draft.to_note(NOW)?;
    for key in ["tags", "source", "anchors", "checks", "evidence", "status"] {
        assert!(
            !note.frontmatter.keys().contains(&key),
            "não deve conter {key}"
        );
    }
    assert!(note.frontmatter.keys().contains(&"classification"));
    let rendered = note.render();
    assert!(!rendered.contains("tags:"));
    assert!(!rendered.contains("null"));
    Ok(())
}

#[test]
fn overlong_statement_is_rejected() {
    let draft = Draft::new(NoteType::Fact, "a".repeat(121));
    assert!(draft.to_note(NOW).is_err());
}

#[test]
fn scope_rules_are_enforced() {
    let scoped_fact = Draft {
        scope: Some(Scope::Task),
        ..Draft::new(NoteType::Fact, "alpha")
    };
    assert!(scoped_fact.to_note(NOW).is_err());

    let task_without_scope = Draft::new(NoteType::Task, "tarefa");
    assert!(task_without_scope.to_note(NOW).is_err());

    let task = Draft {
        scope: Some(Scope::Task),
        ..Draft::new(NoteType::Task, "tarefa")
    };
    assert!(task.to_note(NOW).is_ok());
}
