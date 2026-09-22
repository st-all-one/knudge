//! Escopo `write`: protocolo de escrita idempotente e dedup (E07).
//!
//! O `write` é **idempotente por conteúdo** (D01) e passa por duas fases: um `recall` ranqueia
//! candidatos e uma decisão calibrada (`<0.75` cria, `0.75–0.92` merge, `≥0.92` rejeita — D26).
//! A escrita é **estrita na forma** (chave/tipo desconhecidos rejeitados, opcionais omitidos) e
//! **tolerante na operação** (retry seguro, nada é apagado — D05/D16/D17/D52).

pub mod dedup;
pub mod draft;
pub mod lifecycle;
pub mod status;
pub mod update;

#[cfg(test)]
mod tests;

pub use dedup::{
    Candidate, DedupDecision, DedupThresholds, MergeProposal, WriteProposal, propose,
    propose_merges,
};
pub use draft::Draft;
pub use lifecycle::{forget, link, restore};
pub use status::validate_transition;
pub use update::{Patch, UpdateOutcome, history, update};

use crate::config::Config;
use crate::retrieval::Index;
use crate::schema::{Frontmatter, NoteType, Value};
use crate::store::{Event, EventLog, Note, Store, commit};
use crate::{Error, Result};

/// Ação resultante de um `write`/`update`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteAction {
    /// Nota nova gravada.
    Created,
    /// Conteúdo fundido em nota existente.
    Merged,
    /// Duplicata forte: rejeitado.
    Rejected,
    /// Nota existente atualizada.
    Updated,
    /// Retry idempotente: nada mudou.
    Unchanged,
}

impl WriteAction {
    /// Rótulo canônico (`created`, `merged`, `rejected`, `updated`, `unchanged`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Merged => "merged",
            Self::Rejected => "rejected",
            Self::Updated => "updated",
            Self::Unchanged => "unchanged",
        }
    }
}

/// Resultado de um `write`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteOutcome {
    /// Ação executada.
    pub action: WriteAction,
    /// Id resultante (candidato em merge/rejeição).
    pub id: String,
    /// Revisão, quando aplicável.
    pub revision: Option<u32>,
}

/// Contexto de escrita: store, eventos, índice e relógio (E07).
pub struct WriteContext<'a> {
    store: Store<'a>,
    events: EventLog<'a>,
    index: Index,
    now_ms: i64,
}

impl<'a> WriteContext<'a> {
    /// Monta o contexto.
    #[must_use]
    pub fn new(store: Store<'a>, events: EventLog<'a>, index: Index, now_ms: i64) -> Self {
        Self {
            store,
            events,
            index,
            now_ms,
        }
    }

    /// Store de notas.
    #[must_use]
    pub fn store(&self) -> &Store<'a> {
        &self.store
    }

    /// Log de eventos.
    #[must_use]
    pub fn events(&self) -> &EventLog<'a> {
        &self.events
    }

    /// Índice de retrieval (candidatos do dedup).
    #[must_use]
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// Instante da operação (ms).
    #[must_use]
    pub const fn now_ms(&self) -> i64 {
        self.now_ms
    }
}

/// Executa o protocolo completo: idempotência → dedup → commit.
///
/// `kd write` **não** cria `task`/`container` (D93): use `kd task`.
///
/// # Errors
/// - `ErrorKind::InvalidInput` para `task`/`container`;
/// - `ErrorKind::Conflict` se o id já existe com corpo diferente (use `update`);
/// - `ErrorKind::Schema` para rascunho inválido;
/// - propaga erros de I/O.
pub fn write(
    ctx: &WriteContext<'_>,
    draft: &Draft,
    thresholds: &DedupThresholds,
) -> Result<WriteOutcome> {
    if matches!(draft.note_type, NoteType::Task | NoteType::Container) {
        return Err(Error::invalid_input(format!(
            "`kd write` não cria `{}`; use `kd task` (D93)",
            draft.note_type
        )));
    }
    let note = draft.to_note(ctx.now_ms())?;
    let id = note.id()?.to_string();
    if ctx.store().exists(&id) {
        return existing(ctx, &id, &note);
    }
    let proposal = propose(ctx.index(), draft, thresholds)?;
    match proposal.decision {
        DedupDecision::Create => {
            let record = event("write", &id, ctx.now_ms(), WriteAction::Created, None);
            commit(ctx.store(), ctx.events(), &note, &record)?;
            Ok(WriteOutcome {
                action: WriteAction::Created,
                id,
                revision: Some(1),
            })
        }
        DedupDecision::Merge { candidate, .. } => {
            let revision = merge_into(ctx, &candidate, &note)?;
            Ok(WriteOutcome {
                action: WriteAction::Merged,
                id: candidate,
                revision: Some(revision),
            })
        }
        DedupDecision::Reject { candidate, score } => {
            let record = event(
                "write",
                &candidate,
                ctx.now_ms(),
                WriteAction::Rejected,
                Some(score),
            );
            ctx.events().append(&record)?;
            Ok(WriteOutcome {
                action: WriteAction::Rejected,
                id: candidate,
                revision: None,
            })
        }
    }
}

