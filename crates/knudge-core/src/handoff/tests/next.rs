//! Linhas dinâmicas do manifest: `next:` e `fresh:` (D106).

use crate::Result;
use crate::handoff::manifest_at;
use crate::handoff::next::{MAX_NEXT, next_tasks};
use crate::lifecycle::Freshness;
use crate::schema::{Scope, Status, Value};

use super::{built, task};

#[test]
fn next_orders_by_impact() -> Result<()> {
    let a0 = task("a0", Scope::Task, None)?;
    let a0_id = a0.id()?.to_string();
    let a1 = task("a1", Scope::Task, Some(&a0_id))?;
    let a1_id = a1.id()?.to_string();
    let a2 = task("a2", Scope::Task, Some(&a1_id))?;
    let b0 = task("b0", Scope::Task, None)?;
    let b0_id = b0.id()?.to_string();
    let notes = [a0, a1, a2, b0];
    let (index, graph) = built(&notes)?;

    let next = next_tasks(&index, &graph, MAX_NEXT);
    assert_eq!(
        next.first().map(|task| task.id.as_str()),
        Some(a0_id.as_str())
    );
    assert_eq!(
        next.get(1).map(|task| task.id.as_str()),
        Some(b0_id.as_str())
    );
    Ok(())
}

#[test]
fn closed_ready_tasks_are_skipped() -> Result<()> {
    let mut closed = task("fechada", Scope::Task, None)?;
    closed
        .frontmatter
        .set("status", Value::Str(Status::Closed.as_str().to_string()))?;
    let open = task("aberta", Scope::Task, None)?;
    let open_id = open.id()?.to_string();
    let notes = [closed, open];
    let (index, graph) = built(&notes)?;

    let next = next_tasks(&index, &graph, MAX_NEXT);
    assert_eq!(next.len(), 1);
    assert_eq!(
        next.first().map(|task| task.id.as_str()),
        Some(open_id.as_str())
    );
    Ok(())
}

#[test]
fn manifest_appends_next_and_fresh() -> Result<()> {
    let a0 = task("a0", Scope::Task, None)?;
    let a0_id = a0.id()?.to_string();
    let notes = [a0];
    let (index, graph) = built(&notes)?;
    let fresh = Freshness {
        stale: 1,
        expiring: 2,
        pending: 3,
    };

    let (text, dropped) = manifest_at(&index, &graph, &[], &fresh, 4000);
    assert!(text.contains(&format!("next: {a0_id}|a0")));
    assert!(text.ends_with("fresh: stale=1 expiring=2 pending=3"));
    assert_eq!(dropped, 0);
    Ok(())
}

#[test]
fn small_budget_drops_extra_next() -> Result<()> {
    let x = task("x", Scope::Task, None)?;
    let y = task("y", Scope::Task, None)?;
    let notes = [x, y];
    let (index, graph) = built(&notes)?;

    let (text, dropped) = manifest_at(&index, &graph, &[], &Freshness::default(), 100);
    assert!(text.contains("next: "));
    assert_eq!(dropped, 1);
    Ok(())
}
