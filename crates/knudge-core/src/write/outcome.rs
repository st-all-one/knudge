//! `outcomes[]` — evidência de execução para **qualquer** nota (D103).
//!
//! Generaliza o antigo `outcome` de tarefa (que exigia `type=task|container`): a confiança
//! derivada (D87) e o boost BM25 (E06-T02) passam a valer para conhecimento confirmado por
//! trabalho. Nada é armazenado em chave nova — `outcomes` já é canônica (D48).

use std::str::FromStr;

use crate::schema::Value;
use crate::time::Timestamp;
use crate::{Error, Result};

use super::{WriteAction, WriteContext, event};

/// Resultado de uma execução.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeStatus {
    /// Sucesso.
    Success,
    /// Parcial.
    Partial,
    /// Falha.
    Failure,
    /// Abandonado.
    Abandoned,
}

impl OutcomeStatus {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Partial => "partial",
            Self::Failure => "failure",
            Self::Abandoned => "abandoned",
        }
    }
}

impl FromStr for OutcomeStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "success" => Ok(Self::Success),
            "partial" => Ok(Self::Partial),
            "failure" => Ok(Self::Failure),
            "abandoned" => Ok(Self::Abandoned),
            other => Err(Error::invalid_input(format!(
                "outcome desconhecido: {other:?}"
            ))),
        }
    }
}

/// Anexa um resultado a `outcomes` (D48) e devolve a nova revisão.
///
/// Vale para **qualquer** nota (D103): `outcomes` é a evidência de que a nota foi exercitada.
///
/// # Errors
/// Retorna `ErrorKind::NotFound` se a nota não existe e propaga erros de I/O.
pub fn outcome(
    ctx: &WriteContext<'_>,
    id: &str,
    status: OutcomeStatus,
    note: Option<&str>,
) -> Result<u32> {
    let mut existing = ctx.store().read(id)?;
    let mut items = match existing.frontmatter.get("outcomes") {
        Some(Value::List(items)) => items.clone(),
        _ => Vec::new(),
    };
    let mut entry = vec![
        (
            "status".to_string(),
            Value::Str(status.as_str().to_string()),
        ),
        (
            "recorded_at".to_string(),
            Value::Str(Timestamp::from_millis(ctx.now_ms()).to_rfc3339()),
        ),
    ];
    if let Some(note) = note {
        entry.push(("notes".to_string(), Value::Str(note.to_string())));
    }
    items.push(Value::map(entry));
    existing.frontmatter.set("outcomes", Value::List(items))?;
    let revision = existing.revision().saturating_add(1);
    existing.set_revision(revision)?;
    existing.frontmatter.validate()?;
    ctx.store().write(&existing)?;
    let record = event("outcome", id, ctx.now_ms(), WriteAction::Updated, None)
        .with_data("outcome", Value::Str(status.as_str().to_string()));
    ctx.events().append(&record)?;
    Ok(revision)
}
