//! Modo de execução derivado (D116).

use crate::task::mode::{Child, Container, Mode, mode};

fn child(id: &str, owner: Option<&str>, depends_on: &[&str]) -> Child {
    Child {
        id: id.to_string(),
        owner: owner.map(str::to_string),
        depends_on: depends_on.iter().map(|dep| (*dep).to_string()).collect(),
    }
}

fn container(children: Vec<Child>) -> Container {
    Container {
        owner: None,
        children,
        handoff: false,
        incremental: false,
    }
}

#[test]
fn chain_is_sequential() {
    let c = container(vec![child("a", None, &[]), child("b", None, &["a"])]);
    assert_eq!(mode(&c), Mode::Sequential);
}

#[test]
fn independent_children_are_concurrent() {
    let c = container(vec![child("a", None, &[]), child("b", None, &[])]);
    assert_eq!(mode(&c), Mode::Concurrent);
}

#[test]
fn owner_with_distinct_child_owners_is_supervisor() {
    let mut c = container(vec![
        child("a", Some("agente-a"), &[]),
        child("b", Some("agente-b"), &[]),
    ]);
    c.owner = Some("supervisor".to_string());
    assert_eq!(mode(&c), Mode::Supervisor);
}

#[test]
fn handoff_has_precedence() {
    let mut c = container(vec![child("a", Some("agente-a"), &[])]);
    c.handoff = true;
    assert_eq!(mode(&c), Mode::Handoff);
}

#[test]
fn incremental_children_are_magentic() {
    let mut c = container(vec![child("a", None, &[]), child("b", None, &[])]);
    c.incremental = true;
    assert_eq!(mode(&c), Mode::Magentic);
}
