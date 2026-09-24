//! Criação de tarefas em lote (JSONL) e por objeto (D141).
//!
//! Uma linha (ou o objeto de `--params`) descreve **uma operação**: cria quando não há `id`,
//! atualiza quando há. `parent` e as arestas aceitam `key` local (definida na própria linha)
//! ou um `id` existente. `depends_on` é resolvido ao final, aceitando `key` de qualquer linha.
//! Best-effort (R33): linha inválida vira `warnings[]` e o lote continua.

mod parse;

use std::collections::BTreeMap;

pub use parse::TaskOp;

use crate::jsonl;
use crate::schema::{EdgeKind, NoteType, Scope, Status, Value};
use crate::store::Note;
use crate::task::{membership, submit, validate_parent, validate_transition};
use crate::write::{WriteContext, link};
use crate::{Error, Result};

/// Modo do lote (D141).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskBatchMode {
    /// Grava os itens.
    Apply,
    /// Só avalia, sem gravar.
    DryRun,
}

/// Item do lote já resolvido (para a saída auto-suficiente).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskBatchItem {
    /// `created`/`updated`.
    pub action: &'static str,
    /// `key` local, quando houver.
    pub key: Option<String>,
    /// Id resultante.
    pub id: String,
    /// Escopo.
    pub scope: Scope,
    /// Espécie.
    pub kind: NoteType,
    /// Status final.
    pub status: Status,
    /// Afirmação.
    pub statement: String,
    /// Pai resolvido.
    pub parent: Option<String>,
    /// Arestas resolvidas.
    pub edges: Vec<(EdgeKind, String)>,
}

/// Resultado do lote (D141).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskBatchOutput {
    /// Itens válidos, na ordem de entrada.
    pub items: Vec<TaskBatchItem>,
    /// Avisos (linhas inválidas), na forma `linha N: <erro>`.
    pub warnings: Vec<String>,
    /// Mapa `key` → id (para a IA vincular sem nova consulta).
    pub keys: BTreeMap<String, String>,
}

/// Aplica um lote de operações de tarefa (JSONL) com best-effort (D141/R33).
///
/// # Errors
/// `ErrorKind::InvalidInput` quando o lote excede `max`.
pub fn batch_jsonl(
    ctx: &WriteContext<'_>,
    source: &str,
    mode: TaskBatchMode,
    max: usize,
) -> Result<TaskBatchOutput> {
    let mut parsed: Vec<(usize, TaskOp)> = Vec::new();
    let mut output = TaskBatchOutput::default();
    for (index, line) in jsonl::lines(source).enumerate() {
        let number = index.saturating_add(1);
        match jsonl::decode(line).and_then(|value| TaskOp::from_value(&value)) {
            Ok(op) => parsed.push((number, op)),
            Err(error) => output.warnings.push(format!("linha {number}: {error}")),
        }
    }
    if parsed.len() > max {
        return Err(Error::invalid_input(format!(
            "lote acima do teto `task.batch_max` ({max})"
        )));
    }
    let mut keys: BTreeMap<String, String> = BTreeMap::new();
    let mut ids: Vec<String> = Vec::with_capacity(parsed.len());
    for (number, op) in &parsed {
        match run_op(ctx, op, mode, &mut keys) {
            Ok(item) => {
                ids.push(item.id.clone());
                output.items.push(item);
            }
            Err(error) => {
                ids.push(String::new());
                output.warnings.push(format!("linha {number}: {error}"));
            }
        }
    }
    if mode == TaskBatchMode::Apply {
        for (index, (_, op)) in parsed.iter().enumerate() {
            let Some(id) = ids.get(index).filter(|id| !id.is_empty()) else {
                continue;
            };
            for (kind, to) in &op.edges {
                if *kind != EdgeKind::DependsOn {
                    continue;
                }
                let to = resolve(to, &keys);
                let _ignored = link(ctx, id, *kind, &to)?;
            }
        }
    }
    output.keys = keys;
    Ok(output)
}

/// Cria ou atualiza uma operação e devolve o item resolvido.
fn run_op(
    ctx: &WriteContext<'_>,
    op: &TaskOp,
    mode: TaskBatchMode,
    keys: &mut BTreeMap<String, String>,
) -> Result<TaskBatchItem> {
    let (id, note) = resolve_target(ctx, op, mode, keys)?;
    let edges = apply_edges(ctx, op, mode, keys, &id)?;
    let parent = op.spec.parent.as_ref().map(|parent| resolve(parent, keys));
    Ok(TaskBatchItem {
        action: if op.id.is_some() {
            "updated"
        } else {
            "created"
        },
        key: op.key.clone(),
        id,
        scope: note.frontmatter.scope()?.unwrap_or(op.spec.scope),
        kind: note.frontmatter.note_type()?,
        status: note.frontmatter.status()?,
        statement: note.frontmatter.statement().unwrap_or_default().to_string(),
        parent,
        edges,
    })
}

