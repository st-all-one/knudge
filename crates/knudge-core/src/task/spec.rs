//! Especificação de tarefa/container (E08-T07).

use crate::schema::{Classification, EdgeKind, NoteType, Scope, Status};
use crate::store::Note;
use crate::write::Draft;
use crate::{Error, Result};

use super::{hierarchy, membership};

/// Especificação de uma tarefa ou container (`plan`/`epic`/`issue`/`task`).
#[derive(Debug, Clone, PartialEq)]
pub struct TaskSpec {
    /// Escopo fechado (define `type` e profundidade).
    pub scope: Scope,
    /// Afirmação (≤ 120 escalares).
    pub statement: String,
    /// Corpo (contexto).
    pub body: String,
    /// Pai (membership por marcador).
    pub parent: Option<String>,
    /// Dependências (`depends_on`).
    pub depends_on: Vec<String>,
    /// Validators.
    pub checks: Vec<String>,
    /// Âncoras.
    pub anchors: Vec<String>,
    /// Ordem 1-based dentro do pai.
    pub blocks: Option<u32>,
    /// Maturidade.
    pub classification: Option<Classification>,
    /// Estado inicial.
    pub status: Option<Status>,
    /// Expiração (ms desde a época).
    pub expires_at: Option<i64>,
    /// Agendamento `not_before` (ms desde a época) — separado da expiração (D56).
    pub not_before: Option<i64>,
    /// Confiança `0..=1`.
    pub confidence: f64,
}

impl TaskSpec {
    /// Especificação com escopo e afirmação.
    #[must_use]
    pub fn new(scope: Scope, statement: impl Into<String>) -> Self {
        Self {
            scope,
            statement: statement.into(),
            body: String::new(),
            parent: None,
            depends_on: Vec::new(),
            checks: Vec::new(),
            anchors: Vec::new(),
            blocks: None,
            classification: None,
            status: None,
            expires_at: None,
            not_before: None,
            confidence: 0.7,
        }
    }

    /// Tipo derivado do escopo: `plan`/`epic` são `container`, `issue`/`task` são `task`.
    #[must_use]
    pub const fn note_type(&self) -> NoteType {
        match self.scope {
            Scope::Plan | Scope::Epic => NoteType::Container,
            Scope::Issue | Scope::Task => NoteType::Task,
        }
    }

    /// Converte em nota válida, gravando o marcador de pai no corpo.
    ///
    /// A validação do **escopo do pai** exige o store e fica em [`super::submit`].
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` para `blocks` inválido, `blocks` sem pai ou escopo inválido.
    pub fn to_note(&self, now_ms: i64) -> Result<Note> {
        hierarchy::validate_blocks(self.blocks)?;
        if self.blocks.is_some() && self.parent.is_none() {
            return Err(Error::schema("`blocks` exige `parent` (D53)"));
        }
        let mut draft = Draft::new(self.note_type(), &self.statement);
        draft.body = match &self.parent {
            Some(parent) => membership::set(&self.body, parent, self.blocks),
            None => membership::strip(&self.body),
        };
        draft.confidence = self.confidence;
        draft.scope = Some(self.scope);
        draft.classification = self.classification;
        draft.status = self.status;
        draft.checks.clone_from(&self.checks);
        draft.anchors.clone_from(&self.anchors);
        draft.expires_at = self.expires_at;
        draft.not_before = self.not_before;
        draft.edges = self
            .depends_on
            .iter()
            .map(|id| (EdgeKind::DependsOn, id.clone()))
            .collect();
        draft.to_note(now_ms)
    }
}
