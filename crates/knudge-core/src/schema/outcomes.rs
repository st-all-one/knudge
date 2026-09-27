//! Estatísticas de `outcomes` (D189/D190).
//!
//! `outcomes` são ensaios de Bernoulli: `success` vale 1, `partial` vale 0,5 em cada lado,
//! `failure`/`abandoned` valem 1 falha. Daqui saem o posterior Beta (D189) e a revisão
//! espaçada do shelf-life (D190).

use crate::schema::{Frontmatter, Value};
use crate::time::Timestamp;

/// Estatísticas derivadas dos `outcomes` de uma nota.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct OutcomeStats {
    /// Sucessos: `success + partial*0.5`.
    pub successes: f64,
    /// Falhas: `failure + abandoned + partial*0.5`.
    pub failures: f64,
    /// Revisões de sucesso plenas (`success`) — base da estabilidade (D190).
    pub reviews: u32,
    /// Instante do último ensaio (ms), se houver `recorded_at` legível.
    pub last_ms: Option<i64>,
}

/// Extrai as estatísticas de `outcomes` do frontmatter (puro).
#[must_use]
pub fn outcome_stats(frontmatter: &Frontmatter) -> OutcomeStats {
    let Some(Value::List(items)) = frontmatter.get("outcomes") else {
        return OutcomeStats::default();
    };
    let mut stats = OutcomeStats::default();
    for item in items {
        let Some(map) = item.as_map() else {
            continue;
        };
        match map.get("status").and_then(Value::as_str) {
            Some("success") => {
                stats.successes += 1.0;
                stats.reviews = stats.reviews.saturating_add(1);
            }
            Some("partial") => {
                stats.successes += 0.5;
                stats.failures += 0.5;
            }
            Some("failure" | "abandoned") => stats.failures += 1.0,
            _ => {}
        }
        if let Some(ms) = map
            .get("recorded_at")
            .and_then(Value::as_str)
            .and_then(|text| text.parse::<Timestamp>().ok())
            .map(Timestamp::as_millis)
        {
            stats.last_ms = Some(stats.last_ms.map_or(ms, |previous| previous.max(ms)));
        }
    }
    stats
}