/// Cria (via `submit`) ou atualiza (via `apply_update`) e devolve o id + a nota final.
fn resolve_target(
    ctx: &WriteContext<'_>,
    op: &TaskOp,
    mode: TaskBatchMode,
    keys: &mut BTreeMap<String, String>,
) -> Result<(String, Note)> {
    if let Some(existing) = &op.id {
        if mode == TaskBatchMode::Apply {
            apply_update(ctx, existing, op, keys)?;
        }
        return Ok((existing.clone(), ctx.store().read(existing)?));
    }
    let mut spec = op.spec.clone();
    if let Some(parent) = &spec.parent {
        spec.parent = Some(resolve(parent, keys));
    }
    let note = spec.to_note(ctx.now_ms())?;
    let id = note.id()?.to_string();
    if mode == TaskBatchMode::Apply {
        let _outcome = submit(ctx, &spec)?;
    }
    if let Some(key) = &op.key {
        keys.insert(key.clone(), id.clone());
    }
    Ok((id, note))
}

/// Cria as arestas explícitas (exceto `depends_on`, adiado) e devolve as resolvidas.
fn apply_edges(
    ctx: &WriteContext<'_>,
    op: &TaskOp,
    mode: TaskBatchMode,
    keys: &BTreeMap<String, String>,
    id: &str,
) -> Result<Vec<(EdgeKind, String)>> {
    let mut edges = Vec::new();
    for (kind, to) in &op.edges {
        let to = resolve(to, keys);
        if *kind != EdgeKind::DependsOn && mode == TaskBatchMode::Apply {
            let _ignored = link(ctx, id, *kind, &to)?;
        }
        edges.push((*kind, to));
    }
    Ok(edges)
}

/// Atualiza uma tarefa existente: `statement`/`status`/`checks`/`body`/`parent`.
fn apply_update(
    ctx: &WriteContext<'_>,
    id: &str,
    op: &TaskOp,
    keys: &BTreeMap<String, String>,
) -> Result<()> {
    let mut note = ctx.store().read(id)?;
    if !op.spec.statement.is_empty() {
        note.frontmatter
            .set("statement", Value::Str(op.spec.statement.clone()))?;
    }
    if let Some(status) = op.spec.status {
        validate_transition(note.frontmatter.status()?, status)?;
        note.frontmatter
            .set("status", Value::Str(status.as_str().to_string()))?;
    }
    if !op.spec.checks.is_empty() {
        let items = op
            .spec
            .checks
            .iter()
            .map(|check| Value::Str(check.clone()))
            .collect();
        note.frontmatter.set("checks", Value::List(items))?;
    }
    if !op.spec.body.is_empty() {
        note.body.clone_from(&op.spec.body);
    }
    if let Some(parent) = &op.spec.parent {
        let parent = resolve(parent, keys);
        reparent(ctx, &mut note, &parent)?;
    }
    let revision = note.revision().saturating_add(1);
    note.set_revision(revision)?;
    note.refresh_body_hash()?;
    note.frontmatter.validate()?;
    ctx.store().write(&note)?;
    Ok(())
}

/// Re-parenta uma nota reusando a regra de hierarquia (D93/D134).
fn reparent(ctx: &WriteContext<'_>, note: &mut Note, parent: &str) -> Result<()> {
    let parent_note = ctx.store().read(parent)?;
    let parent_scope = parent_note
        .frontmatter
        .scope()?
        .ok_or_else(|| Error::schema(format!("pai {parent} não tem `scope`")))?;
    let child_scope = note
        .frontmatter
        .scope()?
        .ok_or_else(|| Error::schema("nota não tem `scope`"))?;
    validate_parent(parent_scope, child_scope)?;
    let blocks = membership::parse(&note.body).and_then(|marker| marker.blocks);
    note.body = membership::set(&note.body, parent, blocks);
    Ok(())
}

/// Resolve `key` → id; sem `key` correspondente, devolve o próprio valor (id).
fn resolve(value: &str, keys: &BTreeMap<String, String>) -> String {
    keys.get(value)
        .cloned()
        .unwrap_or_else(|| value.to_string())
}
