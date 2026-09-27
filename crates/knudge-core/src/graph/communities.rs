//! Detecção de comunidades determinística (Louvain) sobre grafo ponderado não-dirigido (D193).
//!
//! Implementa *local moving* + agregação do Louvain com ordem canônica: nós processados em
//! ordem lexicográfica, empate de ganho fica com a comunidade corrente (só troca em ganho
//! **estritamente** maior) e nº de níveis/passos é fixo. O resultado é função **pura** do grafo
//! (mesma entrada ⇒ mesma partição), sem RNG nem `HashMap`.

use std::collections::BTreeMap;

/// Número máximo de níveis de agregação.
pub const MAX_LEVELS: usize = 8;
/// Número máximo de passos de *local moving* por nível.
pub const MAX_PASSES: usize = 64;
/// Tolerância de comparação de ganho de modularidade.
const EPSILON: f64 = 1e-9;

/// Grafo não-dirigido ponderado, com ids canônicos ordenados.
#[derive(Debug, Clone, Default)]
pub struct WeightedGraph {
    nodes: Vec<String>,
    index: BTreeMap<String, usize>,
    adj: Vec<BTreeMap<usize, f64>>,
}

impl WeightedGraph {
    /// Cria um grafo com os nós dados (dedup + ordena).
    #[must_use]
    pub fn new<I, S>(nodes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut unique: Vec<String> = nodes.into_iter().map(Into::into).collect();
        unique.sort();
        unique.dedup();
        let index: BTreeMap<String, usize> = unique
            .iter()
            .enumerate()
            .map(|(i, node)| (node.clone(), i))
            .collect();
        let adj = vec![BTreeMap::new(); unique.len()];
        Self {
            nodes: unique,
            index,
            adj,
        }
    }

    /// Ids dos nós, ordenados.
    #[must_use]
    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }

    /// Adiciona peso à aresta não-dirigida `a—b` (ignora auto-aresta, peso ≤ 0 e id ausente).
    pub fn add_edge(&mut self, a: &str, b: &str, weight: f64) {
        let (Some(&i), Some(&j)) = (self.index.get(a), self.index.get(b)) else {
            return;
        };
        if i == j || weight <= 0.0 {
            return;
        }
        if let Some(neighbors) = self.adj.get_mut(i) {
            *neighbors.entry(j).or_insert(0.0) += weight;
        }
        if let Some(neighbors) = self.adj.get_mut(j) {
            *neighbors.entry(i).or_insert(0.0) += weight;
        }
    }
}

/// Particiona os nós em comunidades (Louvain determinístico).
///
/// Devolve as comunidades ordenadas por tamanho desc e, em empate, pelo primeiro membro asc;
/// cada comunidade tem os membros ordenados. Nós isolados viram comunidades de 1.
#[must_use]
pub fn louvain(graph: &WeightedGraph) -> Vec<Vec<String>> {
    let n = graph.nodes.len();
    if n == 0 {
        return Vec::new();
    }
    let mut adj = graph.adj.clone();
    // `members[new_idx]` = nós originais daquele super-nó.
    let mut members: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    let mut levels = 0;
    loop {
        let labels = local_moving(&adj);
        let mut grouped: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (node, &label) in labels.iter().enumerate() {
            if let Some(list) = members.get(node) {
                grouped
                    .entry(label)
                    .or_default()
                    .extend(list.iter().copied());
            }
        }
        let mut order: Vec<usize> = grouped.keys().copied().collect();
        order.sort_by_key(|&c| {
            grouped
                .get(&c)
                .and_then(|list| list.iter().copied().min())
                .unwrap_or(usize::MAX)
        });
        let mut remap: BTreeMap<usize, usize> = BTreeMap::new();
        let mut next_members: Vec<Vec<usize>> = Vec::with_capacity(order.len());
        for (new_idx, &label) in order.iter().enumerate() {
            remap.insert(label, new_idx);
            let mut list = grouped.get(&label).cloned().unwrap_or_default();
            list.sort_unstable();
            next_members.push(list);
        }
        let changed = next_members.len() < adj.len();
        members = next_members;
        if !changed || levels >= MAX_LEVELS {
            break;
        }
        adj = aggregate(&adj, &labels, &remap);
        levels = levels.saturating_add(1);
    }
    let mut result: Vec<Vec<String>> = members
        .iter()
        .map(|list| {
            list.iter()
                .filter_map(|&i| graph.nodes.get(i).cloned())
                .collect()
        })
        .collect();
    result.sort_by(|a, b| {
        b.len()
            .cmp(&a.len())
            .then_with(|| a.first().cmp(&b.first()))
    });
    result
}

