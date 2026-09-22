//! Worker de reconcile da fila de embeddings (E11-T03/T04/T06/T10).
//!
//! O dreno é **explícito e off-path**: compara o `body_hash` das notas com o índice vetorial,
//! re-embeda o que falta do **corpo canônico**, usa o cache por `body_hash` e grava uma vez no
//! fim (flush coalescido — D85). Falha do provedor **não descarta nota**: ela continua `pending`
//! (D83) e o resultado é parcial com `warnings` (R33).

use crate::Result;
use std::path::PathBuf;

use crate::config::Config;
use crate::ports::{Embedder, Fs};
use crate::schema::Value;
use crate::store::{Note, Store};

use super::cache::{CACHE_FILE, DEFAULT_MAX_BYTES, EmbeddingCache};
use super::flush::FlushState;
use super::index::EmbeddingIndex;
use super::state::{EmbeddingMode, EmbeddingState, is_backlogged};

/// Entrada do worker de dreno.
pub struct DrainInput<'a> {
    /// Store de notas (a verdade).
    pub store: &'a Store<'a>,
    /// Provedor de vetores.
    pub embedder: &'a dyn Embedder,
    /// Config efetiva.
    pub config: &'a Config,
    /// Instante atual em milissegundos.
    pub now_ms: i64,
}

/// Resultado de um dreno.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DrainOutcome {
    /// Vetores indexados neste dreno.
    pub indexed: usize,
    /// Notas ainda `pending`/`stale` depois do dreno.
    pub pending: usize,
    /// Notas `stale` observadas na fila.
    pub stale: usize,
    /// Inferências evitadas pelo cache.
    pub cache_hits: usize,
    /// Avisos de degradação graciosa.
    pub warnings: Vec<String>,
}

/// Vetor resolvido para uma nota: `(id, body_hash, vetor)`.
type VectorRecord = (String, String, Vec<f32>);

/// Nota pendente de digestão.
struct PendingNote {
    id: String,
    body_hash: String,
    text: String,
}

/// Configurações derivadas da config efetiva.
struct Settings {
    cache_enabled: bool,
    cache_path: PathBuf,
    max_bytes: usize,
    ttl_ms: i64,
    batch: usize,
    max_pending: usize,
}

impl Settings {
    fn from_input(input: &DrainInput<'_>) -> Result<Self> {
        let _mode =
            EmbeddingMode::parse(input.config.get_str("embeddings.mode").unwrap_or("lazy"))?;
        let ttl_days = input
            .config
            .get_int("embeddings.cache_ttl_days")
            .unwrap_or(0)
            .max(0);
        Ok(Self {
            cache_enabled: input.config.get_bool("embeddings.cache").unwrap_or(true),
            cache_path: input.store.root().join(".idx").join(CACHE_FILE),
            max_bytes: cache_max_bytes(input.config),
            ttl_ms: ttl_days.saturating_mul(86_400_000),
            batch: positive(input.config.get_int("embeddings.batch"), 32),
            max_pending: non_negative(input.config.get_int("embeddings.max_pending"), 1000),
        })
    }

    fn load_cache(&self, fs: &dyn Fs, warnings: &mut Vec<String>) -> EmbeddingCache {
        if self.cache_enabled {
            EmbeddingCache::load(fs, &self.cache_path, self.max_bytes, warnings)
        } else {
            EmbeddingCache::new(self.max_bytes)
        }
    }
}

/// Texto canônico que vai ao provedor (statement + corpo, sem frontmatter).
#[must_use]
pub fn embedding_text(statement: &str, body: &str) -> String {
    if body.trim().is_empty() {
        statement.to_string()
    } else {
        format!("{statement}\n{body}")
    }
}

/// Drena um lote da fila (nunca descarta nota; falha vira `pending`).
///
/// # Errors
/// Retorna `ErrorKind::Io`/`Config` para falha de leitura/escrita/config.
pub fn drain(input: &DrainInput<'_>) -> Result<DrainOutcome> {
    let fs = input.store.fs();
    let root = input.store.root();
    let meta = input.embedder.meta().clone();
    let mut warnings = Vec::new();
    let settings = Settings::from_input(input)?;

    let mut index = EmbeddingIndex::load(fs, root, &meta, &mut warnings)?
        .unwrap_or_else(|| EmbeddingIndex::new(meta.clone()));
    let mut cache = settings.load_cache(fs, &mut warnings);
    let _pruned = cache.prune_expired(input.now_ms, settings.ttl_ms);

    let (mut queue, stale) = pending_queue(input.store, &index, &mut warnings)?;
    let pending_before = queue.len();
    let backlogged = is_backlogged(pending_before, settings.max_pending);
    if backlogged {
        warnings.push(format!(
            "fila de embeddings acima de max_pending ({pending_before} > {}); catch-up forçado",
            settings.max_pending
        ));
    }
    queue.truncate(if backlogged {
        pending_before
    } else {
        settings.batch
    });

    let cache_ref = if settings.cache_enabled {
        Some(&mut cache)
    } else {
        None
    };
    let (vectors, cache_hits) = resolve_vectors(
        &queue,
        cache_ref,
        input.embedder,
        input.now_ms,
        &mut warnings,
    );
    let indexed = vectors.len();
    for (id, body_hash, vector) in vectors {
        index.insert(&id, &body_hash, vector);
    }

    if indexed > 0 {
        let mut flush = FlushState::new(input.now_ms);
        flush.mark_dirty();
        index.save(fs, root, &mut warnings)?;
        if settings.cache_enabled {
            cache.save(fs, &settings.cache_path)?;
        }
        flush.force(input.now_ms);
    }

    Ok(DrainOutcome {
        indexed,
        pending: pending_before.saturating_sub(indexed),
        stale,
        cache_hits,
        warnings,
    })
}

