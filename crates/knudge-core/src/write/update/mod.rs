//! `update` versionado e supersede caminhável (D01/D21/D48).
//!
//! Se a **chave de conteúdo** (`type` + `statement`) muda, a nota não é sobrescrita: cria-se uma
//! nova (novo `id`) com `replaces` e a antiga vira `superseded` com `superseded_by` (D01). Caso
//! contrário, a edição acontece no lugar e `revision` incrementa.

use std::collections::BTreeSet;

use crate::graph;
use crate::schema::{EdgeKind, NoteType, Scope, Status, Value, id};
use crate::store::{Note, Store};
use crate::time::Timestamp;
use crate::{Error, Result};

use super::{WriteAction, WriteContext, event};

mod patch;

pub use patch::Patch;

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
    let original_type = note.frontmatter.note_type()?;
    let original_statement = note.frontmatter.statement()?.to_string();
    // Id que a nota **teria** se fosse derivada hoje; diferente de `id` = id histórico/legado
    // (`container_*`, D02/D95) ou afirmação fora do normalizador.
    let canonical_id = id::note_id(original_type, &original_statement);
    patch.apply(&mut note)?;
    let note_type = note.frontmatter.note_type()?;
    let statement = note.frontmatter.statement()?.to_string();
    let new_id = id::note_id(note_type, &statement);
    let content_key_changed = note_type != original_type || statement != original_statement;
    // Id não-derivável não renomeia em silêncio numa edição de corpo/tags/âncoras: revisa no
    // lugar. Só supersede quando a **chave de conteúdo** (`type` + `statement`) muda (D01).
    let revises_in_place = new_id == id || (canonical_id != id && !content_key_changed);
    if revises_in_place {
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

pub(super) fn validate_scope(note_type: NoteType, scope: Option<Scope>) -> Result<()> {
    if scope.is_some() && !note_type.is_scoped() {
        return Err(Error::schema(
            "scope só vale para item de trabalho/container (D93/D113)",
        ));
    }
    if note_type.requires_scope() && scope.is_none() {
        return Err(Error::schema(
            "`type` de trabalho/container exige scope (D93/D113)",
        ));
    }
    Ok(())
}