/// Limiares de dedup a partir da config efetiva.
///
/// # Errors
/// Retorna `ErrorKind::Config` se os limiares da config forem inconsistentes.
pub fn thresholds_from_config(config: &Config) -> Result<DedupThresholds> {
    DedupThresholds::from_config(config)
}

fn existing(ctx: &WriteContext<'_>, id: &str, incoming: &Note) -> Result<WriteOutcome> {
    let current = ctx.store().read(id)?;
    let same = current.frontmatter.get("body_hash") == incoming.frontmatter.get("body_hash");
    if !same {
        return Err(Error::conflict(format!(
            "nota {id} já existe com corpo diferente; use update"
        )));
    }
    let record = event("write", id, ctx.now_ms(), WriteAction::Unchanged, None);
    ctx.events().append(&record)?;
    Ok(WriteOutcome {
        action: WriteAction::Unchanged,
        id: id.to_string(),
        revision: Some(current.revision()),
    })
}

pub(crate) fn merge_into(ctx: &WriteContext<'_>, target_id: &str, incoming: &Note) -> Result<u32> {
    let mut target = ctx.store().read(target_id)?;
    let mut tags: Vec<String> = target
        .frontmatter
        .string_list("tags")?
        .into_iter()
        .map(str::to_string)
        .collect();
    for tag in incoming.frontmatter.string_list("tags")? {
        if !tags.iter().any(|existing| existing == tag) {
            tags.push(tag.to_string());
        }
    }
    set_list(&mut target.frontmatter, "tags", &tags)?;
    let mut anchors: Vec<String> = target
        .frontmatter
        .string_list("anchors")?
        .into_iter()
        .map(str::to_string)
        .collect();
    for anchor in incoming.frontmatter.string_list("anchors")? {
        if !anchors.iter().any(|existing| existing == anchor) {
            anchors.push(anchor.to_string());
        }
    }
    set_list(&mut target.frontmatter, "anchors", &anchors)?;
    if !incoming.body.is_empty() && !target.body.contains(incoming.body.as_str()) {
        if !target.body.is_empty() {
            target.body.push_str("\n\n");
        }
        target.body.push_str(&incoming.body);
    }
    let incoming_confidence = incoming.frontmatter.confidence()?;
    if incoming_confidence > target.frontmatter.confidence()? {
        target
            .frontmatter
            .set("confidence", Value::Float(incoming_confidence))?;
    }
    let revision = target.revision().saturating_add(1);
    target.set_revision(revision)?;
    target.refresh_body_hash()?;
    target.frontmatter.validate()?;
    ctx.store().write(&target)?;
    let record = event("update", target_id, ctx.now_ms(), WriteAction::Merged, None);
    ctx.events().append(&record)?;
    Ok(revision)
}

pub(crate) fn event(op: &str, id: &str, at: i64, action: WriteAction, score: Option<f64>) -> Event {
    let mut record = Event::new(op, at)
        .with_note_id(id)
        .with_data("action", Value::Str(action.as_str().to_string()));
    if let Some(score) = score {
        record = record.with_data("score", Value::Float(score));
    }
    record
}

fn set_list(frontmatter: &mut Frontmatter, key: &str, items: &[String]) -> Result<()> {
    if items.is_empty() {
        frontmatter.remove(key);
        return Ok(());
    }
    let value = Value::List(items.iter().map(|item| Value::Str(item.clone())).collect());
    frontmatter.set(key, value)
}
