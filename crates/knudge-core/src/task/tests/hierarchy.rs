//! Validação da hierarquia (E08-T07/D134).

use crate::schema::Scope;
use crate::task::hierarchy::{child, validate_blocks, validate_parent};

#[test]
fn child_is_always_the_task_leaf() {
    assert_eq!(child(Scope::Epic), Some(Scope::Task));
    assert_eq!(child(Scope::Issue), Some(Scope::Task));
    assert_eq!(child(Scope::Task), None);
}

#[test]
fn valid_parents_pass() {
    assert!(validate_parent(Scope::Epic, Scope::Issue).is_ok());
    assert!(validate_parent(Scope::Epic, Scope::Task).is_ok());
    assert!(validate_parent(Scope::Issue, Scope::Task).is_ok());
}

#[test]
fn invalid_parents_fail() {
    assert!(validate_parent(Scope::Issue, Scope::Epic).is_err());
    assert!(validate_parent(Scope::Task, Scope::Epic).is_err());
    assert!(validate_parent(Scope::Task, Scope::Issue).is_err());
    assert!(validate_parent(Scope::Epic, Scope::Epic).is_err());
}

#[test]
fn blocks_are_one_based() {
    assert!(validate_blocks(None).is_ok());
    assert!(validate_blocks(Some(1)).is_ok());
    assert!(validate_blocks(Some(0)).is_err());
}
