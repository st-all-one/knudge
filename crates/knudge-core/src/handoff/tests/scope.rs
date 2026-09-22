//! Auto-scope e auto-flip (E08-T03).

use crate::Result;
use crate::handoff::scope::{detect_scope, should_flip};
use crate::schema::NoteType;

use super::{built, container, member, note};

#[test]
fn flip_thresholds() {
    assert!(!should_flip(100, 5));
    assert!(should_flip(101, 0));
    assert!(should_flip(0, 6));
}

#[test]
fn detect_scope_picks_matching_container() -> Result<()> {
    let plan = container("plano")?;
    let plan_id = plan.id()?.to_string();
    let other = container("outro")?;
    let other_id = other.id()?.to_string();
    let first = member("membro", &["src/**"], &plan_id)?;
    let second = member("outro membro", &["docs/**"], &other_id)?;
    let loose = note(NoteType::Fact, "solta")?;
    let notes = [plan, other, first, second, loose];
    let (index, graph) = built(&notes)?;
    let scope = detect_scope(&["src/main.rs".to_string()], &graph, &index);
    assert_eq!(scope.as_deref(), Some(plan_id.as_str()));
    Ok(())
}

#[test]
fn no_match_yields_none() -> Result<()> {
    let plan = container("plano")?;
    let notes = [plan];
    let (index, graph) = built(&notes)?;
    assert!(detect_scope(&["nope.rs".to_string()], &graph, &index).is_none());
    Ok(())
}
