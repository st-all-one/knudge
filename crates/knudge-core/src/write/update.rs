//! `update` versionado e supersede caminhável (D01/D21/D48).
//!
//! Se a **chave de conteúdo** (`type` + `statement`) muda, a nota não é sobrescrita: cria-se uma
//! nova (novo `id`) com `replaces` e a antiga vira `superseded` com `superseded_by` (D01). Caso
//! contrário, a edição acontece no lugar e `revision` incrementa.

use std::collections::BTreeSet;

use crate::graph;
use crate::schema::{Classification, EdgeKind, NoteType, Scope, Status, Value, id};
use crate::store::{Note, Store};
use crate::time::Timestamp;
use crate::{Error, Result};

use super::status::validate_transition;
use super::{WriteAction, WriteContext, event, set_list};

/// Campos mutáveis por [`update`] (ausente = não mexe).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Patch {
    /// Novo tipo (mudança dispara supersede).
    pub note_type: Option<NoteType>,
    /// Nova afirmação (mudança dispara supersede).
    pub statement: Option<String>,
    /// Novo corpo.
    pub body: Option<String>,
    /// Nova confiança.
    pub confidence: Option<f64>,
    /// Novas tags (vazio limpa).
    pub tags: Option<Vec<String>>,
    /// Nova classificação.
    pub classification: Option<Classification>,
    /// Novo status (validado por [`validate_transition`]).
    pub status: Option<Status>,
    /// Novo escopo (só `task`/`container`).
    pub scope: Option<Scope>,
    /// Novas âncoras (vazio limpa).
    pub anchors: Option<Vec<String>>,
    /// Nova proveniência.
    pub source: Option<String>,
    /// Nova expiração (ms).
    pub expires_at: Option<i64>,
    /// Novo agendamento `not_before` (ms) — separado da expiração (D56).
    pub not_before: Option<i64>,
}

impl Patch {
    /// Aplica o patch a uma nota (sem gravar).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema`/`InvalidInput` para valor ou transição inválidos.
    pub fn apply(&self, note: &mut Note) -> Result<()> {
        if let Some(note_type) = self.note_type {
            note.frontmatter
                .set("type", Value::Str(note_type.as_str().to_string()))?;
        }
        if let Some(statement) = &self.statement {
            note.frontmatter
                .set("statement", Value::Str(statement.clone()))?;
        }
        if let Some(body) = &self.body {
            note.body.clone_from(body);
        }
        if let Some(confidence) = self.confidence {
            note.frontmatter
                .set("confidence", Value::Float(confidence))?;
        }
        if let Some(tags) = &self.tags {
            set_list(&mut note.frontmatter, "tags", tags)?;
        }
        if let Some(class) = self.classification {
            note.frontmatter
                .set("classification", Value::Str(class.as_str().to_string()))?;
        }
        if let Some(status) = self.status {
            validate_transition(note.frontmatter.status()?, status)?;
            note.frontmatter
                .set("status", Value::Str(status.as_str().to_string()))?;
        }
        if let Some(scope) = self.scope {
            validate_scope(note.frontmatter.note_type()?, Some(scope))?;
            note.frontmatter
                .set("scope", Value::Str(scope.as_str().to_string()))?;
        }
        if let Some(anchors) = &self.anchors {
            set_list(&mut note.frontmatter, "anchors", anchors)?;
        }
        if let Some(source) = &self.source {
            note.frontmatter.set("source", Value::Str(source.clone()))?;
        }
        if let Some(expires) = self.expires_at {
            note.frontmatter.set(
                "expires_at",
                Value::Str(Timestamp::from_millis(expires).to_rfc3339()),
            )?;
        }
        if let Some(not_before) = self.not_before {
            note.frontmatter.set(
                "not_before",
                Value::Str(Timestamp::from_millis(not_before).to_rfc3339()),
            )?;
        }
        Ok(())
    }
}

/// Resultado de [`update`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateOutcome {
    /// Editou no lugar.
    Revised {
        /// Id (inalterado).
        id: String,
        /// Nova revisão.
        revision: u32,
    },
    /// Criou nova versão e marcou a antiga.
    Superseded {
        /// Id antigo.
        old_id: String,
        /// Id novo.
        new_id: String,
        /// Revisão da nota antiga.
        revision: u32,
    },
}

