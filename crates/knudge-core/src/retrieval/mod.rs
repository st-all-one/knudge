//! Escopo `retrieval`: BM25, âncoras, filtros, views e fusão RRF (E06).
//!
//! A cascata do `recall` vai do mais barato ao mais caro: **filtros determinísticos** → **BM25**
//! no resíduo → **âncoras** como canal → **fusão RRF** (D39/D41/D81). Canais ausentes/falhos
//! degradam para o lexical com `warnings`, nunca abortam (a menos que `strict` esteja ligado).

pub mod anchor;
pub mod bm25;
mod contradiction;
pub mod filter;
pub mod format;
pub mod index;
pub mod pipeline;
pub mod postings;
pub mod rank;
pub mod rrf;
pub mod snippet;
pub mod stem;
pub mod tags;
pub mod temporal;
pub mod token;
pub mod views;
pub mod weights;
pub mod why;

#[cfg(test)]
mod tests;

pub use bm25::{B, Bm25Hit, CONFIRMATION_STEP, K1, type_weight};
pub use filter::{Filter, Meta};
pub use format::{format_brief, format_hit};
pub use index::{
    Field, FieldTf, INDEX_FILE, INDEX_FORMAT, INDEX_WARN_BYTES, Index, NoteDoc, Stats, size_warning,
};
pub use postings::Postings;
pub use rank::{RankQuery, Universe, rank};
pub use rrf::{Channel, Fused, fuse};
pub use snippet::{body_matches, body_snippet};
pub use tags::tag_counts;
pub use temporal::{State, active_ids, state_at};
pub use views::{BlockReason, Views, block_reason, compute_views};
pub use weights::{
    DEFAULT_ANCHOR_WEIGHT, DEFAULT_LEXICAL_WEIGHT, DEFAULT_PPR_WEIGHT, DEFAULT_SEMANTIC_WEIGHT,
    FusionWeights,
};
pub use why::Why;

use std::collections::BTreeSet;

use crate::graph::Graph;
use crate::lifecycle::DriftIndex;
use crate::lifecycle::confidence::DEFAULT_TASK_CONFIRMATION;
use crate::store::{Note, Store};
use crate::{Error, Result};

use pipeline::{
    ChannelLabel, FusedChannels, build_hits, candidates, lexical_channel, ppr_channel,
    semantic_channel, task_confirmers,
};

/// `k` padrão da fusão RRF (config `recall.rrf_k`).
pub const DEFAULT_RRF_K: u32 = 60;

/// Limite padrão de hits (config `recall.default_limit`).
///
/// 5 (D121): contexto de LLM é caro e o `ask` devolvia hits demais. Ajuste por config.
pub const DEFAULT_LIMIT: usize = 5;

/// Corte de termos de alta frequência (config `recall.max_term_ratio`, D173).
///
/// Um termo presente em **≥** esta fração dos documentos é descartado do canal lexical. O IDF já
/// o atenua, mas o RRF usa *ranks*: um termo ubíquo ainda poderia puxar um rank-1 espúrio. `0`
/// desliga; conservador de propósito (só vale para corpora grandes — [`MIN_CUTOFF_CORPUS`]).
pub const DEFAULT_MAX_TERM_RATIO: f64 = 0.9;

