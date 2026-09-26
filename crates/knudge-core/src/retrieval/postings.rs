//! Índice invertido em memória (derivado, nunca persistido) — E15-T06/O2.1.
//!
//! Reconstruído sob demanda a partir de [`Index`] e guardado em cache (`OnceLock`). Serve de
//! **peneira** para o BM25: o índice invertido nunca define a ordem da soma — isso mudaria o
//! último bit do `f64` serializado. A soma segue doc-major, field-major, term-major.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::retrieval::index::{Field, Index};

/// Índice invertido `Field → termo → posições` em [`Index::docs`] (posições ordenadas e únicas).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Postings {
    len: usize,
    by_field: BTreeMap<Field, BTreeMap<String, Vec<u32>>>,
}

impl Postings {
    /// Constrói o índice invertido a partir do índice (posições em `index.docs`).
    #[must_use]
    pub fn build(index: &Index) -> Self {
        let mut by_field: BTreeMap<Field, BTreeMap<String, Vec<u32>>> = BTreeMap::new();
        for (position, doc) in index.docs.iter().enumerate() {
            let Ok(position) = u32::try_from(position) else {
                break;
            };
            for field in Field::ALL {
                let Some(entry) = doc.fields.get(&field) else {
                    continue;
                };
                let field_map = by_field.entry(field).or_default();
                for term in entry.tf.keys() {
                    match field_map.get_mut(term.as_str()) {
                        Some(list) => list.push(position),
                        None => {
                            field_map.insert(term.clone(), vec![position]);
                        }
                    }
                }
            }
        }
        Self {
            len: index.docs.len(),
            by_field,
        }
    }

    /// Posições de documentos que compartilham ≥1 termo de conteúdo, em ordem **crescente**.
    ///
    /// O resultado é exatamente o conjunto de docs com `score_doc > 0` (o BM25 só soma quando há
    /// termo casado), o que preserva a ordem doc-major da varredura original.
    #[must_use]
    pub fn sieve(&self, terms: &[Cow<'_, str>]) -> Vec<u32> {
        let mut marked = vec![false; self.len];
        let mut positions = Vec::new();
        for terms_map in self.by_field.values() {
            for term in terms {
                let Some(list) = terms_map.get(term.as_ref()) else {
                    continue;
                };
                for &position in list {
                    let Ok(slot) = usize::try_from(position) else {
                        continue;
                    };
                    if let Some(flag) = marked.get_mut(slot)
                        && !*flag
                    {
                        *flag = true;
                        positions.push(position);
                    }
                }
            }
        }
        positions.sort_unstable();
        positions
    }
}
