//! Criação de tarefas (`kd task new`) e leitura de corpo (E12-T01).

use std::io::Read;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::schema::Scope;
use knudge_core::task::{TaskSpec, submit};
use serde_json::json;

use crate::cli::TaskNewArgs;
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

/// `kd task new`.
///
/// # Errors
/// Propaga erros de hierarquia/validação e I/O.
pub(super) fn new_task(session: &Session, args: &TaskNewArgs) -> Result<Output> {
    let ctx = session.write_context()?;
    let scope: Scope = args.scope.parse()?;
    let mut spec = TaskSpec::new(scope, args.statement.join(" "));
    spec.body = read_body(args.body.as_deref())?;
    spec.parent.clone_from(&args.parent);
    spec.depends_on.clone_from(&args.depends_on);
    spec.checks.clone_from(&args.checks);
    spec.anchors.clone_from(&args.anchors);
    spec.source.clone_from(&args.source);
    spec.expires_at = parse::timestamp_opt(args.expires_at.as_ref())?;
    spec.not_before = parse::timestamp_opt(args.not_before.as_ref())?;
    let id = submit(&ctx, &spec)?.id;
    let data = json!({ "id": id, "scope": scope.as_str(), "parent": args.parent });
    Ok(Output::new(id, data))
}

pub(super) fn read_body(value: Option<&str>) -> Result<String> {
    match value {
        Some("-") => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|error| Error::io("stdin", error))?;
            Ok(buffer)
        }
        Some(text) => Ok(text.to_string()),
        None => Ok(String::new()),
    }
}
