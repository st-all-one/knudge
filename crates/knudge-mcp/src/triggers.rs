//! Motor de gatilhos proativos do MCP (E12-T03, D68).
//!
//! Três gatilhos, **só ponteiros** (`id + statement + score`), cap **3** e dedup por sessão:
//! pré-`write` (quase-duplicados), pré-edição de arquivo (contexto do working set) e fim de
//! sessão (`learn` quando houve diff e zero writes). Nenhum hint injeta conteúdo.

use std::collections::BTreeSet;

use knudge_core::handoff::ManifestItem;
use knudge_core::maintenance::{LearnKind, LearnProposal};
use knudge_core::write::Candidate;

/// Cap default de hints por gatilho.
pub const DEFAULT_HINTS_CAP: usize = 3;

/// Gatilho proativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// Antes de gravar (quase-duplicados).
    PreWrite,
    /// Antes de editar arquivo (working set).
    PreEdit,
    /// Fim de sessão (`learn`).
    SessionEnd,
}

impl Trigger {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreWrite => "pre_write",
            Self::PreEdit => "pre_edit",
            Self::SessionEnd => "session_end",
        }
    }
}

/// Tipo do hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HintKind {
    /// Quase-duplicata: considere `update`/`compact`.
    Duplicate,
    /// Contexto relevante do working set.
    Context,
    /// Atividade sem registro (`learn` write-gap).
    WriteGap,
    /// Link faltante sugerido.
    MissingLink,
    /// Merge/supersede sugerido.
    Merge,
}

impl HintKind {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::Context => "context",
            Self::WriteGap => "write_gap",
            Self::MissingLink => "missing_link",
            Self::Merge => "merge",
        }
    }
}

/// Hint **ponteiro**: nunca carrega o corpo.
#[derive(Debug, Clone, PartialEq)]
pub struct Hint {
    /// Tipo.
    pub kind: HintKind,
    /// Ids referenciados.
    pub ids: Vec<String>,
    /// Score (maior = mais relevante).
    pub score: f64,
    /// Motivo curto.
    pub why: String,
    /// `true` quando o motor ainda está em modo observação.
    pub observed: bool,
}

/// Motor de hints com dedup por sessão e modo observação.
#[derive(Debug, Clone)]
pub struct HintEngine {
    cap: usize,
    observation_sessions: u32,
    sessions_seen: u32,
    seen: BTreeSet<String>,
}

impl HintEngine {
    /// Cria o motor com cap e número de sessões em observação.
    #[must_use]
    pub fn new(cap: usize, observation_sessions: u32) -> Self {
        Self {
            cap,
            observation_sessions,
            sessions_seen: 0,
            seen: BTreeSet::new(),
        }
    }

    /// `true` enquanto o motor está em observação.
    #[must_use]
    pub fn is_observing(&self) -> bool {
        self.sessions_seen < self.observation_sessions
    }

    /// Registra o fim de uma sessão.
    pub fn end_session(&mut self) {
        self.sessions_seen = self.sessions_seen.saturating_add(1);
        self.seen.clear();
    }

    /// Hints do gatilho pré-`write` (quase-duplicados).
    pub fn pre_write(&mut self, candidates: &[Candidate]) -> Vec<Hint> {
        let observed = self.is_observing();
        let hints = candidates.iter().map(|candidate| Hint {
            kind: HintKind::Duplicate,
            ids: vec![candidate.id.clone()],
            score: candidate.score,
            why: format!("quase-duplicata de {}", candidate.id),
            observed,
        });
        self.collect(hints)
    }

    /// Hints do gatilho pré-edição (contexto do working set).
    pub fn pre_edit(&mut self, items: &[ManifestItem]) -> Vec<Hint> {
        let observed = self.is_observing();
        let hints = items.iter().map(|item| Hint {
            kind: HintKind::Context,
            ids: vec![item.id.clone()],
            score: item.score,
            why: format!("contexto para {}", item.id),
            observed,
        });
        self.collect(hints)
    }

    /// Hints do gatilho fim de sessão (write-gap/merge/link via `learn`).
    pub fn session_end(&mut self, writes: usize, proposals: &[LearnProposal]) -> Vec<Hint> {
        if writes > 0 {
            return Vec::new();
        }
        let observed = self.is_observing();
        let hints = proposals.iter().map(|proposal| Hint {
            kind: hint_kind(proposal),
            ids: proposal.ids.clone(),
            score: proposal.score,
            why: proposal.why.clone(),
            observed,
        });
        self.collect(hints)
    }

    fn collect(&mut self, hints: impl Iterator<Item = Hint>) -> Vec<Hint> {
        let mut out = Vec::new();
        for hint in hints {
            if out.len() >= self.cap {
                break;
            }
            let key = dedup_key(&hint);
            if !self.seen.insert(key) {
                continue;
            }
            out.push(hint);
        }
        out
    }
}

fn hint_kind(proposal: &LearnProposal) -> HintKind {
    match proposal.kind {
        LearnKind::CreateNote => HintKind::WriteGap,
        LearnKind::Merge | LearnKind::Supersede => HintKind::Merge,
        LearnKind::Link => HintKind::MissingLink,
    }
}

fn dedup_key(hint: &Hint) -> String {
    format!("{}:{}", hint.kind.as_str(), hint.ids.join(","))
}
