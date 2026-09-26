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

/// Padrão de glob compilado, reutilizável em muitos textos (E15-T07/O2.3).
///
/// Evita retokenizar o padrão e realocar a matriz DP a cada par (padrão, texto).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobPattern {
    tokens: Vec<Glob>,
}

impl GlobPattern {
    /// Compila `pattern` (globs `?`, `*`, `**`).
    #[must_use]
    pub fn new(pattern: &str) -> Self {
        Self {
            tokens: glob_tokens(pattern),
        }
    }

    /// `true` se o padrão casa com `text`.
    #[must_use]
    pub fn matches(&self, text: &str) -> bool {
        match_tokens(&self.tokens, text)
    }
}

/// `true` se `pattern` casa com `text` (globs `?`, `*`, `**`).
#[must_use]
pub fn glob_match(pattern: &str, text: &str) -> bool {
    GlobPattern::new(pattern).matches(text)
}

/// Casa tokens de glob contra `text` com uma matriz DP de **uma linha** (O(texto) de espaço).
///
/// Equivale à recorrência 2-D anterior (`dp[row][col]`), mas sem alocar `tokens.len()+1`
/// `Vec`s por par: o texto é percorrido da direita para a esquerda guardando `dp[row+1][col+1]`
/// num escalar.
fn match_tokens(tokens: &[Glob], text: &str) -> bool {
    let bytes = text.as_bytes();
    let cols = bytes.len();
    // `row[col]` representa `dp[row+1][col]`; `dp[rows]` só tem `dp[rows][cols] = true`.
    let mut row = vec![false; cols.saturating_add(1)];
    if let Some(last) = row.last_mut() {
        *last = true;
    }
    for token in tokens.iter().rev() {
        let Some(last) = row.last_mut() else {
            break;
        };
        // `dp[row][cols]`: só `*`/`**` casam o fim do texto.
        let mut old_right = *last;
        *last = matches!(token, Glob::Star | Glob::DoubleStar) && old_right;
        for col in (0..cols).rev() {
            let Some(old_here) = row.get(col).copied() else {
                break;
            };
            let right = row.get(col.saturating_add(1)).copied().unwrap_or(false);
            let value = match token {
                Glob::Any => bytes.get(col) != Some(&b'/') && old_right,
                Glob::Star => old_here || (bytes.get(col) != Some(&b'/') && right),
                Glob::DoubleStar => old_here || right,
                Glob::Lit(byte) => bytes.get(col) == Some(byte) && old_right,
            };
            if let Some(target) = row.get_mut(col) {
                *target = value;
            }
            old_right = old_here;
        }
    }
    row.first().copied().unwrap_or(false)
}

/// Casa as âncoras de uma nota contra o working set.
#[must_use]
pub fn match_note(meta: &Meta, working_paths: &[String], working_ids: &[String]) -> AnchorMatch {
    let mut result = AnchorMatch::default();
    for anchor in &meta.anchors {
        let pattern = GlobPattern::new(anchor);
        for path in working_paths {
            if pattern.matches(path) {
                result.file = true;
                result.count = result.count.saturating_add(1);
            }
        }
        for target in working_ids {
            if anchor == target || pattern.matches(target) {
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
    hits.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
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
