//! `learn`: sugestões determinísticas a partir de eventos + âncoras (D33, E08-T06).
//!
//! Nunca escreve: emite **propostas** `{kind, ids, why, score}` com
//! `kind ∈ {create_note, merge, supersede, link}` (D47). Três sinais: atividade sem registro
//! (write-gap), quase-duplicata e lacuna de grafo (âncora compartilhada sem aresta explícita).

use crate::graph::Graph;
use crate::handoff::manifest::belongs_to;
use crate::retrieval::Index;
use crate::retrieval::anchor::glob_match;
use crate::retrieval::index::NoteDoc;
use crate::schema::EdgeKind;
use crate::store::Event;
use crate::write::dedup::{DedupThresholds, dice};

/// Teto de notas consideradas nos sinais pairwise (determinístico por id).
pub const MAX_DOCS: usize = 64;

/// Teto de propostas emitidas.
pub const MAX_PROPOSALS: usize = 50;

/// Tipo de proposta de `learn`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LearnKind {
    /// Criar nota a partir de atividade sem registro.
    CreateNote,
    /// Fundir quase-duplicatas.
    Merge,
    /// Superseder quase-duplicatas fortes.
    Supersede,
    /// Criar aresta entre notas relacionadas.
    Link,
}

impl LearnKind {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreateNote => "create_note",
            Self::Merge => "merge",
            Self::Supersede => "supersede",
            Self::Link => "link",
        }
    }
}

/// Proposta revisável (ponteiro, nunca conteúdo).
#[derive(Debug, Clone, PartialEq)]
pub struct LearnProposal {
    /// Tipo.
    pub kind: LearnKind,
    /// Ids envolvidos (ou a âncora, em `create_note`).
    pub ids: Vec<String>,
    /// Por que foi proposta.
    pub why: String,
    /// Score de apoio.
    pub score: f64,
}

/// Entradas de `learn`.
pub struct LearnInput<'a> {
    /// Índice.
    pub index: &'a Index,
    /// Grafo (arestas explícitas).
    pub graph: &'a Graph,
    /// Eventos (auditoria).
    pub events: &'a [Event],
    /// Arquivos tocados.
    pub changed_paths: &'a [String],
    /// Escopo opcional (container).
    pub scope: Option<&'a str>,
    /// Limiares de dedup.
    pub thresholds: &'a DedupThresholds,
}

/// Gera propostas determinísticas (sem escrever nada).
#[must_use]
pub fn learn(input: &LearnInput<'_>) -> Vec<LearnProposal> {
    let docs = scoped_docs(input);
    let mut proposals = Vec::new();
    proposals.extend(write_gaps(input, &docs));
    proposals.extend(duplicates(input, &docs));
    proposals.extend(missing_links(input, &docs));
    proposals.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| left.ids.cmp(&right.ids))
    });
    proposals.truncate(MAX_PROPOSALS);
    proposals
}

fn scoped_docs<'a>(input: &LearnInput<'a>) -> Vec<&'a NoteDoc> {
    let mut docs: Vec<&NoteDoc> = input
        .index
        .docs
        .iter()
        .filter(|doc| {
            input
                .scope
                .is_none_or(|container| belongs_to(input.graph, &doc.meta.id, container))
        })
        .collect();
    docs.sort_by(|a, b| a.meta.id.cmp(&b.meta.id));
    docs.truncate(MAX_DOCS);
    docs
}

fn write_gaps(input: &LearnInput<'_>, docs: &[&NoteDoc]) -> Vec<LearnProposal> {
    let mut proposals = Vec::new();
    for path in input.changed_paths {
        let covered = docs.iter().any(|doc| {
            doc.meta
                .anchors
                .iter()
                .any(|anchor| glob_match(anchor, path))
        });
        if !covered {
            proposals.push(LearnProposal {
                kind: LearnKind::CreateNote,
                ids: vec![path.clone()],
                why: "atividade sem registro".to_string(),
                score: 0.5,
            });
        }
    }
    proposals
}

fn duplicates(input: &LearnInput<'_>, docs: &[&NoteDoc]) -> Vec<LearnProposal> {
    let mut proposals = Vec::new();
    for (index, left) in docs.iter().enumerate() {
        for right in docs.iter().skip(index.saturating_add(1)) {
            let score = dice(left, right);
            let kind = if score >= input.thresholds.merge_below {
                LearnKind::Supersede
            } else if score >= input.thresholds.create_below {
                LearnKind::Merge
            } else {
                continue;
            };
            let (keep, drop) = order_pair(left, right);
            proposals.push(LearnProposal {
                kind,
                ids: vec![keep, drop],
                why: format!("similaridade {score:.2}"),
                score,
            });
        }
    }
    proposals
}

fn missing_links(input: &LearnInput<'_>, docs: &[&NoteDoc]) -> Vec<LearnProposal> {
    let mut proposals = Vec::new();
    for (index, left) in docs.iter().enumerate() {
        for right in docs.iter().skip(index.saturating_add(1)) {
            if !shares_anchor(left, right) || has_edge(input.graph, &left.meta.id, &right.meta.id) {
                continue;
            }
            proposals.push(LearnProposal {
                kind: LearnKind::Link,
                ids: vec![left.meta.id.clone(), right.meta.id.clone()],
                why: "âncora compartilhada sem aresta".to_string(),
                score: 0.4,
            });
        }
    }
    proposals
}

fn shares_anchor(left: &NoteDoc, right: &NoteDoc) -> bool {
    left.meta
        .anchors
        .iter()
        .any(|anchor| right.meta.anchors.iter().any(|other| other == anchor))
}

fn has_edge(graph: &Graph, left: &str, right: &str) -> bool {
    EdgeKind::ALL.iter().any(|kind| {
        graph.targets(left, *kind).iter().any(|id| id == right)
            || graph.targets(right, *kind).iter().any(|id| id == left)
    })
}

fn order_pair(left: &NoteDoc, right: &NoteDoc) -> (String, String) {
    if (left.meta.created_ms, &left.meta.id) <= (right.meta.created_ms, &right.meta.id) {
        (left.meta.id.clone(), right.meta.id.clone())
    } else {
        (right.meta.id.clone(), left.meta.id.clone())
    }
}
