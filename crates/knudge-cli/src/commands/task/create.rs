//! Criação de tarefas (`kd task new`) e leitura de corpo (E12-T01).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::schema::Scope;
use knudge_core::task::{TaskBatchMode, TaskSpec, submit};
use serde_json::json;

use crate::cli::TaskNewArgs;
use crate::commands::input;
use crate::output::Output;
use crate::session::Session;

use super::batch::TaskInput;

/// `kd task new`.
///
/// # Errors
/// Propaga erros de hierarquia/validação e I/O.
pub(super) fn new_task(session: &Session, args: &TaskNewArgs) -> Result<Output> {
    let mode = if args.dry_run {
        TaskBatchMode::DryRun
    } else {
        TaskBatchMode::Apply
    };
    if let Some(source) = &args.batch {
        return super::batch::run(session, source, TaskInput::Batch, mode);
    }
    if let Some(params) = &args.params {
        return super::batch::run(session, params, TaskInput::Params, mode);
    }
    let ctx = session.write_context()?;
    let scope: Scope = args
        .scope
        .as_deref()
        .ok_or_else(|| Error::invalid_input("`kd task new` exige `--scope`"))?
        .parse()?;
    let summary = args
        .summary
        .clone()
        .ok_or_else(|| Error::invalid_input("`kd task new` exige `--summary`"))?;
    let mut spec = TaskSpec::new(scope, summary);
    if let Some(kind) = &args.kind {
        spec.kind = Some(kind.parse()?);
    }
    spec.body = input::content(&args.body)?;
    spec.parent.clone_from(&args.parent);
    spec.checks.clone_from(&args.checks);
    spec.anchors.clone_from(&args.anchors);
    spec.tags.clone_from(&args.tag);
    let id = submit(&ctx, &spec)?.id;
    let data = json!({ "id": id, "scope": scope.as_str(), "parent": args.parent });
    let mut output = Output::new(id, data);
    if scope == Scope::Epic && args.anchors.is_empty() {
        output = output.with_warnings(vec![
            "épico sem âncora: ancore com `--anchor plan/<arquivo>.md` (D119/D134)".to_string(),
        ]);
    }
    Ok(output)
}
