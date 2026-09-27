//! Testes dos slots mínimos por espécie (D191).

use crate::schema::NoteType;

use super::{expected_slots, missing_slots};

#[test]
fn decision_requires_alternatives_rationale_and_consequence() {
    let missing = missing_slots(NoteType::Decision, "só uma decisão seca", 0, 0);
    assert_eq!(missing, vec!["alternativas", "por quê", "consequência"]);
}

#[test]
fn accented_and_plain_headers_match() {
    let body = "## Alternativas\n- a\n\n## Porque\nx\n\n## Consequencia\ny\n";
    assert!(missing_slots(NoteType::Decision, body, 0, 0).is_empty());
}

#[test]
fn english_aliases_match() {
    let body = "## Alternatives\n- a\n\nWhy: x\n\nImpact: y\n";
    assert!(missing_slots(NoteType::Decision, body, 0, 0).is_empty());
}

#[test]
fn word_boundaries_are_respected() {
    // `AlternativasXYZ` não é o slot `alternativas`.
    let body = "## AlternativasXYZ\n\nPor quê: x\n\nConsequência: y\n";
    let missing = missing_slots(NoteType::Decision, body, 0, 0);
    assert_eq!(missing, vec!["alternativas"]);
}

#[test]
fn error_requires_cause_and_fix() {
    let missing = missing_slots(NoteType::Error, "## Causa\nfaltou X", 0, 0);
    assert_eq!(missing, vec!["correção"]);
    let complete = "## Causa\nfaltou X\n\n## Correção\ntrocar Y\n";
    assert!(missing_slots(NoteType::Error, complete, 0, 0).is_empty());
}

#[test]
fn risk_requires_probability_and_impact() {
    let missing = missing_slots(NoteType::Risk, "## Probabilidade\n40%", 0, 0);
    assert_eq!(missing, vec!["impacto"]);
}

#[test]
fn def_requires_meaning() {
    assert_eq!(missing_slots(NoteType::Def, "", 0, 0), vec!["significado"]);
}

#[test]
fn snippet_requires_language_and_anchor() {
    let body = "```rust\nlet x = 1;\n```\nLinguagem: rust\n";
    assert_eq!(missing_slots(NoteType::Snippet, body, 0, 0), vec!["âncora"]);
    assert!(missing_slots(NoteType::Snippet, body, 1, 0).is_empty());
}

#[test]
fn snippet_fenced_language_satisfies_the_slot() {
    let body = "```rust\nlet x = 1;\n```\n";
    assert_eq!(missing_slots(NoteType::Snippet, body, 0, 0), vec!["âncora"]);
    assert!(missing_slots(NoteType::Snippet, body, 1, 0).is_empty());
}

#[test]
fn snippet_bare_fence_does_not_satisfy_language() {
    let body = "```\nlet x = 1;\n```\n";
    assert_eq!(
        missing_slots(NoteType::Snippet, body, 1, 0),
        vec!["linguagem"]
    );
}

#[test]
fn fact_has_no_slot_contract() {
    // `fact` sem lastro é responsabilidade do D162, não deste contrato.
    assert!(missing_slots(NoteType::Fact, "", 0, 0).is_empty());
    assert!(missing_slots(NoteType::Fact, "corpo qualquer", 0, 0).is_empty());
}

#[test]
fn question_requires_dependency() {
    assert_eq!(
        missing_slots(NoteType::Question, "", 0, 0),
        vec!["o que falta (âncora ou depends_on)"]
    );
    assert!(missing_slots(NoteType::Question, "", 0, 1).is_empty());
}

#[test]
fn types_without_contract_have_no_slots() {
    assert!(expected_slots(NoteType::Task).is_empty());
    assert!(expected_slots(NoteType::Epic).is_empty());
    assert!(expected_slots(NoteType::Meta).is_empty());
    assert!(missing_slots(NoteType::Task, "", 0, 0).is_empty());
    assert!(missing_slots(NoteType::Meta, "", 0, 0).is_empty());
}
