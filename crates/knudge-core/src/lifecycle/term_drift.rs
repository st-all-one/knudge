//! Drift de termos por KL/JS (E19-T10/D208).
//!
//! Compara a **distribuição de termos** de um tópico em duas janelas de tempo (por `created_at`):
//! `JS` alto ⇒ o vocabulário mudou ⇒ o conhecimento provavelmente envelheceu ⇒ candidato a
//! revisão/`prune`. Puro e determinístico (`BTreeMap`, base 2, suavização de Laplace); off-path
//! (só roda no `prune`/`doctor`).

use std::collections::BTreeMap;

use crate::Result;
use crate::retrieval::token::content_terms;
use crate::store::Note;

/// Suavização de Laplace para evitar `log(0)`/divisão por zero.
pub const SMOOTHING: f64 = 1e-9;

/// Distribuição normalizada de termos (probabilidade) de um conjunto de textos.
#[must_use]
pub fn term_distribution<'a>(texts: impl Iterator<Item = &'a str>) -> BTreeMap<String, f64> {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut total: u64 = 0;
    for text in texts {
        for term in content_terms(text) {
            let entry = counts.entry(term.into_owned()).or_insert(0);
            *entry = entry.saturating_add(1);
            total = total.saturating_add(1);
        }
    }
    if total == 0 {
        return BTreeMap::new();
    }
    let denominator = f64_from(total);
    counts
        .into_iter()
        .map(|(term, count)| (term, f64_from(count) / denominator))
        .collect()
}

/// Divergência KL `P‖Q` (base 2), suavizada; `0` se `P` for vazia.
#[must_use]
pub fn kl_divergence(p: &BTreeMap<String, f64>, q: &BTreeMap<String, f64>) -> f64 {
    let mut sum = 0.0;
    for (term, &probability) in p {
        if probability <= 0.0 {
            continue;
        }
        let reference = q.get(term).copied().unwrap_or(SMOOTHING);
        sum = probability.mul_add((probability / reference).log2(), sum);
    }
    sum
}

/// Divergência Jensen-Shannon (base 2, em `[0, 1]`) — simétrica e limitada.
#[must_use]
pub fn js_divergence(p: &BTreeMap<String, f64>, q: &BTreeMap<String, f64>) -> f64 {
    if p.is_empty() && q.is_empty() {
        return 0.0;
    }
    let mut mixture: BTreeMap<String, f64> = BTreeMap::new();
    for (term, &probability) in p {
        mixture.insert(term.clone(), probability / 2.0);
    }
    for (term, &probability) in q {
        *mixture.entry(term.clone()).or_insert(0.0) += probability / 2.0;
    }
    f64::midpoint(kl_divergence(p, &mixture), kl_divergence(q, &mixture))
}

/// Drift de um tópico: separa as notas por `created_at` (mediana) e mede a `JS`.
///
/// Devolve `None` quando há menos de 2 notas ou uma das janelas fica vazia.
///
/// # Errors
/// Propaga erro de leitura do frontmatter.
pub fn topic_drift(notes: &[Note]) -> Result<Option<f64>> {
    if notes.len() < 2 {
        return Ok(None);
    }
    let mut dated: Vec<(i64, String)> = Vec::with_capacity(notes.len());
    for note in notes {
        dated.push((note.frontmatter.created_at()?, note_text(note)));
    }
    dated.sort_by_key(|(created_at, _)| *created_at);
    let middle = dated.len() / 2;
    let (old, new) = dated.split_at(middle);
    if old.is_empty() || new.is_empty() {
        return Ok(None);
    }
    let old_distribution = term_distribution(old.iter().map(|(_, text)| text.as_str()));
    let new_distribution = term_distribution(new.iter().map(|(_, text)| text.as_str()));
    Ok(Some(js_divergence(&old_distribution, &new_distribution)))
}

/// Texto indexável de uma nota (afirmação + corpo).
fn note_text(note: &Note) -> String {
    let statement = note.frontmatter.statement().unwrap_or("");
    if note.body.is_empty() {
        statement.to_string()
    } else {
        format!("{statement} {}", note.body)
    }
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "contagens de termos cabem exatamente em f64 (< 2^53)"
)]
fn f64_from(value: u64) -> f64 {
    value as f64
}
