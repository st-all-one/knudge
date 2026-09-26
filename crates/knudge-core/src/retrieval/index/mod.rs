//! Índice derivado do retrieval (E06-T01).
//!
//! O índice é **derivado** de `notas/`: reconstruível byte a byte e descartável (D15/D27). Vive
//! em `.idx/retrieval.jsonl` (uma linha JSON por nota) e a troca é atômica via `tmp + rename`.
//! As estatísticas (df/avgdl) são **recomputadas ao carregar**, então nunca divergem do corpo.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::Result;
use crate::retrieval::filter::Meta;
use crate::retrieval::postings::Postings;
use crate::retrieval::token::tokenize;
use crate::store::{Note, Store};

mod persist;

pub use persist::size_warning;

/// Nome do arquivo do índice dentro de `.idx/`.
pub const INDEX_FILE: &str = "retrieval.jsonl";

/// Acima deste tamanho (bytes) o índice emite aviso (E06-T07).
pub const INDEX_WARN_BYTES: u64 = 8_388_608;

/// Campo indexado (IDF é calculado por campo — D37).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Field {
    /// `statement` — o campo que domina.
    Statement,
    /// Corpo.
    Body,
    /// Tags.
    Tags,
}

impl Field {
    /// Todos os campos, em ordem canônica.
    pub const ALL: [Self; 3] = [Self::Statement, Self::Body, Self::Tags];

    /// Rótulo de persistência.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Statement => "statement",
            Self::Body => "body",
            Self::Tags => "tags",
        }
    }

    /// Peso do campo na soma BM25 (`statement` domina — D37).
    #[must_use]
    pub const fn weight(self) -> f64 {
        match self {
            Self::Statement => 3.0,
            Self::Body => 1.0,
            Self::Tags => 2.0,
        }
    }

    /// Interpreta o rótulo de persistência.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.as_str() == text)
    }
}

/// Frequências de termos de um campo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldTf {
    /// termo → frequência.
    pub tf: BTreeMap<String, u32>,
    /// Número de tokens do campo.
    pub len: u32,
}

/// Documento indexado.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteDoc {
    /// Metadados (filtros, `why`, boost).
    pub meta: Meta,
    /// `statement` (para a saída do `recall`).
    pub statement: String,
    /// Campos e suas frequências.
    pub fields: BTreeMap<Field, FieldTf>,
}

impl NoteDoc {
    /// Indexa uma nota válida.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se o frontmatter estiver malformado.
    pub fn from_note(note: &Note) -> Result<Self> {
        let meta = Meta::from_frontmatter(&note.frontmatter)?;
        let statement = note.frontmatter.statement()?.to_string();
        let tags = meta.tags.join(" ");
        let mut fields = BTreeMap::new();
        fields.insert(Field::Statement, field_tf(&statement));
        fields.insert(Field::Body, field_tf(&note.body));
        fields.insert(Field::Tags, field_tf(&tags));
        Ok(Self {
            meta,
            statement,
            fields,
        })
    }

    /// Frequência do termo no campo (0 se ausente).
    #[must_use]
    pub fn tf(&self, field: Field, term: &str) -> u32 {
        self.fields
            .get(&field)
            .and_then(|entry| entry.tf.get(term))
            .copied()
            .unwrap_or(0)
    }

    /// Número de tokens do campo.
    #[must_use]
    pub fn len(&self, field: Field) -> u32 {
        self.fields.get(&field).map_or(0, |entry| entry.len)
    }
}

/// Estatísticas globais do corpus (recomputadas ao carregar).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stats {
    /// Número de documentos.
    pub n: u32,
    /// Comprimento médio por campo.
    pub avg_len: BTreeMap<Field, f64>,
    /// Document frequency por campo e termo.
    pub df: BTreeMap<Field, BTreeMap<String, u32>>,
}

/// Índice de retrieval.
#[derive(Debug, Default)]
pub struct Index {
    /// Documentos, ordenados por id.
    pub docs: Vec<NoteDoc>,
    /// Estatísticas globais.
    pub stats: Stats,
    /// Cache do índice invertido (derivado, construído sob demanda — E15-T06/O2.1).
    postings: OnceLock<Postings>,
}

impl Clone for Index {
    fn clone(&self) -> Self {
        Self::from_parts(self.docs.clone(), self.stats.clone())
    }
}

impl PartialEq for Index {
    fn eq(&self, other: &Self) -> bool {
        self.docs == other.docs && self.stats == other.stats
    }
}

impl Index {
    fn from_parts(docs: Vec<NoteDoc>, stats: Stats) -> Self {
        Self {
            docs,
            stats,
            postings: OnceLock::new(),
        }
    }

    /// Índice invertido em cache, construído na primeira consulta BM25 (E15-T06/O2.1).
    pub(crate) fn postings(&self) -> &Postings {
        self.postings.get_or_init(|| Postings::build(self))
    }

    /// Constrói o índice a partir de notas já parseadas.
    ///
    /// # Errors
    /// Propaga erros de extração de metadados.
    pub fn build(notes: &[Note]) -> Result<Self> {
        let mut docs = Vec::new();
        let _reserved = docs.try_reserve(notes.len());
        for note in notes {
            docs.push(NoteDoc::from_note(note)?);
        }
        docs.sort_unstable_by(|a, b| a.meta.id.cmp(&b.meta.id));
        let stats = compute_stats(&docs);
        Ok(Self::from_parts(docs, stats))
    }

    /// Constrói o índice lendo todas as notas do store.
    ///
    /// # Errors
    /// Propaga erros de listagem/leitura/parse.
    pub fn from_store(store: &Store<'_>) -> Result<Self> {
        let mut notes = Vec::new();
        for id in store.list_ids()? {
            if let Some(note) = store.read_optional(&id)? {
                notes.push(note);
            }
        }
        Self::build(&notes)
    }
}

fn field_tf(text: &str) -> FieldTf {
    let mut tf: BTreeMap<String, u32> = BTreeMap::new();
    let mut len = 0_u32;
    for token in tokenize(text) {
        len = len.saturating_add(1);
        let entry = tf.entry(token.into_owned()).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    FieldTf { tf, len }
}

#[allow(
    clippy::arithmetic_side_effects,
    reason = "médias do corpus em f64; somas e `n` não negativos"
)]
fn compute_stats(docs: &[NoteDoc]) -> Stats {
    let n = u32::try_from(docs.len()).unwrap_or(u32::MAX);
    let mut total: BTreeMap<Field, f64> = BTreeMap::new();
    let mut df: BTreeMap<Field, BTreeMap<String, u32>> = BTreeMap::new();
    for doc in docs {
        for field in Field::ALL {
            *total.entry(field).or_insert(0.0) += f64::from(doc.len(field));
            if let Some(entry) = doc.fields.get(&field) {
                for term in entry.tf.keys() {
                    let count = df
                        .entry(field)
                        .or_default()
                        .entry(term.clone())
                        .or_insert(0);
                    *count = count.saturating_add(1);
                }
            }
        }
    }
    let mut avg_len = BTreeMap::new();
    for field in Field::ALL {
        let sum = total.get(&field).copied().unwrap_or(0.0);
        let avg = if n == 0 { 0.0 } else { sum / f64::from(n) };
        avg_len.insert(field, avg);
    }
    Stats { n, avg_len, df }
}
