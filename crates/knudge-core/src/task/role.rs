//! Papel derivado da árvore (D115).
//!
//! O papel exibido (Initiative/Epic/Feature/Story/Sub-task/Bug/Spike/Risk/Decision) é uma
//! **função pura** de `(scope, type, tem_filhos)` — nunca um campo. A escada de 4 níveis (D93)
//! continua sendo o **nível**; o papel é a leitura.

use crate::schema::{NoteType, Scope};

/// Papel exibido de um item de trabalho.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// `plan` (container raiz).
    Initiative,
    /// `epic` (container).
    Epic,
    /// `issue` com filhos.
    Feature,
    /// `issue` folha com `type=task`.
    Story,
    /// `task` sob `issue`.
    SubTask,
    /// `type=error`.
    Bug,
    /// `type=question`.
    Spike,
    /// `type=risk`.
    Risk,
    /// `type=decision`.
    Decision,
}

impl Role {
    /// Rótulo canônico (exibição; nunca armazenado).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Initiative => "Initiative",
            Self::Epic => "Epic",
            Self::Feature => "Feature",
            Self::Story => "Story",
            Self::SubTask => "Sub-task",
            Self::Bug => "Bug",
            Self::Spike => "Spike",
            Self::Risk => "Risk",
            Self::Decision => "Decision",
        }
    }
}

/// Papel de um item: a **espécie** manda; senão o **nível** (um `issue` com filhos é Feature).
#[allow(
    clippy::fn_params_excessive_bools,
    reason = "`has_children` é o único sinal derivado"
)]
#[must_use]
pub fn role(scope: Scope, kind: NoteType, has_children: bool) -> Role {
    match kind {
        NoteType::Error => Role::Bug,
        NoteType::Question => Role::Spike,
        NoteType::Risk => Role::Risk,
        NoteType::Decision => Role::Decision,
        _ => match scope {
            Scope::Plan => Role::Initiative,
            Scope::Epic => Role::Epic,
            Scope::Issue if has_children => Role::Feature,
            Scope::Issue => Role::Story,
            Scope::Task => Role::SubTask,
        },
    }
}
