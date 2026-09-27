//! Inferência de ontologia leve (E19-T09/D207).
//!
//! Fecha o vocabulário SKOS-lite sobre o grafo explícito, **sem** armazenar nada derivado:
//!
//! - `same_as` forma **classes de equivalência** (identidade de entidade) — representante
//!   canônico é o menor id da classe (determinístico).
//! - `broader`/`narrower` formam uma **hierarquia**: os dois sentidos são tratados como inversos,
//!   então `A --broader--> B` e `B --narrower--> A` descrevem o mesmo par. A **clausura
//!   transitiva** dá ancestrais (`broader`) e descendentes (`narrower`).
//!
//! Tudo é `BTreeMap`/`BTreeSet` (determinismo, D92) e puro — reconstruível a partir das notas.

use std::collections::{BTreeMap, BTreeSet};

use crate::Result;
use crate::graph::Graph;
use crate::schema::{EdgeKind, claims};
use crate::store::Note;

/// Classes de equivalência por `same_as` (id → representante = menor id da classe).
#[must_use]
pub fn equivalence_classes(graph: &Graph) -> BTreeMap<String, String> {
    let mut parent: BTreeMap<String, String> = graph
        .ids()
        .into_iter()
        .map(|id| (id.to_string(), id.to_string()))
        .collect();
    for id in graph.ids() {
        for target in graph.targets(id, EdgeKind::SameAs) {
            union(&mut parent, id, target);
        }
    }
    let mut canonical: BTreeMap<String, String> = BTreeMap::new();
    for id in graph.ids() {
        let root = find(&mut parent, id);
        canonical.insert(id.to_string(), root);
    }
    canonical
}

/// Ancestrais `broader` transitivos de `id` (ordem canônica, sem incluir `id`).
#[must_use]
pub fn broader_ancestors(graph: &Graph, id: &str) -> Vec<String> {
    closure(&broader_of(graph), id)
}

/// Descendentes `narrower` transitivos de `id` (ordem canônica, sem incluir `id`).
#[must_use]
pub fn narrower_descendants(graph: &Graph, id: &str) -> Vec<String> {
    closure(&invert(&broader_of(graph)), id)
}

/// `true` se a hierarquia `broader`/`narrower` tem ciclo.
#[must_use]
pub fn has_hierarchy_cycle(graph: &Graph) -> bool {
    let map = broader_of(graph);
    let mut color: BTreeMap<String, u8> = BTreeMap::new();
    for id in graph.ids() {
        if color.get(id).copied().unwrap_or(0) == 0 && dfs_cycle(&map, id, &mut color) {
            return true;
        }
    }
    false
}

/// Conflito de claims: mesma `(sujeito, relação)` com **objetos divergentes** (D207).
///
/// É a contradição **precisa**: não depende de aresta `contradicts` declarada, mas de duas
/// afirmações atômicas incompatíveis. `objects` traz ≥2 objetos distintos, ordenados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimConflict {
    /// Sujeito compartilhado.
    pub subject: String,
    /// Relação compartilhada.
    pub relation: String,
    /// Objetos divergentes (≥2, ordenados).
    pub objects: Vec<String>,
}

/// Detecta conflitos de claims no corpus (puro, determinístico).
///
/// # Errors
/// Retorna `ErrorKind::Schema` se uma claim for malformada.
pub fn claim_conflicts(notes: &[Note]) -> Result<Vec<ClaimConflict>> {
    let mut groups: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for note in notes {
        for claim in claims(&note.frontmatter)? {
            let (subject, relation) = claim.key();
            groups
                .entry((subject.to_string(), relation.to_string()))
                .or_default()
                .insert(claim.object);
        }
    }
    let mut conflicts = Vec::new();
    for ((subject, relation), objects) in groups {
        if objects.len() > 1 {
            conflicts.push(ClaimConflict {
                subject,
                relation,
                objects: objects.into_iter().collect(),
            });
        }
    }
    Ok(conflicts)
}

/// Mapa `filho → pais mais amplos` derivado de `broader` **e** do inverso de `narrower`.
fn broader_of(graph: &Graph) -> BTreeMap<String, BTreeSet<String>> {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for id in graph.ids() {
        for broader in graph.targets(id, EdgeKind::Broader) {
            map.entry(id.to_string())
                .or_default()
                .insert(broader.clone());
        }
        for narrower in graph.targets(id, EdgeKind::Narrower) {
            map.entry(narrower.clone())
                .or_default()
                .insert(id.to_string());
        }
    }
    map
}

/// Clausura transitiva a partir de `start` (sem incluir `start`).
fn closure(step: &BTreeMap<String, BTreeSet<String>>, start: &str) -> Vec<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = step
        .get(start)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect();
    while let Some(current) = stack.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        if let Some(next) = step.get(&current) {
            stack.extend(next.iter().cloned());
        }
    }
    seen.into_iter().collect()
}

fn invert(map: &BTreeMap<String, BTreeSet<String>>) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (child, parents) in map {
        for parent in parents {
            out.entry(parent.clone()).or_default().insert(child.clone());
        }
    }
    out
}

/// DFS tricolor sobre `child → parents`; `true` se há ciclo (aresta para nó cinza).
fn dfs_cycle(
    map: &BTreeMap<String, BTreeSet<String>>,
    node: &str,
    color: &mut BTreeMap<String, u8>,
) -> bool {
    color.insert(node.to_string(), 1);
    if let Some(parents) = map.get(node) {
        for parent in parents {
            match color.get(parent).copied().unwrap_or(0) {
                1 => return true,
                0 if dfs_cycle(map, parent, color) => return true,
                _ => {}
            }
        }
    }
    color.insert(node.to_string(), 2);
    false
}

fn find(parent: &mut BTreeMap<String, String>, id: &str) -> String {
    let mut root = id.to_string();
    loop {
        let next = parent.get(&root).cloned().unwrap_or_else(|| root.clone());
        if next == root {
            break;
        }
        root = next;
    }
    // Compressão de caminho.
    let mut node = id.to_string();
    while node != root {
        let next = parent.get(&node).cloned().unwrap_or_else(|| root.clone());
        parent.insert(node.clone(), root.clone());
        node = next;
    }
    root
}

fn union(parent: &mut BTreeMap<String, String>, a: &str, b: &str) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra == rb {
        return;
    }
    // Menor id vira raiz (determinístico).
    let (root, child) = if ra < rb { (ra, rb) } else { (rb, ra) };
    parent.insert(child, root);
}
