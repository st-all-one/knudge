//! Purga de inativos com histórico (D48, E10-T03).
//!
//! A aposentadoria (`retired_at`) é **derivada** dos eventos de auditoria (`forget`/`supersede`),
//! nunca gravada no frontmatter. O conteúdo só é removido após a **janela de retenção**; nunca
//! há hard-delete imediato, e a remoção purga o derivado (D84).

use crate::Result;
use crate::config::Config;
use crate::schema::Value;
use crate::store::{Event, EventLog, Store};

use super::shelf_life::DAY_MS;

/// Janela de retenção após a aposentadoria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retention {
    /// Dias de retenção antes da purga.
    pub retired_days: i64,
}

/// Dias de retenção default.
pub const DEFAULT_RETIRED_DAYS: i64 = 30;

impl Default for Retention {
    fn default() -> Self {
        Self {
            retired_days: DEFAULT_RETIRED_DAYS,
        }
    }
}

impl Retention {
    /// Lê a política da config efetiva.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        Self {
            retired_days: config
                .get_int("retention.retired_days")
                .unwrap_or(DEFAULT_RETIRED_DAYS),
        }
    }

    /// `true` se a janela de retenção venceu.
    #[must_use]
    pub fn is_due(self, retired_at: i64, now_ms: i64) -> bool {
        let deadline = retired_at.saturating_add(self.retired_days.saturating_mul(DAY_MS));
        now_ms >= deadline
    }
}

/// Aposentadoria derivada de um evento de auditoria.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retirement {
    /// Id da nota aposentada.
    pub id: String,
    /// Instante da aposentadoria (ms).
    pub retired_at: i64,
    /// Operação de origem (`forget`/`supersede`).
    pub reason: String,
}

/// Deriva as aposentadorias dos eventos, mantendo a **última** por nota.
#[must_use]
pub fn retirements(events: &[Event]) -> Vec<Retirement> {
    let mut latest: Vec<Retirement> = Vec::new();
    for event in events {
        if !matches!(event.op.as_str(), "forget" | "supersede") {
            continue;
        }
        let Some(id) = event.note_id.clone() else {
            continue;
        };
        let reason = match event.data.get("reason").and_then(Value::as_str) {
            Some(reason) => reason.to_string(),
            None => event.op.clone(),
        };
        let retirement = Retirement {
            id,
            retired_at: event.at,
            reason,
        };
        match latest
            .iter_mut()
            .find(|existing| existing.id == retirement.id)
        {
            Some(existing) if existing.retired_at <= retirement.retired_at => {
                *existing = retirement;
            }
            Some(_) => {}
            None => latest.push(retirement),
        }
    }
    latest.sort_by(|left, right| (&left.id, left.retired_at).cmp(&(&right.id, right.retired_at)));
    latest
}

/// Ids cuja janela de retenção venceu, ordenados.
#[must_use]
pub fn due_for_purge(retirements: &[Retirement], now_ms: i64, retention: Retention) -> Vec<String> {
    let mut ids: Vec<String> = retirements
        .iter()
        .filter(|retirement| retention.is_due(retirement.retired_at, now_ms))
        .map(|retirement| retirement.id.clone())
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// Purga o conteúdo das notas aposentadas cuja retenção venceu (nota + derivado).
///
/// # Errors
/// Propaga erros de leitura do log, de remoção e da purga do derivado.
pub fn purge_due(
    store: &Store<'_>,
    events: &EventLog<'_>,
    now_ms: i64,
    retention: Retention,
) -> Result<Vec<String>> {
    let (all, _warnings) = events.read_all()?;
    let mut purged = Vec::new();
    for id in due_for_purge(&retirements(&all), now_ms, retention) {
        if store.exists(&id) {
            store.remove(&id)?;
            purged.push(id);
        }
    }
    Ok(purged)
}
