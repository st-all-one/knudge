//! Testes do fechamento por evidência (E09-T02).

use crate::Result;
use crate::health::{CheckOutcome, CheckResult, Severity, close_task, infer_outcome};
use crate::ports::fakes::MemFs;
use crate::schema::{NoteType, Status, Value};
use crate::task::OutcomeStatus;

use super::{note, seeded, task};

fn check(name: &str, result: CheckResult, severity: Severity) -> CheckOutcome {
    CheckOutcome {
        name: name.to_string(),
        result,
        severity,
        duration_ms: Some(10),
        output: None,
    }
}

#[test]
fn infer_outcome_respects_severity() {
    assert_eq!(infer_outcome(&[]), OutcomeStatus::Abandoned);
    assert_eq!(
        infer_outcome(&[check("a", CheckResult::Pass, Severity::Error)]),
        OutcomeStatus::Success
    );
    assert_eq!(
        infer_outcome(&[
            check("a", CheckResult::Pass, Severity::Error),
            check("b", CheckResult::Fail, Severity::Warn),
        ]),
        OutcomeStatus::Partial
    );
    assert_eq!(
        infer_outcome(&[
            check("a", CheckResult::Pass, Severity::Error),
            check("b", CheckResult::Fail, Severity::Warn),
            check("c", CheckResult::Fail, Severity::Error),
        ]),
        OutcomeStatus::Failure
    );
}

#[test]
fn closing_without_evidence_is_rejected() -> Result<()> {
    let fs = MemFs::new();
    let task = task("fechar", &[], &[])?;
    let task_id = task.id()?.to_string();
    let ctx = seeded(&fs, &[task])?;
    assert!(close_task(&ctx, &task_id, &[], None).is_err());
    Ok(())
}

#[test]
fn close_task_records_evidence_and_outcome() -> Result<()> {
    let fs = MemFs::new();
    let task = task("fechar com evidência", &["lint"], &[])?;
    let task_id = task.id()?.to_string();
    let ctx = seeded(&fs, &[task])?;

    let results = vec![
        check("lint", CheckResult::Pass, Severity::Error),
        check("test", CheckResult::Fail, Severity::Warn),
    ];
    let outcome = close_task(&ctx, &task_id, &results, Some("cli"))?;
    assert_eq!(outcome.status, OutcomeStatus::Partial);

    let stored = ctx.store().read(&task_id)?;
    assert_eq!(stored.frontmatter.status()?, Status::Closed);
    let outcomes = stored
        .frontmatter
        .get("outcomes")
        .and_then(Value::as_list)
        .map(<[_]>::len);
    assert_eq!(outcomes, Some(1));
    let evidence = stored
        .frontmatter
        .get("evidence")
        .and_then(Value::as_map)
        .is_some_and(|map| map.contains_key("lint") && map.contains_key("test"));
    assert!(evidence);
    Ok(())
}

#[test]
fn confirmation_is_derived_not_stored() -> Result<()> {
    let fs = MemFs::new();
    let task = task("confirmação derivada", &[], &[])?;
    let task_id = task.id()?.to_string();
    let ctx = seeded(&fs, &[task])?;

    let results = vec![check("lint", CheckResult::Pass, Severity::Error)];
    let _closed = close_task(&ctx, &task_id, &results, None)?;
    let stored = ctx.store().read(&task_id)?;
    // Nenhuma chave `confirmation`/`confidence` é gravada (derivadas — D87/D142).
    assert!(stored.frontmatter.get("confirmation").is_none());
    assert!(stored.frontmatter.get("confidence").is_none());
    Ok(())
}

#[test]
fn non_task_cannot_close() -> Result<()> {
    let fs = MemFs::new();
    let fact = note(NoteType::Fact, "não é tarefa", "")?;
    let fact_id = fact.id()?.to_string();
    let ctx = seeded(&fs, &[fact])?;
    let results = vec![check("lint", CheckResult::Pass, Severity::Error)];
    assert!(close_task(&ctx, &fact_id, &results, None).is_err());
    Ok(())
}
