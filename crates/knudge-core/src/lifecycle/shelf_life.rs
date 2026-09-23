//! Shelf-life por classificação (D44, E10-T01).
//!
//! `foundational` **nunca** expira; `tactical` e `observational` têm prazos configuráveis
//! (defaults conservadores). O `expires_at` **explícito** da nota sempre vence o prazo derivado
//! da classificação. A expiração é **derivada** (`created_at` + prazo), nunca gravada.

use crate::Result;
use crate::config::Config;
use crate::schema::Classification;
use crate::store::Note;

/// Milissegundos de um dia.
pub const DAY_MS: i64 = 86_400_000;

/// Política de shelf-life por classificação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::struct_field_names,
    reason = "o sufixo `days` documenta a unidade de cada prazo"
)]
pub struct ShelfLife {
    /// Dias de vida de `foundational` (`0` = nunca expira).
    pub foundational_days: i64,
    /// Dias de vida de `tactical` (`0` = nunca expira).
    pub tactical_days: i64,
    /// Dias de vida de `observational` (`0` = nunca expira).
    pub observational_days: i64,
}

impl Default for ShelfLife {
    fn default() -> Self {
        Self {
            foundational_days: 0,
            tactical_days: 365,
            observational_days: 30,
        }
    }
}

impl ShelfLife {
    /// Lê a política da config efetiva (defaults embutidos como fallback).
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let defaults = Self::default();
        Self {
            foundational_days: config
                .get_int("retention.foundational_days")
                .unwrap_or(defaults.foundational_days),
            tactical_days: config
                .get_int("retention.tactical_days")
                .unwrap_or(defaults.tactical_days),
            observational_days: config
                .get_int("retention.observational_days")
                .unwrap_or(defaults.observational_days),
        }
    }

    /// Prazo em dias de uma classificação (`None` = nunca expira).
    #[must_use]
    pub const fn ttl_days(&self, classification: Classification) -> Option<i64> {
        let days = match classification {
            Classification::Foundational => self.foundational_days,
            Classification::Tactical => self.tactical_days,
            Classification::Observational => self.observational_days,
        };
        if days > 0 { Some(days) } else { None }
    }

    /// Expiração derivada (`created_ms` + prazo) — `None` se nunca expira.
    #[must_use]
    pub fn derived_expiry(&self, classification: Classification, created_ms: i64) -> Option<i64> {
        self.ttl_days(classification)
            .map(|days| created_ms.saturating_add(days.saturating_mul(DAY_MS)))
    }
}

/// Expiração efetiva de uma nota: `expires_at` explícito ou prazo derivado da classificação.
///
/// # Errors
/// Retorna `ErrorKind::Schema` se os campos tipados estiverem malformados.
pub fn expiry_for(note: &Note, policy: &ShelfLife) -> Result<Option<i64>> {
    if let Some(explicit) = note.frontmatter.expires_at()? {
        return Ok(Some(explicit));
    }
    let created_ms = created_ms(note);
    Ok(policy.derived_expiry(note.frontmatter.classification()?, created_ms))
}

/// `true` se a nota expirou em `now_ms`.
///
/// # Errors
/// Retorna `ErrorKind::Schema` se os campos tipados estiverem malformados.
pub fn is_expired(note: &Note, now_ms: i64, policy: &ShelfLife) -> Result<bool> {
    Ok(expiry_for(note, policy)?.is_some_and(|expiry| now_ms >= expiry))
}

/// Ids de notas expiradas, ordenados.
///
/// # Errors
/// Retorna `ErrorKind::Schema` se alguma nota tiver campos malformados.
pub fn expired_ids(notes: &[Note], now_ms: i64, policy: &ShelfLife) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for note in notes {
        if is_expired(note, now_ms, policy)? {
            ids.push(note.id()?.to_string());
        }
    }
    ids.sort();
    Ok(ids)
}

/// Janela de aviso antes da expiração (dias).
pub const EXPIRING_GRACE_DAYS: i64 = 7;

/// Contagem de frescor do corpus (D106): expiradas, prestes a expirar e pendentes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Freshness {
    /// Notas já expiradas (fora da shelf-life).
    pub stale: usize,
    /// Notas que expiram dentro da janela de aviso.
    pub expiring: usize,
    /// Notas pendentes na fila de embeddings.
    pub pending: usize,
}

impl Freshness {
    /// Linha canônica `stale=N expiring=N pending=N`.
    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "stale={} expiring={} pending={}",
            self.stale, self.expiring, self.pending
        )
    }
}

/// Conta `stale`/`expiring` do corpus (o `pending` vem da fila de embeddings).
///
/// # Errors
/// Retorna `ErrorKind::Schema` se os campos tipados estiverem malformados.
pub fn freshness(
    notes: &[Note],
    now_ms: i64,
    policy: &ShelfLife,
    pending: usize,
) -> Result<Freshness> {
    let grace = EXPIRING_GRACE_DAYS.saturating_mul(DAY_MS);
    let mut stale = 0_usize;
    let mut expiring = 0_usize;
    for note in notes {
        let Some(expiry) = expiry_for(note, policy)? else {
            continue;
        };
        if now_ms >= expiry {
            stale = stale.saturating_add(1);
        } else if expiry.saturating_sub(now_ms) <= grace {
            expiring = expiring.saturating_add(1);
        }
    }
    Ok(Freshness {
        stale,
        expiring,
        pending,
    })
}

/// Idade da nota em dias (`>= 0`) a partir de `created_at`.
#[must_use]
pub fn age_days(note: &Note, now_ms: i64) -> i64 {
    let age_ms = now_ms.saturating_sub(created_ms(note));
    if age_ms <= 0 { 0 } else { age_ms / DAY_MS }
}

/// `created_at` da nota em ms (0 se ilegível).
fn created_ms(note: &Note) -> i64 {
    use crate::schema::Value;
    use crate::time::Timestamp;
    match note.frontmatter.get("created_at") {
        Some(Value::Int(ms)) => *ms,
        Some(Value::Str(text)) => text.parse::<Timestamp>().map_or(0, Timestamp::as_millis),
        _ => 0,
    }
}
