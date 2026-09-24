//! Modo de execução derivado (D116/D136).

use crate::task::mode::{Child, Container, Mode, mode};

fn child(id: &str, depends_on: &[&str]) -> Child {
    Child {
        id: id.to_string(),
        depends_on: depends_on.iter().map(|dep| (*dep).to_string()).collect(),
    }
}

fn container(children: Vec<Child>) -> Container {
    Container {
        children,
        incremental: false,
    }
}

#[test]
fn chain_is_sequential() {
    let c = container(vec![child("a", &[]), child("b", &["a"])]);
    assert_eq!(mode(&c), Mode::Sequential);
}

#[test]
fn independent_children_are_concurrent() {
    let c = container(vec![child("a", &[]), child("b", &[])]);
    assert_eq!(mode(&c), Mode::Concurrent);
}

#[test]
fn incremental_children_are_magentic() {
    let mut c = container(vec![child("a", &[]), child("b", &[])]);
    c.incremental = true;
    assert_eq!(mode(&c), Mode::Magentic);
}

#[test]
fn empty_or_single_child_is_sequential() {
    assert_eq!(mode(&container(vec![])), Mode::Sequential);
    assert_eq!(mode(&container(vec![child("a", &[])])), Mode::Sequential);
}
