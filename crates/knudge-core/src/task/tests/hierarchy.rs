//! Validação da hierarquia (E08-T07).

use crate::schema::Scope;
use crate::task::hierarchy::{expected_parent, validate_blocks, validate_parent};

#[test]
fn expected_parents_are_immediate() {
    assert_eq!(expected_parent(Scope::Plan), None);
    assert_eq!(expected_parent(Scope::Epic), Some(Scope::Plan));
    assert_eq!(expected_parent(Scope::Issue), Some(Scope::Epic));
    assert_eq!(expected_parent(Scope::Task), Some(Scope::Issue));
}

#[test]
fn valid_parents_pass() {
    assert!(validate_parent(Scope::Plan, Scope::Epic).is_ok());
    assert!(validate_parent(Scope::Epic, Scope::Issue).is_ok());
    assert!(validate_parent(Scope::Issue, Scope::Task).is_ok());
}

#[test]
fn invalid_parents_fail() {
    assert!(validate_parent(Scope::Epic, Scope::Task).is_err());
    assert!(validate_parent(Scope::Issue, Scope::Epic).is_err());
    assert!(validate_parent(Scope::Plan, Scope::Plan).is_err());
}

#[test]
fn blocks_are_one_based() {
    assert!(validate_blocks(None).is_ok());
    assert!(validate_blocks(Some(1)).is_ok());
    assert!(validate_blocks(Some(0)).is_err());
}
