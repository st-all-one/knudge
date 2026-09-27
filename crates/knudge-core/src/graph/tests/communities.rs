//! Testes de comunidades (Louvain determinístico — E19-T05/D193).

use std::collections::BTreeSet;

use proptest::collection::vec;
use proptest::prelude::*;

use crate::graph::communities::{WeightedGraph, louvain};

/// Grafo com os nós dados e as arestas `(a, b)`.
fn graph(nodes: &[&str], edges: &[(&str, &str)]) -> WeightedGraph {
    let mut graph = WeightedGraph::new(nodes.iter().copied());
    for (a, b) in edges {
        graph.add_edge(a, b, 1.0);
    }
    graph
}

/// Cada nó aparece em exatamente uma comunidade.
fn is_partition(partition: &[Vec<String>], nodes: &[String]) -> bool {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for community in partition {
        for member in community {
            if !seen.insert(member.as_str()) {
                return false;
            }
        }
    }
    seen.len() == nodes.len()
}

#[test]
fn two_cliques_with_a_bridge_split_in_two() {
    let nodes = ["a", "b", "c", "d", "e", "f"];
    let edges = [
        ("a", "b"),
        ("b", "c"),
        ("c", "a"),
        ("d", "e"),
        ("e", "f"),
        ("f", "d"),
        ("c", "d"),
    ];
    let partition = louvain(&graph(&nodes, &edges));
    assert_eq!(
        partition.len(),
        2,
        "esperava duas comunidades: {partition:?}"
    );
    assert!(partition.iter().all(|community| community.len() == 3));
    let first: BTreeSet<&str> = partition
        .first()
        .map(|community| community.iter().map(String::as_str).collect())
        .unwrap_or_default();
    assert!(
        first.contains("a") && first.contains("b") && first.contains("c")
            || first.contains("d") && first.contains("e") && first.contains("f"),
        "clique partido: {partition:?}"
    );
}

#[test]
fn isolated_nodes_are_singletons() {
    let partition = louvain(&graph(&["a", "b", "c"], &[]));
    assert_eq!(partition.len(), 3);
    assert!(partition.iter().all(|community| community.len() == 1));
}

#[test]
fn empty_graph_has_no_communities() {
    assert!(louvain(&WeightedGraph::new(Vec::<String>::new())).is_empty());
}

#[test]
fn missing_nodes_and_self_edges_are_ignored() {
    let mut graph = WeightedGraph::new(["a", "b"]);
    graph.add_edge("a", "a", 1.0);
    graph.add_edge("a", "z", 1.0);
    graph.add_edge("a", "b", 0.0);
    assert_eq!(graph.nodes().len(), 2);
    let partition = louvain(&graph);
    assert_eq!(partition.len(), 2, "sem aresta válida: {partition:?}");
}

#[test]
fn insertion_order_does_not_change_the_partition() {
    let nodes = ["a", "b", "c", "d"];
    let edges = [("a", "b"), ("b", "c"), ("c", "d"), ("d", "a")];
    let forward = louvain(&graph(&nodes, &edges));
    let mut reversed_edges = edges;
    reversed_edges.reverse();
    let reversed = louvain(&graph(&nodes, &reversed_edges));
    assert_eq!(forward, reversed);
}

#[test]
fn a_shared_anchor_group_is_one_community() {
    // Estrela: o hub liga todos; devem cair na mesma comunidade.
    let mut graph = WeightedGraph::new(["hub", "a", "b", "c", "d"]);
    for member in ["a", "b", "c", "d"] {
        graph.add_edge("hub", member, 1.0);
    }
    let partition = louvain(&graph);
    assert_eq!(
        partition.len(),
        1,
        "estrela devia ser 1 comunidade: {partition:?}"
    );
    assert_eq!(partition.first().map(Vec::len), Some(5));
}

#[test]
fn the_partition_is_deterministic() {
    let nodes = ["a", "b", "c", "d", "e"];
    let edges = [("a", "b"), ("b", "c"), ("c", "d"), ("d", "e"), ("e", "a")];
    let graph = graph(&nodes, &edges);
    assert_eq!(louvain(&graph), louvain(&graph));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn every_node_appears_exactly_once(edge_list in vec((0u8..8, 0u8..8), 0..24)) {
        let nodes: Vec<String> = (0u8..8).map(|i| format!("n{i}")).collect();
        let mut graph = WeightedGraph::new(nodes.clone());
        for (a, b) in edge_list {
            graph.add_edge(&format!("n{a}"), &format!("n{b}"), 1.0);
        }
        let partition = louvain(&graph);
        prop_assert!(is_partition(&partition, &nodes));
    }

    #[test]
    fn the_result_does_not_depend_on_edge_order(edge_list in vec((0u8..6, 0u8..6), 0..16)) {
        let nodes: Vec<String> = (0u8..6).map(|i| format!("n{i}")).collect();
        let mut forward = WeightedGraph::new(nodes.clone());
        for (a, b) in &edge_list {
            forward.add_edge(&format!("n{a}"), &format!("n{b}"), 1.0);
        }
        let mut reversed = WeightedGraph::new(nodes);
        for (a, b) in edge_list.iter().rev() {
            reversed.add_edge(&format!("n{a}"), &format!("n{b}"), 1.0);
        }
        prop_assert_eq!(louvain(&forward), louvain(&reversed));
    }
}
