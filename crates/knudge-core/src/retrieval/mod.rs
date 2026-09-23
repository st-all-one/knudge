//! Escopo `retrieval`: BM25, âncoras, filtros, views e fusão RRF (E06).
//!
//! A cascata do `recall` vai do mais barato ao mais caro: **filtros determinísticos** → **BM25**
//! no resíduo → **âncoras** como canal → **fusão RRF** (D39/D41/D81). Canais ausentes/falhos
//! degradam para o lexical com `warnings`, nunca abortam (a menos que `strict` esteja ligado).

pub mod anchor;
pub mod bm25;
pub mod filter;
pub mod format;
pub mod index;
pub mod rrf;
pub mod token;
pub mod views;
pub mod why;

#[cfg(test)]
mod tests;

pub use bm25::{B, Bm25Hit, CONFIRMATION_STEP, K1, type_weight};
pub use filter::{Filter, Meta};
pub use format::{format_brief, format_hit};
pub use index::{
    Field, FieldTf, INDEX_FILE, INDEX_WARN_BYTES, Index, NoteDoc, Stats, size_warning,
};
pub use rrf::{Fused, fuse};
pub use views::{BlockReason, Views, block_reason, compute_views, compute_views_at};
pub use why::Why;

use std::collections::{BTreeMap, BTreeSet};

use crate::graph::Graph;
use crate::lifecycle::confidence::{ConfidenceInput, confidence_score};
use crate::schema::EdgeKind;
use crate::store::{Note, Store};
use crate::{Error, Result};

/// `k` padrão da fusão RRF (config `recall.rrf_k`).
pub const DEFAULT_RRF_K: u32 = 60;

/// Limite padrão de hits (config `recall.default_limit`).
pub const DEFAULT_LIMIT: usize = 10;

/// Janela de recência do `why = recent` (7 dias em ms).
pub const RECENT_WINDOW_MS: i64 = 604_800_000;

/// Consulta de `recall`.
#[derive(Debug, Clone, Default)]
pub struct RecallQuery {
    /// Texto livre (tokenizado em ASCII).
    pub text: String,
    /// Máximo de hits (0 = sem limite).
    pub limit: usize,
    /// Constante da fusão RRF.
    pub rrf_k: u32,
    /// Filtros estruturais.
    pub filter: Filter,
    /// Container pedido (pertencimento via `depends_on` transitivo).
    pub container: Option<String>,
    /// Arquivos do working set (canal de âncoras).
    pub working_paths: Vec<String>,
    /// Ids do working set (canal de âncoras).
    pub working_ids: Vec<String>,
    /// Canal vetorial opcional (E11); `None` = desligado sem aviso.
    pub vector: Option<Vec<String>>,
    /// Avisos de canais fornecidos pelo chamador (ex.: provedor indisponível).
    pub channel_warnings: Vec<String>,
    /// Instante atual para o `why = recent` (opcional).
    pub now_ms: Option<i64>,
    /// Promove `warnings` a erro (config `behavior.strict` — D94).
    pub strict: bool,
}

impl RecallQuery {
    /// Consulta com defaults de config (`rrf_k=60`, `limit=10`).
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            limit: DEFAULT_LIMIT,
            rrf_k: DEFAULT_RRF_K,
            ..Self::default()
        }
    }
}

/// Hit de `recall` com o motivo.
#[derive(Debug, Clone, PartialEq)]
pub struct RecallHit {
    /// Id.
    pub id: String,
    /// `statement` da nota.
    pub statement: String,
    /// Score fundido (RRF).
    pub score: f64,
    /// Confiança **derivada** `[0,1]` (D87) — nunca armazenada.
    pub confidence: f64,
    /// Por que apareceu.
    pub why: Why,
}

/// Resultado (possivelmente parcial) de `recall`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecallOutput {
    /// Hits ordenados por `(score desc, id asc)`.
    pub hits: Vec<RecallHit>,
    /// Avisos de degradação graciosa.
    pub warnings: Vec<String>,
}

