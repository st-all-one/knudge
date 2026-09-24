//! Criação de tarefas (`kd task new`) e leitura de corpo (E12-T01).

use std::io::Read;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::schema::Scope;
use knudge_core::task::{TaskSpec, submit};
use serde_json::json;

use crate::cli::TaskNewArgs;
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
    if let Some(kind) = &args.kind {
        spec.kind = Some(kind.parse()?);
    }
    spec.body = read_body(args.body.as_deref())?;
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
