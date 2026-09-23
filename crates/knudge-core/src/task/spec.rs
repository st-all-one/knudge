//! Especificação de tarefa/container (E08-T07).

use crate::schema::{Classification, EdgeKind, NoteType, Scope, Status};
use crate::store::Note;
use crate::write::Draft;
use crate::{Error, Result};

use super::{hierarchy, membership};

/// Especificação de uma tarefa ou container (`plan`/`epic`/`issue`/`task`).
#[derive(Debug, Clone, PartialEq)]
pub struct TaskSpec {
    /// Escopo fechado (define a profundidade e o `type` default).
    pub scope: Scope,
    /// Espécie (`type`), quando difere do default por escopo (D113).
    pub kind: Option<NoteType>,
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
    /// Tags declaradas.
    pub tags: Vec<String>,
    /// Proveniência (`source`).
    pub source: Option<String>,
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
            kind: None,
            statement: statement.into(),
            body: String::new(),
            parent: None,
            depends_on: Vec::new(),
            checks: Vec::new(),
            anchors: Vec::new(),
            tags: Vec::new(),
            source: None,
            blocks: None,
            classification: None,
            status: None,
            expires_at: None,
            not_before: None,
            confidence: 0.7,
        }
    }

    /// Tipo derivado: `kind` quando presente (D113); senão o default por escopo.
    ///
    /// `plan`/`epic` são `container`; `issue`/`task` são `task`. Uma espécie de trabalho
    /// (`error`/`question`/`risk`/`decision`) é válida só para `issue`/`task`.
    #[must_use]
    pub const fn note_type(&self) -> NoteType {
        match self.kind {
            Some(kind) => kind,
            None => match self.scope {
                Scope::Plan | Scope::Epic => NoteType::Container,
                Scope::Issue | Scope::Task => NoteType::Task,
            },
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
        validate_kind(self.scope, self.kind)?;
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
        draft.tags.clone_from(&self.tags);
        draft.source.clone_from(&self.source);
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

/// Espécies válidas para um item de trabalho (`--kind`) — relaxa D93 (D113).
pub const WORK_KINDS: [NoteType; 5] = [
    NoteType::Task,
    NoteType::Error,
    NoteType::Question,
    NoteType::Risk,
    NoteType::Decision,
];

/// `true` se `kind` é coerente com `scope`.
///
/// Containers (`plan`/`epic`) só podem ser `container`; itens de trabalho aceitam [`WORK_KINDS`].
///
/// # Errors
/// Retorna `ErrorKind::Schema` para combinação inválida.
pub fn validate_kind(scope: Scope, kind: Option<NoteType>) -> Result<()> {
    let Some(kind) = kind else {
        return Ok(());
    };
    let valid = match scope {
        Scope::Plan | Scope::Epic => kind == NoteType::Container,
        Scope::Issue | Scope::Task => kind.is_work_kind(),
    };
    if valid {
        Ok(())
    } else {
        Err(Error::schema(format!(
            "`kind {}` inválido para escopo `{}`",
            kind.as_str(),
            scope.as_str()
        )))
    }
}