/// Corpus mínimo (notas) para o corte de alta frequência valer (D173).
///
/// Abaixo disso, `df/N` é alto para quase todo termo e o corte derrubaria casamentos legítimos.
pub const MIN_CUTOFF_CORPUS: usize = 64;

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
    /// Pesos dos canais na fusão (D123).
    pub weights: FusionWeights,
    /// Filtros estruturais.
    pub filter: Filter,
    /// Universo considerado (D146): conhecimento por padrão, trabalho com `--with-task`.
    pub universe: Universe,
    /// Escopo pedido (pertencimento via `depends_on` transitivo).
    pub scope: Option<String>,
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
    /// Peso da confirmação derivada de tarefas (X1/D108; config
    /// `recall.confirmation_from_tasks`).
    pub task_confirmation_weight: f64,
    /// Drift de âncoras por id (`0` = em dia; ausente = `0`) — derivado `.idx/drift.jsonl` (D203).
    pub drift: DriftIndex,
    /// Corte de termos de alta frequência no canal lexical (D173; config
    /// `recall.max_term_ratio`); `0` desliga.
    pub max_term_ratio: f64,
    /// Promove `warnings` a erro (config `behavior.strict` — D94).
    pub strict: bool,
    /// Conjunto ativo em `as_of` (D155); `None` = corpus atual.
    pub as_of: Option<BTreeSet<String>>,
}

impl RecallQuery {
    /// Consulta com defaults de config (`rrf_k=60`, `limit=10`).
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            limit: DEFAULT_LIMIT,
            rrf_k: DEFAULT_RRF_K,
            universe: Universe::All,
            task_confirmation_weight: DEFAULT_TASK_CONFIRMATION,
            max_term_ratio: DEFAULT_MAX_TERM_RATIO,
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
    /// Contribuição de cada sinal na ordenação (D151).
    pub channels: HitChannels,
}

/// Contribuição de cada sinal para um hit do `recall` (D151).
///
/// `lexical`/`anchor`/`semantic` são as **parcelas RRF** (unidades do score fundido);
/// `recent`/`stars` são **boosts informativos** em `[0,1]` — não alteram a ordem por si
/// (a confirmação de tarefas X1/D108 já entra no canal lexical).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HitChannels {
    /// Parcela do canal lexical (BM25) na fusão RRF.
    pub lexical: f64,
    /// Parcela do canal de âncoras (working set) na fusão RRF.
    pub anchor: f64,
    /// Parcela do canal vetorial na fusão RRF (0 sem embeddings).
    pub semantic: f64,
    /// Parcela do canal de autoridade (`PageRank` personalizado — D192); 0 se desligado.
    pub ppr: f64,
    /// Fator de recência `(0,1]` (1 = recém-criada) — boost informativo.
    pub recent: f64,
    /// Confirmação derivada (`outcomes` + tarefas, X1/D108) em `[0,1]` — boost informativo.
    pub stars: f64,
    /// Fração do score lexical que veio do campo `body`, em `[0,1]` — informativo (D161).
    pub body: f64,
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
    let confirmers = task_confirmers(index);
    let weight = query.task_confirmation_weight;

    let lexical = lexical_channel(index, &allowed, query, &confirmers, weight);
    let anchored = anchor::rank(index, &allowed, &query.working_paths, &query.working_ids);
    let semantic = semantic_channel(query, &allowed);
    let ppr = ppr_channel(graph, &allowed, &anchored, query.weights.ppr);

    let mut channels: Vec<Channel<'_>> = vec![
        Channel::new(&lexical, query.weights.lexical),
        Channel::new(&anchored, query.weights.anchor),
    ];
    let mut labels = vec![ChannelLabel::Lexical, ChannelLabel::Anchor];
    if !semantic.is_empty() {
        channels.push(Channel::new(&semantic, query.weights.semantic));
        labels.push(ChannelLabel::Semantic);
    }
    if !ppr.is_empty() {
        channels.push(Channel::new(&ppr, query.weights.ppr));
        labels.push(ChannelLabel::Ppr);
    }
    let fused = fuse(&channels, query.rrf_k);

    let limit = if query.limit == 0 {
        usize::MAX
    } else {
        query.limit
    };
    let hits = {
        let semantic_ids: BTreeSet<&str> = semantic.iter().map(String::as_str).collect();
        build_hits(
            index,
            query,
            graph,
            &FusedChannels {
                fused: &fused,
                semantic: &semantic_ids,
                labels: &labels,
            },
            limit,
        )
    };

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
