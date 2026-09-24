//! Escopo `handoff`: `rewind` — estado/handoff entre agentes e rodadas (E08-T01..T04/T09).
//!
//! O `prime` é o **protocolo estático** (D57); o `rewind` é o **estado dinâmico**. A família tem
//! três modos — manifest (~30 tokens), escopo (container/domínio) e working set (arquivos) —,
//! respeita orçamento sem tokenizer (D40/D82) e emite um `context_id` endereçável para handoff
//! 1:1 (D88).

pub mod budget;
pub mod context;
pub mod manifest;
pub mod next;
pub mod scope;

#[cfg(test)]
mod tests;

pub use budget::{BudgetSummary, Budgeted, DEFAULT_BUDGET, MIN_TAIL, apply_into, estimate_tokens};
pub use context::{CONTEXT_PREFIX, ContextStore, derive_id, is_valid_context_id};
pub use manifest::{
    ManifestItem, TrustTier, manifest_text, rank, rank_with, render_item, tier_of, trust_score,
};
pub use next::{NextTask, manifest_at, next_tasks};
pub use scope::{FLIP_CONTAINERS, FLIP_NOTES, detect_scope, should_flip};

use crate::graph::Graph;
use crate::lifecycle::Freshness;
use crate::retrieval::Index;
use crate::schema::NoteType;
use crate::store::Event;
use crate::{Error, Result};

/// Modo de `rewind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewindMode {
    /// Manifest ultra-curta (contadores + recentes + dirty).
    Manifest,
    /// Container/domínio.
    Scope(String),
    /// Working set por arquivos.
    Files(Vec<String>),
    /// Deriva o escopo dos arquivos tocados; cai para manifest se o corpus for grande.
    Auto,
}

/// Pedido de `rewind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewindRequest {
    /// Modo.
    pub mode: RewindMode,
    /// Orçamento de tokens.
    pub budget: usize,
    /// Início do intervalo (ms), quando houver.
    pub since: Option<i64>,
    /// Fim do intervalo (ms), quando houver.
    pub until: Option<i64>,
    /// Retoma um contexto por id (handoff 1:1).
    pub resume: Option<String>,
}

impl Default for RewindRequest {
    fn default() -> Self {
        Self {
            mode: RewindMode::Manifest,
            budget: DEFAULT_BUDGET,
            since: None,
            until: None,
            resume: None,
        }
    }
}

impl RewindRequest {
    /// Pedido com defaults (manifest, 4000 tokens).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Entradas derivadas do store para o `rewind`.
pub struct RewindInput<'a> {
    /// Índice (metadados/ranking).
    pub index: &'a Index,
    /// Grafo (containers/views).
    pub graph: &'a Graph,
    /// Eventos (intervalo/diff).
    pub events: &'a [Event],
    /// Arquivos tocados no working set.
    pub changed_paths: &'a [String],
    /// Frescor do corpus (shelf-life + fila de embeddings — D106).
    pub freshness: Freshness,
    /// Peso da confirmação derivada de tarefas (X1/D108).
    pub task_confirmation_weight: f64,
    /// Instante atual (ms).
    pub now_ms: i64,
}

/// Saída do `rewind`.
#[derive(Debug, Clone, PartialEq)]
pub struct RewindOutput {
    /// Id endereçável do contexto.
    pub context_id: String,
    /// Texto renderizado (respeitando o orçamento).
    pub text: String,
    /// Itens ranqueados (vazio no modo manifest/resume).
    pub items: Vec<ManifestItem>,
    /// `true` se o último item foi truncado pelo orçamento.
    pub truncated: bool,
    /// Itens descartados pelo orçamento.
    pub dropped: usize,
    /// Notas ainda `pending`/`stale` na fila de embeddings.
    pub embeddings_pending: usize,
    /// Avisos de degradação graciosa.
    pub warnings: Vec<String>,
}

/// Executa o `rewind` e registra o contexto derivado (D88).
///
/// # Errors
/// - `ErrorKind::InvalidInput` para `context_id` malformado;
/// - `ErrorKind::NotFound` para `--resume` de contexto ausente;
/// - propaga erros de I/O do store de contextos.
pub fn rewind(
    input: &RewindInput<'_>,
    request: &RewindRequest,
    contexts: &ContextStore<'_>,
) -> Result<RewindOutput> {
    if let Some(resume) = &request.resume {
        return resume_context(resume, contexts);
    }
    let container_count = count_containers(input.graph);
    let mode = effective_mode(&request.mode, input, container_count);
    let (text, items, truncated, dropped) = match &mode {
        RewindMode::Manifest | RewindMode::Auto => {
            let (text, dropped) = manifest_at(
                input.index,
                input.graph,
                input.changed_paths,
                &input.freshness,
                request.budget,
            );
            (text, Vec::new(), false, dropped)
        }
        RewindMode::Scope(_) | RewindMode::Files(_) => {
            let items = rank_with(
                input.index,
                input.graph,
                &mode,
                input.task_confirmation_weight,
            );
            let lines: Vec<String> = items.iter().map(render_item).collect();
            let budgeted = budget::apply(&lines, request.budget);
            (budgeted.text, items, budgeted.truncated, budgeted.dropped)
        }
    };
    let context_id = derive_id(&text);
    contexts.save(&context_id, &text)?;
    Ok(RewindOutput {
        context_id,
        text,
        items,
        truncated,
        dropped,
        embeddings_pending: input.freshness.pending,
        warnings: Vec::new(),
    })
}

fn resume_context(resume: &str, contexts: &ContextStore<'_>) -> Result<RewindOutput> {
    if !is_valid_context_id(resume) {
        return Err(Error::invalid_input(format!(
            "context_id inválido: {resume:?}"
        )));
    }
    let text = contexts
        .load(resume)?
        .ok_or_else(|| Error::not_found(format!("contexto ausente: {resume}")))?;
    Ok(RewindOutput {
        context_id: resume.to_string(),
        text,
        items: Vec::new(),
        truncated: false,
        dropped: 0,
        embeddings_pending: 0,
        warnings: Vec::new(),
    })
}

fn effective_mode(
    mode: &RewindMode,
    input: &RewindInput<'_>,
    container_count: usize,
) -> RewindMode {
    if mode != &RewindMode::Auto {
        return mode.clone();
    }
    if should_flip(input.index.docs.len(), container_count) {
        return RewindMode::Manifest;
    }
    match detect_scope(input.changed_paths, input.graph, input.index) {
        Some(container) => RewindMode::Scope(container),
        None => RewindMode::Manifest,
    }
}

fn count_containers(graph: &Graph) -> usize {
    graph
        .ids()
        .iter()
        .filter(|id| graph.note_type(id) == Some(NoteType::Epic))
        .count()
}
