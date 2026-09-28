//! Sugestão de valor para conjuntos fechados (did-you-mean) — D212.
//!
//! Um flag com lista fixa de valores deve, ao receber algo inexistente, **listar as
//! possibilidades** e **destacar a mais provável**. A distância é Levenshtein (determinística,
//! `O(n·m)`), sem dependência nova.

/// Distância de edição (Levenshtein) entre `a` e `b`.
#[must_use]
pub fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0_usize; b.len().saturating_add(1)];
    for (row, left) in a.iter().enumerate() {
        if let Some(first) = current.first_mut() {
            *first = row.saturating_add(1);
        }
        for (column, right) in b.iter().enumerate() {
            let cost = usize::from(left != right);
            let insertion = current.get(column).copied().unwrap_or(0).saturating_add(1);
            let deletion = previous
                .get(column.saturating_add(1))
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
            let substitution = previous
                .get(column)
                .copied()
                .unwrap_or(0)
                .saturating_add(cost);
            if let Some(slot) = current.get_mut(column.saturating_add(1)) {
                *slot = insertion.min(deletion).min(substitution);
            }
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous.get(b.len()).copied().unwrap_or(0)
}

/// Valor mais provável de `valid` para `raw` (menor distância; empate mantém a ordem).
///
/// Só sugere quando a distância é pequena (`≤ 2` ou `≤ len/3`), para não "adivinhar" demais.
#[must_use]
pub fn closest<'a>(raw: &str, valid: &[&'a str]) -> Option<&'a str> {
    let raw_lower = raw.to_lowercase();
    let mut best: Option<(usize, &'a str)> = None;
    for candidate in valid {
        let distance = distance(&raw_lower, &candidate.to_lowercase());
        if best.is_none_or(|(current, _)| distance < current) {
            best = Some((distance, candidate));
        }
    }
    let (distance, candidate) = best?;
    let limit = core::cmp::max(2, raw_lower.chars().count() / 3);
    (distance <= limit).then_some(candidate)
}

/// Cauda de erro: `use: a, b, c` e, se houver, `; você quis dizer "x"?`.
#[must_use]
pub fn hint(raw: &str, valid: &[&str]) -> String {
    let list = valid.join(", ");
    match closest(raw, valid) {
        Some(suggestion) => format!("use: {list}; você quis dizer {suggestion:?}?"),
        None => format!("use: {list}"),
    }
}

/// Só a sugestão (para conjuntos grandes, onde listar tudo é inviável).
#[must_use]
pub fn did_you_mean(raw: &str, valid: &[&str]) -> String {
    match closest(raw, valid) {
        Some(suggestion) => format!(" (você quis dizer {suggestion:?}?)"),
        None => String::new(),
    }
}
