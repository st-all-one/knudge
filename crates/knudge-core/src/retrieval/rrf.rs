//! Fusão de canais por Reciprocal Rank Fusion (D81).
//!
//! `score(id) = Σ_canais 1/(k + rank + 1)` com `k=60` (config `recall.rrf_k`). O desempate é
//! **determinístico**: `(score desc, id asc)`. A fusão não depende do número de canais, então
//! um canal ausente apenas não soma — nunca quebra.

use std::collections::BTreeMap;

/// Hit fundido.
#[derive(Debug, Clone, PartialEq)]
pub struct Fused {
    /// Id do hit.
    pub id: String,
    /// Soma RRF dos canais.
    pub score: f64,
    /// Quantos canais contribuíram.
    pub channels: u32,
}

/// Funde listas de ids já ordenadas por rank.
#[must_use]
#[allow(
    clippy::arithmetic_side_effects,
    reason = "fórmula RRF com `k`, `rank` e score em f64 de domínio não negativo"
)]
pub fn fuse(channels: &[&[String]], k: u32) -> Vec<Fused> {
    let k = f64::from(k);
    let mut scores: BTreeMap<&str, (f64, u32)> = BTreeMap::new();
    for channel in channels {
        for (rank, id) in channel.iter().enumerate() {
            let rank = u32::try_from(rank).unwrap_or(u32::MAX);
            let weight = 1.0 / (k + f64::from(rank) + 1.0);
            let entry = scores.entry(id.as_str()).or_insert((0.0, 0));
            entry.0 += weight;
            entry.1 = entry.1.saturating_add(1);
        }
    }
    let mut fused: Vec<Fused> = scores
        .into_iter()
        .map(|(id, (score, channels))| Fused {
            id: id.to_string(),
            score,
            channels,
        })
        .collect();
    fused.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
    fused
}
