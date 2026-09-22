//! Escopo `embeddings`: provedor plugável, cache, fila e avaliação (E11).
//!
//! Vetores são **derivados e opcionais** (D42/D79). O sistema **nunca** bloqueia por embedding:
//! notas recém-criadas ficam *dark* até serem digeridas por uma fila derivada (D80), com cache
//! por `body_hash` (D83), purga em remoção (D84), flush coalescido (D85) e provedor determinístico
//! para testes (D89). O default consome um servidor local OpenAI-compatible (`llama-server` com
//! o GGUF — D101); sem inferência embutida no binário (R16/R43).

pub mod cache;
pub mod eval;
pub mod flush;
pub mod index;
pub mod lightweight;
pub mod meta;
pub mod pipeline;
pub mod semantic;
pub mod state;
pub mod vector;

#[cfg(test)]
mod tests;

pub use cache::{
    CACHE_FILE, CacheEntry, DEFAULT_MAX_BYTES as CACHE_DEFAULT_MAX_BYTES, EmbeddingCache,
};
pub use eval::{
    AbReport, EvalMetrics, GoldenCase, Winner, ab_compare, evaluate, mrr, ndcg_at_k, recall_at_k,
};
pub use flush::{DEFAULT_FLUSH_MS, FlushState};
pub use index::{EmbeddingIndex, INDEX_FILE as EMBEDDINGS_FILE, IndexedVector};
pub use lightweight::{DEFAULT_DIMENSIONS, LightweightEmbedder, embed as lightweight_embed};
pub use meta::{EmbeddingMeta, Similarity};
pub use pipeline::{DrainInput, DrainOutcome, drain, embedding_text};
pub use semantic::{
    DuplicatePair, LinkSuggestion, Neighbor, clusters, duplicate_pairs, link_suggestions,
    neighbors, unlinked_ids,
};
pub use state::{EmbeddingMode, EmbeddingState, classify, is_backlogged};
pub use vector::{cosine, dot, is_normalized, l2_norm, normalize, similarity};