/// Executa a cascata filtros → BM25 → âncoras → RRF.
///
/// # Errors
/// Retorna `ErrorKind::Config` quando `strict` está ligado e há `warnings`.
pub fn recall(index: &Index, graph: &Graph, query: &RecallQuery) -> Result<RecallOutput> {
    let warnings = query.channel_warnings.clone();
    let allowed = candidates(index, graph, query);

    let lexical: Vec<String> = index
        .score(&query.text, &allowed)
        .into_iter()
        .map(|hit| hit.id)
        .collect();
    let anchored = anchor::rank(index, &allowed, &query.working_paths, &query.working_ids);

    let mut channels: Vec<&[String]> = vec![lexical.as_slice(), anchored.as_slice()];
    let semantic = semantic_channel(query, &allowed);
    if !semantic.is_empty() {
        channels.push(semantic.as_slice());
    }
    let fused = fuse(&channels, query.rrf_k);
    let max_score = fused.first().map_or(0.0, |hit| hit.score);

    let by_id: BTreeMap<&str, &NoteDoc> = index
        .docs
        .iter()
        .map(|doc| (doc.meta.id.as_str(), doc))
        .collect();
    let limit = if query.limit == 0 {
        usize::MAX
    } else {
        query.limit
    };
    let mut hits = Vec::new();
    for fused_hit in fused.into_iter().take(limit) {
        let Some(doc) = by_id.get(fused_hit.id.as_str()) else {
            continue;
        };
        let similarity = if max_score > 0.0 {
            fused_hit.score / max_score
        } else {
            0.0
        };
        let confidence = confidence_score(&ConfidenceInput {
            similarity,
            confirmation: doc.meta.confirmation,
            age_days: age_days(doc, query.now_ms),
            ..ConfidenceInput::default()
        });
        hits.push(RecallHit {
            id: fused_hit.id,
            statement: doc.statement.clone(),
            score: fused_hit.score,
            confidence,
            why: choose_why(doc, query, graph),
        });
    }

    if query.strict && !warnings.is_empty() {
        return Err(Error::config(format!(
            "recall estrito: {}",
            warnings.join("; ")
        )));
    }
    Ok(RecallOutput { hits, warnings })
}

/// Resultado de [`get`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetOutput {
    /// Notas encontradas, na ordem pedida.
    pub notes: Vec<Note>,
    /// Ids ausentes (resultado parcial).
    pub warnings: Vec<String>,
}

/// Lê os corpos dos ids pedidos, preservando a ordem (E06-T06).
///
/// # Errors
/// Propaga erros de I/O; ids ausentes viram `warnings` (resultado parcial).
pub fn get(store: &Store<'_>, ids: &[String]) -> Result<GetOutput> {
    let mut output = GetOutput::default();
    for id in ids {
        match store.read(id) {
            Ok(note) => output.notes.push(note),
            Err(Error::NotFound(_)) => output.warnings.push(format!("nota ausente: {id}")),
            Err(error) => return Err(error),
        }
    }
    Ok(output)
}

/// Canal vetorial **filtrado** por `allowed` (D102).
///
/// As views/filtros determinísticos são contrato e não podem ser furados por uma nota só
/// semanticamente próxima.
fn semantic_channel(query: &RecallQuery, allowed: &BTreeSet<String>) -> Vec<String> {
    query
        .vector
        .as_ref()
        .map(|ids| {
            ids.iter()
                .filter(|id| allowed.contains(id.as_str()))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn candidates(index: &Index, graph: &Graph, query: &RecallQuery) -> BTreeSet<String> {
    let mut allowed = BTreeSet::new();
    for doc in &index.docs {
        if !query.filter.matches(&doc.meta) {
            continue;
        }
        if let Some(container) = query.container.as_deref()
            && !belongs_to(graph, &doc.meta.id, container)
        {
            continue;
        }
        allowed.insert(doc.meta.id.clone());
    }
    allowed
}

fn belongs_to(graph: &Graph, id: &str, container: &str) -> bool {
    if id == container {
        return true;
    }
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![id.to_string()];
    while let Some(current) = stack.pop() {
        for dependency in graph.targets(&current, EdgeKind::DependsOn) {
            if dependency == container {
                return true;
            }
            if seen.insert(dependency.clone()) {
                stack.push(dependency.clone());
            }
        }
    }
    false
}

fn choose_why(doc: &NoteDoc, query: &RecallQuery, graph: &Graph) -> Why {
    let matched = anchor::match_note(&doc.meta, &query.working_paths, &query.working_ids);
    if matched.file {
        return Why::FileMatch;
    }
    if matched.id {
        return Why::AnchorMatch;
    }
    if let Some(container) = query.container.as_deref()
        && belongs_to(graph, &doc.meta.id, container)
    {
        return Why::TrackerMatch;
    }
    if doc.meta.confirmation > 0.0 {
        return Why::Stars;
    }
    if let Some(now) = query.now_ms
        && now.saturating_sub(doc.meta.created_ms) <= RECENT_WINDOW_MS
    {
        return Why::Recent;
    }
    Why::Universal
}

fn age_days(doc: &NoteDoc, now_ms: Option<i64>) -> f64 {
    let Some(now) = now_ms else {
        return 0.0;
    };
    let elapsed = now.saturating_sub(doc.meta.created_ms).max(0);
    f64::from(i32::try_from(elapsed / 86_400_000).unwrap_or(i32::MAX))
}
