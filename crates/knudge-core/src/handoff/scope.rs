//! Auto-context-scope e auto-flip (D41).
//!
//! Deriva o escopo dos arquivos tocados (`git status -uall` + trabalho ativo) e decide quando
//! trocar a listagem por uma manifest barata: `>100 notas` ou `>5 containers`.

use crate::graph::Graph;
use crate::retrieval::Index;
use crate::retrieval::anchor::glob_match;
use crate::schema::NoteType;

use super::manifest::belongs_to;

/// Limite de notas que dispara o flip para manifest.
pub const FLIP_NOTES: usize = 100;

/// Limite de containers que dispara o flip para manifest.
pub const FLIP_CONTAINERS: usize = 5;

/// `true` se o corpus é grande o bastante para preferir a manifest.
#[must_use]
pub const fn should_flip(note_count: usize, container_count: usize) -> bool {
    note_count > FLIP_NOTES || container_count > FLIP_CONTAINERS
}

/// Deriva o container mais coerente com os arquivos tocados.
#[must_use]
pub fn detect_scope(changed_paths: &[String], graph: &Graph, index: &Index) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for container in graph.ids() {
        if graph.note_type(container) != Some(NoteType::Epic) {
            continue;
        }
        let count = matches_for(container, changed_paths, graph, index);
        if count == 0 {
            continue;
        }
        let candidate = (count, container.to_string());
        best = match best {
            Some(current) if current.0 > count => Some(current),
            Some(current) if current.0 == count && current.1 <= candidate.1 => Some(current),
            _ => Some(candidate),
        };
    }
    best.map(|(_, id)| id)
}

fn matches_for(container: &str, changed_paths: &[String], graph: &Graph, index: &Index) -> usize {
    let mut count = 0_usize;
    for doc in &index.docs {
        if !belongs_to(graph, &doc.meta.id, container) {
            continue;
        }
        for anchor in &doc.meta.anchors {
            if changed_paths.iter().any(|path| glob_match(anchor, path)) {
                count = count.saturating_add(1);
            }
        }
    }
    count
}