/// Resolve os vetores do lote: cache (hit pula inferência) + provedor para os *misses*.
fn resolve_vectors(
    queue: &[PendingNote],
    cache: Option<&mut EmbeddingCache>,
    embedder: &dyn Embedder,
    now_ms: i64,
    warnings: &mut Vec<String>,
) -> (Vec<VectorRecord>, usize) {
    let model = embedder.meta().model.as_str();
    let mut vectors: Vec<VectorRecord> = Vec::new();
    let mut cache_hits = 0_usize;
    let mut misses: Vec<String> = Vec::new();
    let mut miss_meta: Vec<(String, String)> = Vec::new();
    let mut cache = cache;
    for note in queue {
        if let Some(cache) = cache.as_deref_mut()
            && let Some(vector) = cache.get(&note.body_hash)
        {
            cache_hits = cache_hits.saturating_add(1);
            vectors.push((note.id.clone(), note.body_hash.clone(), vector));
            continue;
        }
        if let Some(cache) = cache.as_deref_mut() {
            cache.record_miss();
        }
        misses.push(note.text.clone());
        miss_meta.push((note.id.clone(), note.body_hash.clone()));
    }
    if misses.is_empty() {
        return (vectors, cache_hits);
    }
    match embedder.embed(&misses) {
        Ok(from_provider) if from_provider.len() == miss_meta.len() => {
            for ((id, body_hash), vector) in miss_meta.iter().zip(from_provider) {
                if let Some(cache) = cache.as_deref_mut() {
                    let _inserted = cache.insert(body_hash, vector.clone(), model, now_ms);
                }
                vectors.push((id.clone(), body_hash.clone(), vector));
            }
        }
        Ok(_) => warnings.push("provedor devolveu número de vetores diferente do pedido".into()),
        Err(error) => warnings.push(format!("provedor de embeddings falhou: {error}")),
    }
    (vectors, cache_hits)
}

fn pending_queue(
    store: &Store<'_>,
    index: &EmbeddingIndex,
    warnings: &mut Vec<String>,
) -> Result<(Vec<PendingNote>, usize)> {
    let mut queue = Vec::new();
    let mut stale = 0_usize;
    for id in store.list_ids()? {
        let note = match store.read(&id) {
            Ok(note) => note,
            Err(error) => {
                warnings.push(format!("nota ilegível {id}: {error}"));
                continue;
            }
        };
        let Some(body_hash) = note_body_hash(&note) else {
            warnings.push(format!("nota sem body_hash: {id}"));
            continue;
        };
        let embedding_state = index.state_of(&id, body_hash);
        if embedding_state == EmbeddingState::Indexed {
            continue;
        }
        let Ok(statement) = note.frontmatter.statement() else {
            warnings.push(format!("nota sem statement: {id}"));
            continue;
        };
        if embedding_state == EmbeddingState::Stale {
            stale = stale.saturating_add(1);
        }
        queue.push(PendingNote {
            id: id.clone(),
            body_hash: body_hash.to_string(),
            text: embedding_text(statement, &note.body),
        });
    }
    Ok((queue, stale))
}

fn note_body_hash(note: &Note) -> Option<&str> {
    note.frontmatter.get("body_hash").and_then(Value::as_str)
}

fn cache_max_bytes(config: &Config) -> usize {
    config
        .get_int("embeddings.cache_max_bytes")
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(DEFAULT_MAX_BYTES)
}

fn positive(value: Option<i64>, default: i64) -> usize {
    let raw = value.unwrap_or(default).max(1);
    usize::try_from(raw).unwrap_or(1)
}

fn non_negative(value: Option<i64>, default: i64) -> usize {
    let raw = value.unwrap_or(default).max(0);
    usize::try_from(raw).unwrap_or(0)
}
