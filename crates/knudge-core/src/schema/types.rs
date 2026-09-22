//! Enums fechados do schema (D04/D13/D93).
//!
//! Tipos abertos degradam o retrieval (o LLM inventa categorias); por isso `type`, `scope`,
//! `classification` e `status` são fechados e a evolução exige bump de `schema_version` (D14).

use std::fmt;
use std::str::FromStr;

use crate::{Error, Result};

/// Tipo de nota — enum fechado de 11 valores.
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
    /// Agregação explícita, sem verdade própria.
    Container,
    /// Conhecimento preditivo (`confidence` = probabilidade).
    Risk,
}

impl NoteType {
    /// Todos os tipos, na ordem canônica.
    pub const ALL: [Self; 11] = [
        Self::Fact,
        Self::Decision,
        Self::Question,
        Self::Task,
        Self::Def,
        Self::Error,
        Self::Snippet,
        Self::Link,
        Self::Meta,
        Self::Container,
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
            Self::Container => "container",
            Self::Risk => "risk",
        }
    }

    /// Prefixo declarativo do `id` (D02).
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        self.as_str()
    }

    /// `true` para os tipos que só existem como container de tarefa (D93).
    #[must_use]
    pub const fn is_container(self) -> bool {
        matches!(self, Self::Container)
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

/// Escopo de tarefa — enum fechado, só para `type ∈ {task, container}` (D93).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scope {
    /// Agregação raiz.
    Plan,
    /// Agregação de segundo nível.
    Epic,
    /// Unidade de trabalho com `type = task`.
    Issue,
    /// Unidade de trabalho folha com `type = task`.
    Task,
}

impl Scope {
    /// Todos os escopos, do mais externo ao mais interno.
    pub const ALL: [Self; 4] = [Self::Plan, Self::Epic, Self::Issue, Self::Task];

    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Epic => "epic",
            Self::Issue => "issue",
            Self::Task => "task",
        }
    }

    /// Profundidade 1-based na hierarquia `plan ⊃ epic ⊃ issue ⊃ task`.
    #[must_use]
    pub const fn depth(self) -> u8 {
        match self {
            Self::Plan => 1,
            Self::Epic => 2,
            Self::Issue => 3,
            Self::Task => 4,
        }
    }

    /// Escopo pai na hierarquia, se houver.
    #[must_use]
    pub const fn parent(self) -> Option<Self> {
        match self {
            Self::Plan => None,
            Self::Epic => Some(Self::Plan),
            Self::Issue => Some(Self::Epic),
            Self::Task => Some(Self::Issue),
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
