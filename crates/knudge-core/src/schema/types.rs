//! Enums fechados do schema (D04/D13/D93/D149).
//!
//! Tipos abertos degradam o retrieval (o LLM inventa categorias); por isso `type`, `scope`,
//! `classification` e `status` são fechados e a evolução exige bump de `schema_version` (D14).

use std::fmt;
use std::str::FromStr;

use crate::{Error, Result};

/// Tipo de nota — **10 espécies** armazenadas, mais [`NoteType::Epic`] (grupo **derivado** de
/// `scope=epic`, nunca gravado — D149).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NoteType {
    /// Assertiva verificável.
    Fact,
    /// Escolha + justificativa.
    Decision,
    /// Aberto, não resolvido.
    Question,
    /// Ação pendente.
    Task,
    /// Termo → significado.
    Def,
    /// Problema + causa + correção.
    Error,
    /// Trecho de código reutilizável.
    Snippet,
    /// Referência externa + por quê.
    Link,
    /// Conhecimento sobre o próprio sistema.
    Meta,
    /// Conhecimento preditivo (probabilidade derivada).
    Risk,
    /// Grupo: nota com `scope=epic` e `type` **omitido** (D149). Não é gravável como `type`.
    Epic,
}

impl NoteType {
    /// Todos os tipos **armazenáveis**, na ordem canônica (D149: `Epic` não entra).
    pub const ALL: [Self; 10] = [
        Self::Fact,
        Self::Decision,
        Self::Question,
        Self::Task,
        Self::Def,
        Self::Error,
        Self::Snippet,
        Self::Link,
        Self::Meta,
        Self::Risk,
    ];

    /// Rótulo canônico (`fact`, `decision`, …).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Decision => "decision",
            Self::Question => "question",
            Self::Task => "task",
            Self::Def => "def",
            Self::Error => "error",
            Self::Snippet => "snippet",
            Self::Link => "link",
            Self::Meta => "meta",
            Self::Risk => "risk",
            Self::Epic => "epic",
        }
    }

    /// Prefixo declarativo do `id` (D02).
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        self.as_str()
    }

    /// `true` para o **grupo** (`scope=epic`, `type` omitido) — D149.
    #[must_use]
    pub const fn is_group(self) -> bool {
        matches!(self, Self::Epic)
    }

    /// `true` se o tipo **pode** carregar `scope` (item de trabalho ou grupo) — D93/D113/D149.
    #[must_use]
    pub const fn is_scoped(self) -> bool {
        self.is_work_kind() || self.is_group()
    }

    /// `true` se o tipo **exige** `scope` (D93/D149): grupo e a tarefa canônica.
    #[must_use]
    pub const fn requires_scope(self) -> bool {
        matches!(self, Self::Task | Self::Epic)
    }

    /// `true` se é **espécie de trabalho** (D113): aceita `scope` e não é grupo.
    #[must_use]
    pub const fn is_work_kind(self) -> bool {
        matches!(
            self,
            Self::Task | Self::Error | Self::Question | Self::Risk | Self::Decision
        )
    }
}

impl fmt::Display for NoteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NoteType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|note_type| note_type.as_str() == s)
            .ok_or_else(|| Error::schema(format!("tipo desconhecido: {s:?}")))
    }
}

/// Escopo de tarefa — enum fechado, só para `type` de trabalho/container (D93/D113/D134).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scope {
    /// Container-raiz (não tem pai).
    Epic,
    /// Agrupamento opcional entre épico e tarefa.
    Issue,
    /// Unidade de trabalho folha.
    Task,
}

impl Scope {
    /// Todos os escopos, do mais externo ao mais interno.
    pub const ALL: [Self; 3] = [Self::Epic, Self::Issue, Self::Task];

    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Epic => "epic",
            Self::Issue => "issue",
            Self::Task => "task",
        }
    }

    /// Rank nominal (1 = mais externo); a profundidade real vem da árvore (D134).
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Epic => 1,
            Self::Issue => 2,
            Self::Task => 3,
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Scope {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|scope| scope.as_str() == s)
            .ok_or_else(|| Error::schema(format!("scope desconhecido: {s:?}")))
    }
}

/// Classificação de maturidade (D44).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Classification {
    /// Estruturante, dificilmente muda.
    Foundational,
    /// Operacional, muda com o trabalho.
    #[default]
    Tactical,
    /// Observação pontual, pode expirar.
    Observational,
}

impl Classification {
    /// Todas as classificações, na ordem canônica.
    pub const ALL: [Self; 3] = [Self::Foundational, Self::Tactical, Self::Observational];

    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Foundational => "foundational",
            Self::Tactical => "tactical",
            Self::Observational => "observational",
        }
    }
}

impl fmt::Display for Classification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Classification {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.as_str() == s)
            .ok_or_else(|| Error::schema(format!("classification desconhecida: {s:?}")))
    }
}

/// Estado da nota — enum fechado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Status {
    /// Ativa.
    #[default]
    Active,
    /// Em progresso.
    InProgress,
    /// Bloqueada.
    Blocked,
    /// Encerrada.
    Closed,
    /// Substituída por outra.
    Superseded,
    /// Esquecida (soft-delete).
    Forgotten,
}

impl Status {
    /// Todos os estados, na ordem canônica.
    pub const ALL: [Self; 6] = [
        Self::Active,
        Self::InProgress,
        Self::Blocked,
        Self::Closed,
        Self::Superseded,
        Self::Forgotten,
    ];

    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Closed => "closed",
            Self::Superseded => "superseded",
            Self::Forgotten => "forgotten",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Status {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|value| value.as_str() == s)
            .ok_or_else(|| Error::schema(format!("status desconhecido: {s:?}")))
    }
}
