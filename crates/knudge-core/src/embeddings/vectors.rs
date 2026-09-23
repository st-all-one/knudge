//! Resolução de vetores do dreno: cache, lote e isolamento (E11-T03/T04/T06/T10).
//!
//! Uma nota não-embeddável **não** pode travar a fila: quando o lote falha, o worker tenta as
//! notas individualmente, indexa as boas e mantém as ruins `pending` (R33/D83).

use crate::ErrorKind;
use crate::ports::Embedder;

use super::cache::EmbeddingCache;

/// Texto curto para sondar se o provedor está alcançável antes de isolar notas.
const PROBE_TEXT: &str = "knudge probe";

/// Nota pendente de digestão.
pub(super) struct PendingNote {
    pub(super) id: String,
    pub(super) body_hash: String,
    pub(super) text: String,
}

/// Vetor resolvido para uma nota: `(id, body_hash, vetor)`.
pub(super) type VectorRecord = (String, String, Vec<f32>);

/// Miss pendente de inferência: `(id, body_hash, texto)`.
type Miss = (String, String, String);

/// Resolve os vetores do lote: cache (hit pula inferência) + provedor para os *misses*.
pub(super) fn resolve_vectors(
    queue: &[PendingNote],
    cache: Option<&mut EmbeddingCache>,
    embedder: &dyn Embedder,
    now_ms: i64,
    warnings: &mut Vec<String>,
) -> (Vec<VectorRecord>, usize) {
    let mut resolver = Resolver::new(cache, embedder, now_ms, warnings);
    resolver.resolve(queue);
    (resolver.vectors, resolver.cache_hits)
}

/// Acumula vetores, cache e warnings enquanto resolve um lote.
struct Resolver<'a> {
    cache: Option<&'a mut EmbeddingCache>,
    embedder: &'a dyn Embedder,
    now_ms: i64,
    warnings: &'a mut Vec<String>,
    vectors: Vec<VectorRecord>,
    cache_hits: usize,
}

impl<'a> Resolver<'a> {
    fn new(
        cache: Option<&'a mut EmbeddingCache>,
        embedder: &'a dyn Embedder,
        now_ms: i64,
        warnings: &'a mut Vec<String>,
    ) -> Self {
        Self {
            cache,
            embedder,
            now_ms,
            warnings,
            vectors: Vec::new(),
            cache_hits: 0,
        }
    }

    fn resolve(&mut self, queue: &[PendingNote]) {
        let mut misses: Vec<Miss> = Vec::new();
        for note in queue {
            if let Some(cache) = self.cache.as_deref_mut()
                && let Some(vector) = cache.get(&note.body_hash)
            {
                self.cache_hits = self.cache_hits.saturating_add(1);
                self.vectors
                    .push((note.id.clone(), note.body_hash.clone(), vector));
                continue;
            }
            if let Some(cache) = self.cache.as_deref_mut() {
                cache.record_miss();
            }
            misses.push((note.id.clone(), note.body_hash.clone(), note.text.clone()));
        }
        if misses.is_empty() {
            return;
        }
        let texts: Vec<String> = misses.iter().map(|(_, _, text)| text.clone()).collect();
        match self.embedder.embed(&texts) {
            Ok(from_provider) if from_provider.len() == misses.len() => {
                for ((id, body_hash, _), vector) in misses.iter().zip(from_provider) {
                    self.record(id, body_hash, vector);
                }
            }
            Ok(_) => self.isolate(
                &misses,
                "provedor devolveu número de vetores diferente do pedido",
                None,
            ),
            Err(error) => {
                let kind = error.kind();
                let detail = format!("provedor de embeddings falhou: {error}");
                self.isolate(&misses, &detail, Some(kind));
            }
        }
    }

    fn record(&mut self, id: &str, body_hash: &str, vector: Vec<f32>) {
        let model = self.embedder.meta().model.as_str();
        if let Some(cache) = self.cache.as_deref_mut() {
            let _inserted = cache.insert(body_hash, vector.clone(), model, self.now_ms);
        }
        self.vectors
            .push((id.to_string(), body_hash.to_string(), vector));
    }

    /// Isola as notas de um lote que falhou: com um único miss, mantém `pending` (R33/D83);
    /// com vários, só tenta individualmente se o provedor estiver **alcançável** — um servidor
    /// fora do ar não deve virar uma tentativa (e um warning) por nota.
    fn isolate(&mut self, misses: &[Miss], detail: &str, kind: Option<ErrorKind>) {
        if misses.len() == 1 {
            self.warnings.push(detail.to_string());
            return;
        }
        let reachable = kind != Some(ErrorKind::Timeout) && self.provider_reachable();
        if !reachable {
            self.warnings.push(format!(
                "{detail}; provedor inalcançável, mantendo {} nota(s) pending",
                misses.len()
            ));
            return;
        }
        self.warnings.push(format!(
            "{detail}; tentando {} nota(s) individualmente para não travar a fila",
            misses.len()
        ));
        self.embed_individually(misses);
    }

    /// Sonda o provedor com um texto curto (não cacheado) para separar falha de conectividade
    /// de rejeição de uma nota específica.
    fn provider_reachable(&self) -> bool {
        self.embedder.embed(&[PROBE_TEXT.to_string()]).is_ok()
    }

    fn embed_individually(&mut self, misses: &[Miss]) {
        for (id, body_hash, text) in misses {
            match self.embedder.embed(std::slice::from_ref(text)) {
                Ok(from_provider) if from_provider.len() == 1 => {
                    let Some(vector) = from_provider.into_iter().next() else {
                        continue;
                    };
                    self.record(id, body_hash, vector);
                }
                Ok(_) => self.warnings.push(format!(
                    "provedor devolveu vetores inválidos para a nota {id}"
                )),
                Err(error) => self
                    .warnings
                    .push(format!("provedor de embeddings falhou para {id}: {error}")),
            }
        }
    }
}
