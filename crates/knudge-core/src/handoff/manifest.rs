//! Manifest e ranking por trust-tier (E08-T01/T03).
//!
//! A manifest é ultra-curta (contadores + recentes + dirty, ~30 tokens). O ranking de escopo/
//! working set ordena por `star*100 + foundational*50 + tactical*20 + observational*10`
//! (D57), com desempate determinístico por `created_ms desc, id asc`.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::graph::Graph;
use crate::lifecycle::confidence::{DEFAULT_TASK_CONFIRMATION, from_tasks_with, is_success_task};
use crate::retrieval::anchor::glob_match;
use crate::retrieval::views::{Views, compute_views};
use crate::retrieval::{Index, Meta};
use crate::schema::{Classification, EdgeKind, NoteType};

use super::{CorpusScope, RewindMode};

/// Nível de confiança do ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustTier {
    /// Confirmada por `outcomes` (`star`).
    Star,
    /// `foundational`.
    Foundational,
    /// `tactical`.
    Tactical,
    /// `observational`.
    Observational,
}

impl TrustTier {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Star => "star",
            Self::Foundational => "foundational",
            Self::Tactical => "tactical",
            Self::Observational => "observational",
        }
    }
}

/// Peso da classificação no ranking.
#[must_use]
pub const fn classification_weight(classification: Classification) -> f64 {
    match classification {
        Classification::Foundational => 50.0,
        Classification::Tactical => 20.0,
        Classification::Observational => 10.0,
    }
}

/// Score de confiança: `star*100 + peso da classificação`.
#[must_use]
pub fn trust_score(meta: &Meta) -> f64 {
    let star = if meta.confirmation > 0.0 { 100.0 } else { 0.0 };
    star + classification_weight(meta.classification)
}

/// Tier exibido para a nota (star domina a classificação).
#[must_use]
pub fn tier_of(meta: &Meta) -> TrustTier {
    if meta.confirmation > 0.0 {
        return TrustTier::Star;
    }
    match meta.classification {
        Classification::Foundational => TrustTier::Foundational,
        Classification::Tactical => TrustTier::Tactical,
        Classification::Observational => TrustTier::Observational,
    }
}

/// Item do manifest de escopo/working set.
#[derive(Debug, Clone, PartialEq)]
pub struct ManifestItem {
    /// Id.
    pub id: String,
    /// `statement`.
    pub statement: String,
    /// Tier exibido.
    pub tier: TrustTier,
    /// Score de confiança.
    pub score: f64,
    /// `created_at` em ms.
    pub created_ms: i64,
}

/// Ranqueia as notas do modo (escopo ou working set).
#[must_use]
pub fn rank(index: &Index, graph: &Graph, mode: &RewindMode) -> Vec<ManifestItem> {
    rank_with(
        index,
        graph,
        mode,
        DEFAULT_TASK_CONFIRMATION,
        &CorpusScope::default(),
    )
}

/// Como [`rank`], promovendo a `Star` notas confirmadas por tarefas (X1/D108) e restringindo
/// ao escopo do corpus (D143).
#[must_use]
pub fn rank_with(
    index: &Index,
    graph: &Graph,
    mode: &RewindMode,
    task_confirmation_weight: f64,
    scope: &CorpusScope,
) -> Vec<ManifestItem> {
    let roots = match mode {
        RewindMode::Files(paths) => program_roots(index, graph, paths),
        _ => Vec::new(),
    };
    let confirmers: Vec<&Meta> = index
        .docs
        .iter()
        .map(|doc| &doc.meta)
        .filter(|meta| is_success_task(meta))
        .collect();
    let mut items: Vec<ManifestItem> = index
        .docs
        .iter()
        .filter(|doc| scope.matches(&doc.meta))
        .filter(|doc| in_mode(&doc.meta, graph, mode, &roots))
        .map(|doc| {
            let confirmed = from_tasks_with(&doc.meta, &confirmers, task_confirmation_weight) > 0.0;
            ManifestItem {
                id: doc.meta.id.clone(),
                statement: doc.statement.clone(),
                tier: if confirmed {
                    TrustTier::Star
                } else {
                    tier_of(&doc.meta)
                },
                score: trust_score(&doc.meta) + if confirmed { 100.0 } else { 0.0 },
                created_ms: doc.meta.created_ms,
            }
        })
        .collect();
    items.sort_unstable_by(compare);
    items
}

