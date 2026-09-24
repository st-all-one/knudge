//! Parser/runner do lote de tarefas (D141).

use crate::jsonl;
use crate::schema::{EdgeKind, NoteType, Scope, Value};
use crate::task::{TaskBatchMode, TaskOp, batch_jsonl};
use crate::{Error, Result};

use super::{MemFs, context};

#[test]
fn task_op_parses_fields_and_edges() -> Result<()> {
    let value = jsonl::decode(
        r#"{"key":"a","statement":"S","scope":"task","kind":"error","tags":["x"],"depends_on":["b"],"references":["c"]}"#,
    )?;
    let op = TaskOp::from_value(&value)?;
    assert_eq!(op.key.as_deref(), Some("a"));
    assert_eq!(op.spec.scope, Scope::Task);
    assert_eq!(op.spec.kind.map(NoteType::as_str), Some("error"));
    assert_eq!(op.spec.tags, ["x".to_string()]);
    assert!(
        op.edges
            .iter()
            .any(|(kind, to)| *kind == EdgeKind::DependsOn && to == "b")
    );
    assert!(
        op.edges
            .iter()
            .any(|(kind, to)| *kind == EdgeKind::References && to == "c")
    );
    Ok(())
}

#[test]
fn task_op_rejects_unknown_key() {
    let value = Value::map(vec![
        ("statement".to_string(), Value::Str("S".to_string())),
        ("scope".to_string(), Value::Str("task".to_string())),
        ("x".to_string(), Value::Int(1)),
    ]);
    assert!(TaskOp::from_value(&value).is_err());
}

#[test]
fn batch_creates_with_parent_by_key() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let source = concat!(
        r#"{"key":"e","statement":"Épico","scope":"epic"}"#,
        "\n",
        r#"{"key":"t","statement":"Tarefa","scope":"task","parent":"e","depends_on":["e"]}"#,
        "\n",
    );
    let out = batch_jsonl(&ctx, source, TaskBatchMode::Apply, 10)?;
    assert_eq!(out.items.len(), 2);
    let epic = out.keys.get("e").ok_or_else(|| Error::not_found("sem e"))?;
    let task = out
        .items
        .iter()
        .find(|item| item.key.as_deref() == Some("t"))
        .ok_or_else(|| Error::not_found("sem t"))?;
    assert_eq!(task.parent.as_deref(), Some(epic.as_str()));
    assert!(
        task.edges
            .iter()
            .any(|(kind, to)| *kind == EdgeKind::DependsOn && to == epic)
    );
    Ok(())
}

#[test]
fn batch_update_applies_anchors() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let created = batch_jsonl(
        &ctx,
        r#"{"key":"t","statement":"Tarefa","scope":"task"}"#,
        TaskBatchMode::Apply,
        10,
    )?;
    let id = created
        .keys
        .get("t")
        .cloned()
        .ok_or_else(|| Error::not_found("sem t"))?;

    // Antes as âncoras do update eram parseadas e ignoradas em silêncio.
    let line = format!(r#"{{"id":"{id}","anchors":["src/a.rs","src/b.rs"]}}"#);
    let updated = batch_jsonl(&ctx, &format!("{line}\n"), TaskBatchMode::Apply, 10)?;
    assert!(updated.items.iter().any(|item| item.action == "updated"));
    let note = ctx.store().read(&id)?;
    assert_eq!(
        note.frontmatter.string_list("anchors")?,
        vec!["src/a.rs", "src/b.rs"]
    );
    Ok(())
}
