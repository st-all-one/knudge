//! Curva de esquecimento e revisão espaçada (D190, E19-T02).
//!
//! A retenção `R(t) = exp(-t/S)` decai com o tempo desde a última revisão; a estabilidade `S`
//! cresce a cada `outcome` de sucesso. O shelf-life usa o cruzamento `R = limiar` como prazo
//! (`ttl = S·ln(1/limiar)`), de modo que o prazo **cresce com as revisões**. Puro, determinístico
//! e sem dependência.

use crate::Result;
use crate::schema::outcome_stats;
use crate::store::Note;

use super::shelf_life::{DAY_MS, ShelfLife, origin_ms};

/// Limiar default de revisão: `R` abaixo disso ⇒ candidata a revisão/purga.
pub const DEFAULT_REVIEW_THRESHOLD: f64 = 0.5;

/// Crescimento default do prazo por revisão de sucesso (%).
pub const DEFAULT_GROWTH_PERCENT: i64 = 50;

/// Retenção `R(t) = limiar^(t/ttl)` em `(0,1]`; `ttl` não-positivo ⇒ 1 (nunca decai).
#[must_use]
pub fn retention(elapsed_days: f64, ttl_days: f64, threshold: f64) -> f64 {
    if ttl_days <= 0.0 {
        return 1.0;
    }
    threshold
        .clamp(1e-9, 1.0)
        .powf(elapsed_days.max(0.0) / ttl_days)
}

/// Estabilidade `S` (dias) tal que `R(ttl) = limiar`: `S = ttl/ln(1/limiar)`.
#[must_use]
pub fn stability_days(ttl_days: f64, threshold: f64) -> f64 {
    if ttl_days <= 0.0 {
        return 0.0;
    }
    ttl_days / (1.0 / threshold.clamp(1e-9, 1.0)).ln()
}

/// Retenção atual da nota (D190) — `1.0` quando a nota nunca expira.
///
/// # Errors
/// Retorna `ErrorKind::Schema` se os campos tipados estiverem malformados.
pub fn retention_for(
    note: &Note,
    now_ms: i64,
    policy: &ShelfLife,
    last_seen_ms: Option<i64>,
) -> Result<f64> {
    let stats = outcome_stats(&note.frontmatter);
    let Some(ttl) = policy.effective_ttl_days(note.frontmatter.classification()?, stats.reviews)
    else {
        return Ok(1.0);
    };
    let origin = origin_ms(note, policy, stats.last_ms, last_seen_ms);
    let elapsed = now_ms.saturating_sub(origin).max(0) / DAY_MS;
    Ok(retention(
        to_f64(elapsed),
        to_f64(ttl),
        DEFAULT_REVIEW_THRESHOLD,
    ))
}

/// `i64` não-negativo → `f64` (sem `as`, que é negado por lint).
fn to_f64(value: i64) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}
