//! Programas externos: Épico-raiz ancorado a `plan/*.md` (D119).

use crate::Result;
use crate::graph::Graph;
use crate::schema::Scope;
use crate::store::Note;
use crate::task::{TaskSpec, program_of, roots_for_path, submit, subtree};

use super::{MemFs, context};

fn read_all(fs: &MemFs) -> Result<Vec<Note>> {
    let ctx = context(fs)?;
    let mut notes = Vec::new();
    for id in ctx.store().list_ids()? {
        notes.push(ctx.store().read(&id)?);
    }
    Ok(notes)
}

#[test]
fn roots_for_path_returns_all_anchored_epics_in_id_order() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let mut first = TaskSpec::new(Scope::Epic, "programa a");
    first.anchors = vec!["plan/foo.md".to_string()];
    let first = submit(&ctx, &first)?.id;
    let mut second = TaskSpec::new(Scope::Epic, "programa b");
    second.anchors = vec!["plan/foo.md".to_string()];
    let second = submit(&ctx, &second)?.id;

    let notes = read_all(&fs)?;
    let mut expected = vec![first, second];
    expected.sort();
    assert_eq!(roots_for_path(&notes, "plan/foo.md")?, expected);
    assert!(roots_for_path(&notes, "plan/outro.md")?.is_empty());
    Ok(())
}

#[test]
fn roots_for_path_ignores_children() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let root = submit(&ctx, &TaskSpec::new(Scope::Epic, "raiz"))?.id;
    let mut child = TaskSpec::new(Scope::Issue, "story");
    child.parent = Some(root);
    child.anchors = vec!["plan/foo.md".to_string()];
    let _child = submit(&ctx, &child)?;

    let notes = read_all(&fs)?;
    assert!(roots_for_path(&notes, "plan/foo.md")?.is_empty());
    Ok(())
}

#[test]
fn subtree_is_deterministic_preorder() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let root = submit(&ctx, &TaskSpec::new(Scope::Epic, "raiz"))?.id;
    let mut issue = TaskSpec::new(Scope::Issue, "story");
    issue.parent = Some(root.clone());
    let story = submit(&ctx, &issue)?.id;
    let mut task = TaskSpec::new(Scope::Task, "task");
    task.parent = Some(story.clone());
    let task_id = submit(&ctx, &task)?.id;

    let graph = Graph::build(ctx.store())?;
    let nodes = subtree(&graph, &root);
    let ids: Vec<&str> = nodes.iter().map(|node| node.id.as_str()).collect();
    assert_eq!(ids, vec![root.as_str(), story.as_str(), task_id.as_str()]);
    assert_eq!(
        nodes.iter().map(|node| node.depth).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    Ok(())
}

#[test]
fn program_of_resolves_by_anchor() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let mut with_anchor = TaskSpec::new(Scope::Epic, "com âncora");
    with_anchor.anchors = vec!["plan/foo.md".to_string()];
    let with_anchor = submit(&ctx, &with_anchor)?.id;
    let plain = submit(&ctx, &TaskSpec::new(Scope::Epic, "sem âncora"))?.id;

    let anchor_note = ctx.store().read(&with_anchor)?;
    let plain_note = ctx.store().read(&plain)?;
    assert_eq!(
        program_of(&anchor_note, "plan/*.md")?,
        Some("plan/foo.md".to_string())
    );
    assert_eq!(program_of(&plain_note, "plan/*.md")?, None);
    Ok(())
}
