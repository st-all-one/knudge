//! Papel derivado da árvore (D115/D134).
//!
//! O papel exibido (Epic/Feature/Story/Sub-task/Bug/Spike/Risk/Decision) é uma **função pura**
//! de `(profundidade, type, tem_filhos)` — nunca um campo. A profundidade é a da **árvore**
//! (derivada), não do `scope`: `epic` é a raiz (depth 0); o nível 1 é Feature (com filhos) ou
//! Story (folha); níveis mais fundos são Sub-task.

use crate::schema::NoteType;

/// Papel exibido de um item de trabalho.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// `epic` (container raiz, depth 0).
    Epic,
    /// Container no nível 1 (com filhos).
    Feature,
    /// Folha no nível 1 (sob o épico).
    Story,
    /// Folha em nível ≥ 2 (sob um issue).
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

/// Papel de um item: a **espécie** manda; senão a **profundidade na árvore** (D134).
#[allow(
    clippy::fn_params_excessive_bools,
    reason = "`has_children` é o único sinal derivado"
)]
#[must_use]
pub fn role(depth: usize, kind: NoteType, has_children: bool) -> Role {
    match kind {
        NoteType::Error => Role::Bug,
        NoteType::Question => Role::Spike,
        NoteType::Risk => Role::Risk,
        NoteType::Decision => Role::Decision,
        _ => match depth {
            0 => Role::Epic,
            1 if has_children => Role::Feature,
            1 => Role::Story,
            _ => Role::SubTask,
        },
    }
}
