//! Papel derivado da árvore (D115/D134).

use crate::schema::NoteType;
use crate::task::role::{Role, role};

#[test]
fn species_wins_over_level() {
    assert_eq!(role(1, NoteType::Error, false), Role::Bug);
    assert_eq!(role(1, NoteType::Question, false), Role::Spike);
    assert_eq!(role(1, NoteType::Risk, false), Role::Risk);
    assert_eq!(role(2, NoteType::Decision, false), Role::Decision);
}

#[test]
fn level_one_with_children_is_feature() {
    assert_eq!(role(1, NoteType::Task, true), Role::Feature);
    assert_eq!(role(1, NoteType::Task, false), Role::Story);
}

#[test]
fn depth_maps_to_epic_feature_subtask() {
    assert_eq!(role(0, NoteType::Epic, true), Role::Epic);
    assert_eq!(role(1, NoteType::Epic, true), Role::Feature);
    assert_eq!(role(2, NoteType::Task, false), Role::SubTask);
}
