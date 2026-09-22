//! Detecção de ciclos (D45).
//!
//! Componentes fortemente conexos via **Kosaraju iterativo** (sem recursão, sem dependências).
//! Membros de ciclo de supersessão **não demovem** (D45): a demolição é decidida pela camada
//! que aplica supersessão (`cycle_members` marca os protegidos).

#![allow(
    clippy::arithmetic_side_effects,
    reason = "índices de varredura com domínio limitado ao número de nós"
)]

use std::collections::{BTreeMap, BTreeSet};

use crate::schema::EdgeKind;

use super::Graph;

/// Adjacência de um grafo (nó → sucessores).
pub type Adjacency = BTreeMap<String, Vec<String>>;

/// Detecta componentes fortemente conexos de `adjacency` sobre `nodes`.
///
/// Devolve só componentes **cíclicos** (mais de um nó, ou um nó com auto-aresta), com
/// membros e ordem determinísticos.
#[must_use]
pub fn cyclic_components(nodes: &BTreeSet<String>, adjacency: &Adjacency) -> Vec<Vec<String>> {
    let order = finishing_order(nodes, adjacency);
    let mut reverse: Adjacency = BTreeMap::new();
    for (from, targets) in adjacency {
        for to in targets {
            reverse.entry(to.clone()).or_default().push(from.clone());
        }
    }
    let mut assigned: BTreeSet<String> = BTreeSet::new();
    let mut cycles = Vec::new();
    for node in order.into_iter().rev() {
        if !assigned.insert(node.clone()) {
            continue;
        }
        let mut component = vec![node.clone()];
        let mut stack = vec![node];
        while let Some(current) = stack.pop() {
            if let Some(neighbors) = reverse.get(&current) {
                for next in neighbors {
                    if assigned.insert(next.clone()) {
                        component.push(next.clone());
                        stack.push(next.clone());
                    }
                }
            }
        }
        component.sort();
        let self_loop = component.len() == 1
            && component.first().is_some_and(|only| {
                adjacency
                    .get(only)
                    .is_some_and(|targets| targets.contains(only))
            });
        if component.len() > 1 || self_loop {
            cycles.push(component);
        }
    }
    cycles.sort();
    cycles
}

/// Todos os componentes fortemente conexos (inclui os triviais), para reuso por E06.
#[must_use]
pub fn strongly_connected(nodes: &BTreeSet<String>, adjacency: &Adjacency) -> Vec<Vec<String>> {
    let order = finishing_order(nodes, adjacency);
    let mut reverse: Adjacency = BTreeMap::new();
    for (from, targets) in adjacency {
        for to in targets {
            reverse.entry(to.clone()).or_default().push(from.clone());
        }
    }
    let mut assigned: BTreeSet<String> = BTreeSet::new();
    let mut components = Vec::new();
    for node in order.into_iter().rev() {
        if !assigned.insert(node.clone()) {
            continue;
        }
        let mut component = vec![node.clone()];
        let mut stack = vec![node];
        while let Some(current) = stack.pop() {
            if let Some(neighbors) = reverse.get(&current) {
                for next in neighbors {
                    if assigned.insert(next.clone()) {
                        component.push(next.clone());
                        stack.push(next.clone());
                    }
                }
            }
        }
        component.sort();
        components.push(component);
    }
    components.sort();
    components
}

fn finishing_order(nodes: &BTreeSet<String>, adjacency: &Adjacency) -> Vec<String> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut order = Vec::new();
    for root in nodes {
        if !visited.insert(root.clone()) {
            continue;
        }
        let mut stack: Vec<(String, usize)> = vec![(root.clone(), 0)];
        while let Some((node, index)) = stack.last().map(|(node, index)| (node.clone(), *index)) {
            let children = adjacency.get(&node).cloned().unwrap_or_default();
            if let Some(child) = children.get(index) {
                if let Some(top) = stack.last_mut() {
                    top.1 = index.saturating_add(1);
                }
                if visited.insert(child.clone()) {
                    stack.push((child.clone(), 0));
                }
            } else {
                let (done, _) = stack.pop().unwrap_or_default();
                order.push(done);
            }
        }
    }
    order
}

impl Graph {
    /// Ciclos de supersessão (aresta explícita `replaces` — D45/D49).
    #[must_use]
    pub fn supersession_cycles(&self) -> Vec<Vec<String>> {
        let (nodes, adjacency) = self.adjacency(|node| {
            node.edges
                .get(&EdgeKind::Replaces)
                .cloned()
                .unwrap_or_default()
        });
        cyclic_components(&nodes, &adjacency)
    }

    /// Ciclos de dependência (`depends_on`).
    #[must_use]
    pub fn dependency_cycles(&self) -> Vec<Vec<String>> {
        let (nodes, adjacency) = self.adjacency(|node| {
            node.edges
                .get(&EdgeKind::DependsOn)
                .cloned()
                .unwrap_or_default()
        });
        cyclic_components(&nodes, &adjacency)
    }

    /// União dos membros de todos os ciclos (proteção contra demolição — D45).
    #[must_use]
    pub fn cycle_members(&self) -> BTreeSet<String> {
        let mut members = BTreeSet::new();
        for cycle in self.supersession_cycles() {
            members.extend(cycle);
        }
        for cycle in self.dependency_cycles() {
            members.extend(cycle);
        }
        members
    }

    fn adjacency(
        &self,
        successors: impl Fn(&super::Node) -> Vec<String>,
    ) -> (BTreeSet<String>, Adjacency) {
        let nodes: BTreeSet<String> = self.nodes.keys().cloned().collect();
        let adjacency = self
            .nodes
            .iter()
            .map(|(id, node)| (id.clone(), successors(node)))
            .collect();
        (nodes, adjacency)
    }
}