/// *Local moving*: cada nó migra para a comunidade vizinha de maior ganho de modularidade.
fn local_moving(adj: &[BTreeMap<usize, f64>]) -> Vec<usize> {
    let n = adj.len();
    let degree: Vec<f64> = adj
        .iter()
        .map(|neighbors| neighbors.values().sum())
        .collect();
    let m2: f64 = degree.iter().sum();
    if m2 <= 0.0 {
        return (0..n).collect();
    }
    let mut labels: Vec<usize> = (0..n).collect();
    let mut total: Vec<f64> = degree.clone();
    for _ in 0..MAX_PASSES {
        let mut moved = false;
        for i in 0..n {
            let Some(&current) = labels.get(i) else {
                continue;
            };
            let mut neighbor_comm: BTreeMap<usize, f64> = BTreeMap::new();
            if let Some(neighbors) = adj.get(i) {
                for (&j, &w) in neighbors {
                    if let Some(label) = labels.get(j) {
                        *neighbor_comm.entry(*label).or_insert(0.0) += w;
                    }
                }
            }
            let degree_i = degree.get(i).copied().unwrap_or(0.0);
            if let Some(entry) = total.get_mut(current) {
                *entry -= degree_i;
            }
            let w_current = neighbor_comm.get(&current).copied().unwrap_or(0.0);
            let total_current = total.get(current).copied().unwrap_or(0.0);
            let mut best = current;
            let mut best_gain = w_current - total_current * degree_i / m2;
            for (&c, &w) in &neighbor_comm {
                if c == current {
                    continue;
                }
                let total_c = total.get(c).copied().unwrap_or(0.0);
                let gain = w - total_c * degree_i / m2;
                if gain > best_gain + EPSILON {
                    best_gain = gain;
                    best = c;
                }
            }
            if let Some(label) = labels.get_mut(i) {
                *label = best;
            }
            if let Some(entry) = total.get_mut(best) {
                *entry += degree_i;
            }
            if best != current {
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    labels
}

/// Agrega o grafo: um super-nó por comunidade, arestas internas viram auto-laço (peso ×2).
fn aggregate(
    adj: &[BTreeMap<usize, f64>],
    labels: &[usize],
    remap: &BTreeMap<usize, usize>,
) -> Vec<BTreeMap<usize, f64>> {
    let mut new_adj: Vec<BTreeMap<usize, f64>> = vec![BTreeMap::new(); remap.len()];
    let mut self_weight: Vec<f64> = vec![0.0; remap.len()];
    for (i, neighbors) in adj.iter().enumerate() {
        let Some(&li) = labels.get(i).and_then(|label| remap.get(label)) else {
            continue;
        };
        for (&j, &w) in neighbors {
            if j < i {
                continue; // cada aresta não-dirigida uma vez
            }
            let Some(&lj) = labels.get(j).and_then(|label| remap.get(label)) else {
                continue;
            };
            if li == lj {
                if let Some(entry) = self_weight.get_mut(li) {
                    *entry += w;
                }
            } else {
                if let Some(entry) = new_adj.get_mut(li) {
                    *entry.entry(lj).or_insert(0.0) += w;
                }
                if let Some(entry) = new_adj.get_mut(lj) {
                    *entry.entry(li).or_insert(0.0) += w;
                }
            }
        }
    }
    for (i, w) in self_weight.iter().enumerate() {
        if *w > 0.0
            && let Some(entry) = new_adj.get_mut(i)
        {
            *entry.entry(i).or_insert(0.0) += 2.0 * w;
        }
    }
    new_adj
}
