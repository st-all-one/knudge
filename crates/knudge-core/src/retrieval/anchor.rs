//! Canal de âncoras (D81/D86).
//!
//! Âncora é **canal de recall**, não só um campo: o match por `path`/`id` é determinístico e
//! roda **antes** da estatística, com interseção ao working set. Globs suportam `?`, `*` (sem
//! `/`) e `**` (qualquer coisa, inclusive `/`).

use std::collections::BTreeSet;

use crate::retrieval::filter::Meta;
use crate::retrieval::index::Index;

/// Resultado do match de âncoras de uma nota.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "`file` e `id` são flags ortogonais do mesmo match"
)]
pub struct AnchorMatch {
    /// Quantas âncoras casaram (para ordenar).
    pub count: u32,
    /// Alguma âncora casou com um arquivo do working set.
    pub file: bool,
    /// Alguma âncora casou com um id do working set.
    pub id: bool,
}

/// Token de glob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Glob {
    /// `?` — um byte que não seja `/`.
    Any,
    /// `*` — zero ou mais bytes que não sejam `/`.
    Star,
    /// `**` — zero ou mais bytes quaisquer.
    DoubleStar,
    /// Byte literal.
    Lit(u8),
}

/// `true` se `pattern` casa com `text` (globs `?`, `*`, `**`).
#[must_use]
#[allow(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "matriz DP com índices limitados a `tokens.len()` e `text.len()`"
)]
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let tokens = glob_tokens(pattern);
    let bytes = text.as_bytes();
    let (rows, cols) = (tokens.len(), bytes.len());
    let mut dp = vec![vec![false; cols + 1]; rows + 1];
    dp[rows][cols] = true;
    for row in (0..rows).rev() {
        for col in (0..=cols).rev() {
            dp[row][col] = match tokens[row] {
                Glob::Any => col < cols && bytes[col] != b'/' && dp[row + 1][col + 1],
                Glob::Star => {
                    dp[row + 1][col] || (col < cols && bytes[col] != b'/' && dp[row][col + 1])
                }
                Glob::DoubleStar => dp[row + 1][col] || (col < cols && dp[row][col + 1]),
                Glob::Lit(byte) => col < cols && bytes[col] == byte && dp[row + 1][col + 1],
            };
        }
    }
    dp[0][0]
}

/// Casa as âncoras de uma nota contra o working set.
#[must_use]
pub fn match_note(meta: &Meta, working_paths: &[String], working_ids: &[String]) -> AnchorMatch {
    let mut result = AnchorMatch::default();
    for anchor in &meta.anchors {
        for path in working_paths {
            if glob_match(anchor, path) {
                result.file = true;
                result.count = result.count.saturating_add(1);
            }
        }
        for target in working_ids {
            if anchor == target || glob_match(anchor, target) {
                result.id = true;
                result.count = result.count.saturating_add(1);
            }
        }
    }
    result
}

/// Ranqueia as notas por número de âncoras casadas, com desempate por id.
#[must_use]
pub fn rank(
    index: &Index,
    allowed: &BTreeSet<String>,
    working_paths: &[String],
    working_ids: &[String],
) -> Vec<String> {
    let mut hits: Vec<(u32, &str)> = Vec::new();
    for doc in &index.docs {
        if !allowed.contains(&doc.meta.id) {
            continue;
        }
        let matched = match_note(&doc.meta, working_paths, working_ids);
        if matched.count > 0 {
            hits.push((matched.count, doc.meta.id.as_str()));
        }
    }
    hits.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    hits.into_iter().map(|(_, id)| id.to_string()).collect()
}

#[allow(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "avanço de índice sobre bytes do padrão, com `index < bytes.len()`"
)]
fn glob_tokens(pattern: &str) -> Vec<Glob> {
    let bytes = pattern.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'*' => {
                if bytes.get(index + 1) == Some(&b'*') {
                    tokens.push(Glob::DoubleStar);
                    index += 2;
                } else {
                    tokens.push(Glob::Star);
                    index += 1;
                }
            }
            b'?' => {
                tokens.push(Glob::Any);
                index += 1;
            }
            byte => {
                tokens.push(Glob::Lit(byte));
                index += 1;
            }
        }
    }
    tokens
}
