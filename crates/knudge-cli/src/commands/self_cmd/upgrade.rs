//! `kd self upgrade` — wrapper fino do instalador oficial (E18/D187).
//!
//! O binário **não reimplementa** o install: resolve o `kd-upgrade.sh` (embutido por padrão;
//! `--script` local ou `--url` remoto com checksum) e o evoca. O script chama o `install.sh`
//! oficial (release + checksum). stdout = dados; stderr = logs (R20).

use std::path::PathBuf;

use knudge_core::Result;
use knudge_core::ports::Env;
use serde_json::json;

use crate::cli::UpgradeArgs;
use crate::commands::script;
use crate::output::Output;
use crate::session::Session;

/// Corpo do instalador de upgrade, embutido no binário (fonte da verdade; D184).
const SCRIPT_BODY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/kd-upgrade.sh"
));

/// Executa `kd self upgrade`.
///
/// # Errors
/// Propaga falha de materialização/download/execução do script e de I/O.
pub(super) fn run(session: &Session, args: &UpgradeArgs) -> Result<Output> {
    let source = source_of(session, args);
    let mut worker = vec!["upgrade".to_string()];
    if let Some(version) = &args.version {
        worker.push("--version".to_string());
        worker.push(version.clone());
    }

    if args.dry_run {
        let mut planned = worker.clone();
        planned.push("--dry-run".to_string());
        let plan = if let script::Source::Embedded { body, .. } = &source {
            script::run_body(body, &planned)
        } else {
            let path = script::resolve(session, &source)?;
            script::run(&path, &planned)
        };
        let detail = plan
            .ok()
            .map(|run| run.stdout.trim().to_string())
            .filter(|detail| !detail.is_empty());
        return Ok(plan_output(&source, &worker, detail.as_deref()));
    }

    let path = script::resolve(session, &source)?;
    let run = script::run(&path, &worker)?;
    let stdout = run.stdout.trim();
    let text = if stdout.is_empty() {
        "self upgrade: ok".to_string()
    } else {
        stdout.to_string()
    };
    Ok(Output::new(
        format!("{text}\npróximos: confira com `kd self version`"),
        json!({
            "done": true,
            "script": path.display().to_string(),
        }),
    ))
}

/// Plano do `--dry-run` (não materializa nem executa; `detail` é o plano do script, se houver).
fn plan_output(source: &script::Source, args: &[String], detail: Option<&str>) -> Output {
    let reference = source.reference();
    let command = script::plan_command(&reference, args);
    let text = match detail {
        Some(detail) => {
            format!("dry-run: {command}\n{detail}\npróximos: rode sem `--dry-run` para atualizar")
        }
        None => format!("dry-run: {command}\npróximos: rode sem `--dry-run` para atualizar"),
    };
    let data = match detail {
        Some(detail) => json!({
            "dry_run": true,
            "source": source.kind(),
            "reference": reference,
            "command": command,
            "plan": detail,
        }),
        None => json!({
            "dry_run": true,
            "source": source.kind(),
            "reference": reference,
            "command": command,
        }),
    };
    Output::new(text, data)
}

/// Prioridade: `--script` > `--url`/`KNUDGE_UPGRADE_URL` > embutido.
fn source_of(session: &Session, args: &UpgradeArgs) -> script::Source {
    if let Some(path) = &args.script {
        return script::Source::Local(PathBuf::from(path));
    }
    if let Some(url) = args
        .url
        .clone()
        .or_else(|| session.env().var("KNUDGE_UPGRADE_URL"))
    {
        return script::Source::Remote {
            url,
            sha256: args.sha256.clone(),
        };
    }
    script::Source::Embedded {
        name: "kd-upgrade.sh",
        body: SCRIPT_BODY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_upgrade_script_is_present() {
        assert!(
            SCRIPT_BODY.contains("install.sh"),
            "sem referência ao install.sh"
        );
        assert!(
            SCRIPT_BODY.contains("kd-upgrade"),
            "sem identificação do script"
        );
    }
}
