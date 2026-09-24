//! Fusão de canais por Reciprocal Rank Fusion (D81/D123).
//!
//! `score(id) = Σ_canais  peso_canal / (k + rank + 1)` com `k=60` (config `recall.rrf_k`) e
//! **peso por canal** (config `recall.lexical_weight`/`anchor_weight`/`semantic_weight`, D123).
//! O desempate é **determinístico**: `(score desc, id asc)`. A fusão não depende do número de
//! canais, então um canal ausente apenas não soma — nunca quebra.

use std::collections::BTreeMap;

/// Canal de ranking com peso na fusão (D123).
#[derive(Debug, Clone, Copy)]
pub struct Channel<'a> {
    /// Ids já ordenados por rank.
    pub ids: &'a [String],
    /// Peso da contribuição do canal (1.0 = neutro).
    pub weight: f64,
}

impl<'a> Channel<'a> {
    /// Canal com peso explícito.
    #[must_use]
    pub const fn new(ids: &'a [String], weight: f64) -> Self {
        Self { ids, weight }
    }

    /// Canal com peso neutro (`1.0`).
    #[must_use]
    pub const fn uniform(ids: &'a [String]) -> Self {
        Self { ids, weight: 1.0 }
    }
}

/// Hit fundido.
#[derive(Debug, Clone, PartialEq)]
pub struct Fused {
    /// Id do hit.
    pub id: String,
    /// Soma RRF ponderada dos canais.
    pub score: f64,
    /// Quantos canais contribuíram.
    pub channels: u32,
    /// Parcela de cada canal de entrada, **na mesma ordem** de `channels` (D151).
    pub contribs: Vec<f64>,
}

/// Acumulador interno da fusão: score, nº de canais e parcela por canal.
struct Accum {
    score: f64,
    channels: u32,
    contribs: Vec<f64>,
}

/// Funde canais (com peso) por RRF.
#[must_use]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "fórmula RRF com `k`, `rank` e score em f64 de domínio não negativo"
)]
pub fn fuse(channels: &[Channel<'_>], k: u32) -> Vec<Fused> {
    let k = f64::from(k);
    let mut scores: BTreeMap<&str, Accum> = BTreeMap::new();
    for (index, channel) in channels.iter().enumerate() {
        for (rank, id) in channel.ids.iter().enumerate() {
            let rank = u32::try_from(rank).unwrap_or(u32::MAX);
            let weight = channel.weight / (k + f64::from(rank) + 1.0);
            let entry = scores.entry(id.as_str()).or_insert_with(|| Accum {
                score: 0.0,
                channels: 0,
                contribs: vec![0.0; channels.len()],
            });
            entry.score += weight;
            entry.channels = entry.channels.saturating_add(1);
            if let Some(slot) = entry.contribs.get_mut(index) {
                *slot += weight;
            }
        }
    }
    let mut fused: Vec<Fused> = scores
        .into_iter()
        .map(|(id, accum)| Fused {
            id: id.to_string(),
            score: accum.score,
            channels: accum.channels,
            contribs: accum.contribs,
        })
        .collect();
    fused.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
    fused
}
