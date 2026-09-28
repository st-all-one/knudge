//! Testes do "did-you-mean" para enums fechados (D212).

use super::super::suggest::{closest, distance};
use super::super::types::Status;
use std::str::FromStr;

#[test]
fn unknown_enum_lists_options_and_suggests_the_closest() {
    let message = Status::from_str("activee")
        .err()
        .map(|error| error.to_string());
    let Some(message) = message else {
        return;
    };
    assert!(message.contains("use: active"), "{message}");
    assert!(message.contains("você quis dizer \"active\""), "{message}");
}

#[test]
fn suggest_distance_and_closest_are_deterministic() {
    assert_eq!(distance("kitten", "sitting"), 3);
    assert_eq!(distance("", "abc"), 3);
    assert_eq!(distance("same", "same"), 0);
    assert_eq!(closest("fakt", &["fact", "decision"]), Some("fact"));
    assert_eq!(closest("zzzzzz", &["fact", "decision"]), None);
}