impl UpdateOutcome {
    /// Id resultante (novo, no caso de supersede).
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Revised { id, .. } => id,
            Self::Superseded { new_id, .. } => new_id,
        }
    }

    /// Revisão da nota editada.
    #[must_use]
    pub fn revision(&self) -> u32 {
        match self {
            Self::Revised { revision, .. } | Self::Superseded { revision, .. } => *revision,
        }
    }
}

/// Aplica um patch versionado; muda `type`/`statement` → supersede (D01).
///
/// # Errors
/// Propaga erros de leitura, validação e escrita; retorna `ErrorKind::Schema` para patch inválido.
pub fn update(ctx: &WriteContext<'_>, id: &str, patch: &Patch) -> Result<UpdateOutcome> {
    let mut note = ctx.store().read(id)?;
    patch.apply(&mut note)?;
    let note_type = note.frontmatter.note_type()?;
    let statement = note.frontmatter.statement()?.to_string();
    let new_id = id::note_id(note_type, &statement);
    if new_id == id {
        let revision = note.revision().saturating_add(1);
        note.set_revision(revision)?;
        note.refresh_body_hash()?;
        note.frontmatter.validate()?;
        ctx.store().write(&note)?;
        let record = event("update", id, ctx.now_ms(), WriteAction::Updated, None);
        ctx.events().append(&record)?;
        return Ok(UpdateOutcome::Revised {
            id: id.to_string(),
            revision,
        });
    }
    supersede(ctx, id, note, new_id, patch)
}

fn supersede(
    ctx: &WriteContext<'_>,
    old_id: &str,
    patched: Note,
    new_id: String,
    patch: &Patch,
) -> Result<UpdateOutcome> {
    let mut new_note = patched;
    new_note.frontmatter.set("id", Value::Str(new_id.clone()))?;
    new_note.frontmatter.set(
        "created_at",
        Value::Str(Timestamp::from_millis(ctx.now_ms()).to_rfc3339()),
    )?;
    new_note.frontmatter.remove("revision");
    new_note.frontmatter.remove("superseded_by");
    new_note.frontmatter.remove("replaces");
    if patch.status.is_none() {
        new_note.frontmatter.remove("status");
    }
    graph::link(&mut new_note.frontmatter, EdgeKind::Replaces, old_id)?;
    new_note.refresh_body_hash()?;
    new_note.frontmatter.validate()?;

    let mut old_note = ctx.store().read(old_id)?;
    old_note
        .frontmatter
        .set("superseded_by", Value::Str(new_id.clone()))?;
    old_note.frontmatter.set(
        "status",
        Value::Str(Status::Superseded.as_str().to_string()),
    )?;
    let revision = old_note.revision().saturating_add(1);
    old_note.set_revision(revision)?;
    old_note.refresh_body_hash()?;
    old_note.frontmatter.validate()?;

    ctx.store().write(&new_note)?;
    ctx.store().write(&old_note)?;
    let record = event(
        "supersede",
        old_id,
        ctx.now_ms(),
        WriteAction::Updated,
        None,
    )
    .with_data("new_id", Value::Str(new_id.clone()));
    ctx.events().append(&record)?;
    Ok(UpdateOutcome::Superseded {
        old_id: old_id.to_string(),
        new_id,
        revision,
    })
}

/// Caminha a cadeia de supersessão, do mais antigo ao mais novo.
///
/// # Errors
/// Propaga erros de I/O; ids ausentes no meio da cadeia apenas encerram a caminhada.
pub fn history(store: &Store<'_>, id: &str) -> Result<Vec<Note>> {
    let mut root = id.to_string();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    while seen.insert(root.clone()) {
        let Ok(note) = store.read(&root) else {
            break;
        };
        match note.frontmatter.string_list("replaces")?.first() {
            Some(previous) => root = (*previous).to_string(),
            None => break,
        }
    }
    let mut chain = Vec::new();
    let mut current = Some(root);
    let mut walked: BTreeSet<String> = BTreeSet::new();
    while let Some(cursor) = current {
        if !walked.insert(cursor.clone()) {
            break;
        }
        let Ok(note) = store.read(&cursor) else {
            break;
        };
        current = note
            .frontmatter
            .get("superseded_by")
            .and_then(Value::as_str)
            .map(str::to_string);
        chain.push(note);
    }
    Ok(chain)
}

fn validate_scope(note_type: NoteType, scope: Option<Scope>) -> Result<()> {
    let scoped = matches!(note_type, NoteType::Task | NoteType::Container);
    if scope.is_some() == scoped {
        Ok(())
    } else {
        Err(Error::schema(
            "scope só vale para `type` task/container (D93)",
        ))
    }
}
