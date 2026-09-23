//! Papel derivado da árvore (D115).

use crate::schema::{NoteType, Scope};
use crate::task::role::{Role, role};

#[test]
fn species_wins_over_level() {
    assert_eq!(role(Scope::Issue, NoteType::Error, false), Role::Bug);
    assert_eq!(role(Scope::Issue, NoteType::Question, false), Role::Spike);
    assert_eq!(role(Scope::Issue, NoteType::Risk, false), Role::Risk);
    assert_eq!(role(Scope::Task, NoteType::Decision, false), Role::Decision);
}

#[test]
fn issue_with_children_is_feature() {
    assert_eq!(role(Scope::Issue, NoteType::Task, true), Role::Feature);
    assert_eq!(role(Scope::Issue, NoteType::Task, false), Role::Story);
}

#[test]
fn levels_map_to_initiative_epic_subtask() {
    assert_eq!(
        role(Scope::Plan, NoteType::Container, true),
        Role::Initiative
    );
    assert_eq!(role(Scope::Epic, NoteType::Container, true), Role::Epic);
    assert_eq!(role(Scope::Task, NoteType::Task, false), Role::SubTask);
}