/// Renderiza um item no contrato `id|statement|tier`.
#[must_use]
pub fn render_item(item: &ManifestItem) -> String {
    format!(
        "{}|{}|{}",
        item.id,
        sanitize(&item.statement),
        item.tier.as_str()
    )
}

/// Manifest ultra-curta: contadores, recentes e dirty (D57).
#[must_use]
pub fn manifest_text(
    index: &Index,
    graph: &Graph,
    changed_paths: &[String],
    scope: &CorpusScope,
) -> String {
    manifest_text_in(&compute_views(graph), index, graph, changed_paths, scope)
}

/// Como [`manifest_text`], mas com as `views` já computadas (O5.2).
pub(super) fn manifest_text_in(
    views: &Views,
    index: &Index,
    graph: &Graph,
    changed_paths: &[String],
    scope: &CorpusScope,
) -> String {
    let containers = graph
        .ids()
        .iter()
        .filter(|id| graph.note_type(id) == Some(NoteType::Epic))
        .count();
    let mut recent: Vec<&Meta> = index
        .docs
        .iter()
        .map(|doc| &doc.meta)
        .filter(|meta| scope.matches(meta))
        .collect();
    recent.sort_unstable_by(|a, b| {
        b.created_ms
            .cmp(&a.created_ms)
            .then_with(|| a.id.cmp(&b.id))
    });
    let recent_ids: Vec<&str> = recent.iter().take(3).map(|meta| meta.id.as_str()).collect();
    let dirty = if changed_paths.is_empty() {
        "clean"
    } else {
        "dirty"
    };
    let notes = recent.len();
    let ready = views.ready.len();
    let blocked = views.blocked.len();
    let recent = recent_ids.join(" ");
    format!(
        "notes={notes} ready={ready} blocked={blocked} containers={containers} {dirty}\nrecent: {recent}"
    )
}

/// `true` se `id` pertence ao container (via `depends_on` ou `results_in`).
#[must_use]
pub fn belongs_to(graph: &Graph, id: &str, container: &str) -> bool {
    if id == container {
        return true;
    }
    reachable(graph, container, id, EdgeKind::ResultsIn)
        || reachable(graph, id, container, EdgeKind::DependsOn)
}

fn reachable(graph: &Graph, from: &str, target: &str, kind: EdgeKind) -> bool {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![from.to_string()];
    while let Some(current) = stack.pop() {
        for next in graph.targets(&current, kind) {
            if next == target {
                return true;
            }
            if seen.insert(next.clone()) {
                stack.push(next.clone());
            }
        }
    }
    false
}

fn in_mode(meta: &Meta, graph: &Graph, mode: &RewindMode, roots: &[String]) -> bool {
    match mode {
        RewindMode::Manifest | RewindMode::Auto => true,
        RewindMode::Scope(container) => belongs_to(graph, &meta.id, container),
        RewindMode::Files(paths) => {
            let anchored = meta
                .anchors
                .iter()
                .any(|anchor| paths.iter().any(|path| glob_match(anchor, path)));
            anchored || roots.iter().any(|root| belongs_to(graph, &meta.id, root))
        }
    }
}

/// Épicos-raiz cujo `anchors` casa um dos `paths` (o arquivo do programa — D119).
fn program_roots(index: &Index, graph: &Graph, paths: &[String]) -> Vec<String> {
    let mut roots: Vec<String> = index
        .docs
        .iter()
        .filter(|doc| doc.meta.note_type == NoteType::Epic && !graph.has_parent(&doc.meta.id))
        .filter(|doc| {
            doc.meta.anchors.iter().any(|anchor| {
                paths
                    .iter()
                    .any(|path| glob_match(anchor, path) || glob_match(path, anchor))
            })
        })
        .map(|doc| doc.meta.id.clone())
        .collect();
    roots.sort();
    roots
}

fn compare(left: &ManifestItem, right: &ManifestItem) -> Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| right.created_ms.cmp(&left.created_ms))
        .then_with(|| left.id.cmp(&right.id))
}

pub(super) fn sanitize(statement: &str) -> String {
    let mut out = String::with_capacity(statement.len());
    let mut pending = false;
    for ch in statement.chars() {
        if ch == '|' || ch.is_whitespace() {
            pending = !out.is_empty();
        } else {
            if pending {
                out.push(' ');
                pending = false;
            }
            out.push(ch);
        }
    }
    out
}
