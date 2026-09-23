//! Vocabulário de tags (`kd ask --tags`, D107).
//!
//! Conta as tags declaradas no índice para o agente descobrir o vocabulário antes de escrever.
//! Notas `forgotten`/`superseded` ficam fora (D43); a ordem é `(count desc, tag asc)`.

use std::collections::BTreeMap;

use super::Index;
use crate::schema::Status;

/// Contagem de tags do índice, `(count desc, tag asc)`.
#[must_use]
pub fn tag_counts(index: &Index) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for doc in &index.docs {
        if matches!(doc.meta.status, Status::Forgotten | Status::Superseded) {
            continue;
        }
        for tag in &doc.meta.tags {
            let entry = counts.entry(tag.clone()).or_insert(0_usize);
            *entry = entry.saturating_add(1);
        }
    }
    let mut out: Vec<(String, usize)> = counts.into_iter().collect();
    out.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    out
}
