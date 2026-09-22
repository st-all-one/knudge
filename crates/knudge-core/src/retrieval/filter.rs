//! Filtros determinísticos aplicados **antes** da estatística (D41/D53).
//!
//! Filtrar é O(1) por nota e reduz `N` antes do BM25: primeiro estrutura, depois similaridade.
//! O `container` é resolvido no grafo (via `depends_on` transitivo) e não entra aqui.

use crate::Result;
use crate::retrieval::anchor::glob_match;
use crate::schema::{Classification, Frontmatter, NoteType, Status, Value};
use crate::time::Timestamp;

/// Metadados de uma nota usados por filtros, `why` e boost.
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    /// Id da nota.
    pub id: String,
    /// Tipo fechado.
    pub note_type: NoteType,
    /// Classificação de maturidade.
    pub classification: Classification,
    /// Estado.
    pub status: Status,
    /// Tags declaradas.
    pub tags: Vec<String>,
    /// Âncoras declaradas (paths/globs).
    pub anchors: Vec<String>,
    /// `created_at` em milissegundos desde a época (0 se ilegível).
    pub created_ms: i64,
    /// Confirmação derivada de `outcomes`: `success + partial*0.5` (D38).
    pub confirmation: f64,
}

impl Meta {
    /// Extrai os metadados de um frontmatter válido.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se os campos tipados estiverem malformados.
    pub fn from_frontmatter(frontmatter: &Frontmatter) -> Result<Self> {
        Ok(Self {
            id: frontmatter.id()?.to_string(),
            note_type: frontmatter.note_type()?,
            classification: frontmatter.classification()?,
            status: frontmatter.status()?,
            tags: string_list(frontmatter, "tags"),
            anchors: string_list(frontmatter, "anchors"),
            created_ms: created_ms(frontmatter),
            confirmation: confirmation(frontmatter),
        })
    }
}

/// Filtros estruturais da consulta.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// Tipos aceitos (vazio = todos).
    pub types: Vec<NoteType>,
    /// Classificações aceitas (vazio = todas).
    pub classifications: Vec<Classification>,
    /// Status aceitos (vazio = todos).
    pub statuses: Vec<Status>,
    /// Tags exigidas (basta uma).
    pub tags: Vec<String>,
    /// Âncoras (path/glob) exigidas (basta uma). O match aceita as duas direções: o
    /// valor pedido como glob sobre a âncora da nota (`V2/**` casa `V2/x.rs`) ou a âncora
    /// da nota como glob sobre o valor pedido (`src/**` casa `src/x.rs`) — alinhado ao
    /// canal de âncoras (`retrieval/anchor.rs`) e a `rewind --files`.
    pub anchors: Vec<String>,
}

impl Filter {
    /// Cria um filtro vazio (aceita tudo).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` se nenhum critério foi definido.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
            && self.classifications.is_empty()
            && self.statuses.is_empty()
            && self.tags.is_empty()
            && self.anchors.is_empty()
    }

    /// `true` se a nota passa por todos os critérios definidos.
    #[must_use]
    pub fn matches(&self, meta: &Meta) -> bool {
        (self.types.is_empty() || self.types.contains(&meta.note_type))
            && (self.classifications.is_empty()
                || self.classifications.contains(&meta.classification))
            && (self.statuses.is_empty() || self.statuses.contains(&meta.status))
            && (self.tags.is_empty() || meta.tags.iter().any(|tag| self.tags.contains(tag)))
            && (self.anchors.is_empty()
                || meta
                    .anchors
                    .iter()
                    .any(|anchor| self.anchors.iter().any(|req| anchor_matches(req, anchor))))
    }
}

/// `true` se o valor pedido casa a âncora da nota em qualquer direção (D81).
///
/// Aceita o pedido como glob (`V2/**` casa `V2/x.rs`) e a âncora como glob
/// (`src/**` casa `src/x.rs`), para que `ask --anchor <path>` se comporte como o canal.
fn anchor_matches(requested: &str, anchor: &str) -> bool {
    glob_match(requested, anchor) || glob_match(anchor, requested)
}

fn string_list(frontmatter: &Frontmatter, key: &str) -> Vec<String> {
    frontmatter
        .string_list(key)
        .map(|items| items.into_iter().map(str::to_string).collect())
        .unwrap_or_default()
}

fn created_ms(frontmatter: &Frontmatter) -> i64 {
    match frontmatter.get("created_at") {
        Some(Value::Int(ms)) => *ms,
        Some(Value::Str(text)) => text.parse::<Timestamp>().map_or(0, Timestamp::as_millis),
        _ => 0,
    }
}

fn confirmation(frontmatter: &Frontmatter) -> f64 {
    let Some(Value::List(items)) = frontmatter.get("outcomes") else {
        return 0.0;
    };
    let mut score = 0.0;
    for item in items {
        let Some(map) = item.as_map() else {
            continue;
        };
        match map.get("status").and_then(Value::as_str) {
            Some("success") => score += 1.0,
            Some("partial") => score += 0.5,
            _ => {}
        }
    }
    score
}
