//! Vocabulário fechado de arestas (D49/D51/D207).
//!
//! Arestas explícitas são a **fonte primária** do grafo; a extração por regex é apenas
//! sugestão (D49/D50). Cada [`EdgeKind`] tem uma chave de frontmatter de mesmo nome cujo valor
//! é uma **lista de ids** (omitida quando vazia — D05). `superseded_by` é o ponteiro reverso de
//! [`EdgeKind::Replaces`] (D46). `same_as`/`broader`/`narrower`/`related` formam a **ontologia
//! leve** (SKOS-lite, D207): identidade e hierarquia de conceitos.

use std::fmt;
use std::str::FromStr;

use crate::{Error, Result};

/// Chaves canônicas das arestas no frontmatter, na ordem de [`EdgeKind::ALL`].
pub const EDGE_KEYS: [&str; 12] = [
    "references",
    "depends_on",
    "contradicts",
    "supports",
    "extends",
    "replaces",
    "rejects",
    "results_in",
    "same_as",
    "broader",
    "narrower",
    "related",
];

/// Tipo de aresta — enum fechado de 12 valores (D51/D207).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EdgeKind {
    /// Menção simples.
    References,
    /// Dependência transitiva (`blocked`/`ready` — E06).
    DependsOn,
    /// Contradição.
    Contradicts,
    /// Suporte/confirmação.
    Supports,
    /// Extensão/refinamento.
    Extends,
    /// Substituição (par reverso em `superseded_by` — D46).
    Replaces,
    /// Rejeição de alternativa.
    Rejects,
    /// Resultado de uma decisão.
    ResultsIn,
    /// Identidade de entidade (entity resolution — SKOS-lite, D207).
    SameAs,
    /// Conceito mais amplo (SKOS `broader`, D207).
    Broader,
    /// Conceito mais estreito (SKOS `narrower`, D207).
    Narrower,
    /// Associação livre (SKOS `related`, D207).
    Related,
}

impl EdgeKind {
    /// Todos os tipos, na ordem canônica.
    pub const ALL: [Self; 12] = [
        Self::References,
        Self::DependsOn,
        Self::Contradicts,
        Self::Supports,
        Self::Extends,
        Self::Replaces,
        Self::Rejects,
        Self::ResultsIn,
        Self::SameAs,
        Self::Broader,
        Self::Narrower,
        Self::Related,
    ];

    /// Rótulo canônico em `snake_case` (também a chave do frontmatter).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::References => "references",
            Self::DependsOn => "depends_on",
            Self::Contradicts => "contradicts",
            Self::Supports => "supports",
            Self::Extends => "extends",
            Self::Replaces => "replaces",
            Self::Rejects => "rejects",
            Self::ResultsIn => "results_in",
            Self::SameAs => "same_as",
            Self::Broader => "broader",
            Self::Narrower => "narrower",
            Self::Related => "related",
        }
    }

    /// Chave de frontmatter (igual ao rótulo).
    #[must_use]
    pub const fn key(self) -> &'static str {
        self.as_str()
    }

    /// `true` para a aresta que participa do ciclo de supersessão.
    #[must_use]
    pub const fn is_supersession(self) -> bool {
        matches!(self, Self::Replaces)
    }

    /// `true` para arestas da **ontologia leve** (SKOS-lite, D207).
    #[must_use]
    pub const fn is_ontology(self) -> bool {
        matches!(
            self,
            Self::SameAs | Self::Broader | Self::Narrower | Self::Related
        )
    }

    /// `true` para arestas **simétricas** (mesma semântica nos dois sentidos).
    #[must_use]
    pub const fn is_symmetric(self) -> bool {
        matches!(self, Self::SameAs | Self::Related)
    }

    /// Aresta **inversa** (`broader`↔`narrower`); as demais são a própria.
    #[must_use]
    pub const fn inverse(self) -> Self {
        match self {
            Self::Broader => Self::Narrower,
            Self::Narrower => Self::Broader,
            other => other,
        }
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EdgeKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.as_str() == s)
            .ok_or_else(|| {
                Error::schema(format!(
                    "aresta desconhecida: {s:?} ({})",
                    super::suggest::hint(s, &Self::ALL.map(Self::as_str))
                ))
            })
    }
}

/// Aresta explícita `from --kind--> to`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    /// Id de origem.
    pub from: String,
    /// Tipo da aresta.
    pub kind: EdgeKind,
    /// Id de destino.
    pub to: String,
}

impl Edge {
    /// Cria uma aresta.
    #[must_use]
    pub fn new(from: impl Into<String>, kind: EdgeKind, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            kind,
            to: to.into(),
        }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} --{}--> {}", self.from, self.kind, self.to)
    }
}
