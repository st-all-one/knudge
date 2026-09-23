//! Escrita em lote a partir de JSONL de rascunhos (K4/D110).

use super::dedup::{DedupDecision, DedupThresholds, propose};
use super::{Draft, WriteAction, WriteContext, WriteOutcome, write};
use crate::jsonl;
use crate::{Error, Result};

/// Modo do lote (K4/D110).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchMode {
    /// Grava os itens.
    Apply,
    /// Só avalia o dedup, sem gravar.
    DryRun,
}

/// Resultado do lote (K4/D110).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BatchOutput {
    /// Resultado por item válido, na ordem de entrada.
    pub items: Vec<WriteOutcome>,
    /// Avisos (linhas inválidas), na forma `linha N: <erro>`.
    pub warnings: Vec<String>,
}

/// Aplica o protocolo de escrita a um lote de rascunhos JSONL (K4/D110).
///
/// Linha malformada/inválida vira `warnings` e o lote continua (R33). Itens válidos preservam
/// a ordem de entrada.
#[must_use]
pub fn batch_jsonl(
    ctx: &WriteContext<'_>,
    source: &str,
    thresholds: &DedupThresholds,
    mode: BatchMode,
) -> BatchOutput {
    let mut output = BatchOutput::default();
    for (index, line) in jsonl::lines(source).enumerate() {
        let number = index.saturating_add(1);
        let draft = match jsonl::decode(line).and_then(|value| Draft::from_value(&value)) {
            Ok(draft) => draft,
            Err(error) => {
                output.warnings.push(format!("linha {number}: {error}"));
                continue;
            }
        };
        let result = match mode {
            BatchMode::Apply => write(ctx, &draft, thresholds),
            BatchMode::DryRun => evaluate(ctx, &draft, thresholds),
        };
        match result {
            Ok(outcome) => output.items.push(outcome),
            Err(error) => output.warnings.push(format!("linha {number}: {error}")),
        }
    }
    output
}

/// Avalia um rascunho sem gravar (dry-run do lote).
fn evaluate(
    ctx: &WriteContext<'_>,
    draft: &Draft,
    thresholds: &DedupThresholds,
) -> Result<WriteOutcome> {
    let note = draft.to_note(ctx.now_ms())?;
    let id = note.id()?.to_string();
    if ctx.store().exists(&id) {
        let current = ctx.store().read(&id)?;
        let same = current.frontmatter.get("body_hash") == note.frontmatter.get("body_hash");
        if !same {
            return Err(Error::conflict(format!(
                "nota {id} já existe com corpo diferente; use update"
            )));
        }
        return Ok(WriteOutcome {
            action: WriteAction::Unchanged,
            id,
            revision: Some(current.revision()),
        });
    }
    let proposal = propose(ctx.index(), draft, thresholds)?;
    Ok(match proposal.decision {
        DedupDecision::Create => WriteOutcome {
            action: WriteAction::Created,
            id,
            revision: None,
        },
        DedupDecision::Merge { candidate, .. } => WriteOutcome {
            action: WriteAction::Merged,
            id: candidate,
            revision: None,
        },
        DedupDecision::Reject { candidate, .. } => WriteOutcome {
            action: WriteAction::Rejected,
            id: candidate,
            revision: None,
        },
    })
}
