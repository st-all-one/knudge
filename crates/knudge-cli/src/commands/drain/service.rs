//! `kd drain service` — gerencia o worker de auto-drain ocioso (E11-T03/D131/D132/D186).
//!
//! As ações (`--install`, `--subscribe`, `--unsubscribe`, `--status`, `--uninstall`) são
//! delegadas ao `knudge-idle.sh` **embutido no binário** (sem download por padrão; `--script` ou
//! `--url`/`KNUDGE_SCRIPT_URL` sobrescrevem). Ação explícita = aceite (D180): mutações executam
//! direto, sem prompt. O wrapper (`commands::script`) faz **stream do stderr** e mantém o stdout
//! como dados (R20).

use std::path::{Path, PathBuf};

use knudge_core::Result;
use knudge_core::ports::Env;
use serde_json::json;

use crate::cli::WatchServiceArgs;
use crate::commands::script;
use crate::output::Output;
use crate::session::Session;

/// Corpo do worker, embutido no binário — sem supply-chain de rede por padrão.
const SCRIPT_BODY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/knudge-idle.sh"
));

/// Ação pedida (default: `--status`, read-only).
#[derive(Clone, Copy)]
enum Action {
    Install,
    Subscribe,
    Unsubscribe,
    Status,
    Uninstall,
}

impl Action {
    /// Resolve a ação a partir das flags (o clap garante no máximo uma).
    fn of(args: &WatchServiceArgs) -> Self {
        if args.install {
            Self::Install
        } else if args.subscribe {
            Self::Subscribe
        } else if args.unsubscribe {
            Self::Unsubscribe
        } else if args.uninstall {
            Self::Uninstall
        } else {
            Self::Status
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Subscribe => "subscribe",
            Self::Unsubscribe => "unsubscribe",
            Self::Status => "status",
            Self::Uninstall => "uninstall",
        }
    }

    /// Ações que operam sobre o projeto atual.
    fn scoped(self) -> bool {
        matches!(self, Self::Install | Self::Subscribe | Self::Unsubscribe)
    }
}

/// Executa `kd drain service`.
///
/// # Errors
/// Propaga falha de materialização/download/execução do worker e de I/O.
pub fn run(session: &Session, args: &WatchServiceArgs) -> Result<Output> {
    let action = Action::of(args);
    let root = session.project_root().to_path_buf();
    let source = source_of(session, args);
    let worker = worker_args(action, &root, args);

    if args.dry_run {
        return Ok(plan(action, &source, &worker));
    }
    let path = script::resolve(session, &source)?;
    let run = script::run(&path, &worker)?;
    let stdout = run.stdout.trim();
    let text = if stdout.is_empty() {
        format!("drain service {}: ok", action.name())
    } else {
        stdout.to_string()
    };
    Ok(Output::new(
        format!("{text}\npróximos: {}", next_step(action)),
        json!({
            "action": action.name(),
            "done": true,
            "script": path.display().to_string(),
        }),
    ))
}

/// Argumentos repassados ao `knudge-idle.sh`.
fn worker_args(action: Action, root: &Path, args: &WatchServiceArgs) -> Vec<String> {
    let mut out = vec![action.name().to_string()];
    if action.scoped() {
        out.push("--project".to_string());
        out.push(root.display().to_string());
    }
    if matches!(action, Action::Install) {
        out.push("--every".to_string());
        out.push(args.every.clone());
        out.push("--port".to_string());
        out.push(args.port.to_string());
        if let Some(model) = &args.model {
            out.push("--model".to_string());
            out.push(model.clone());
        }
        if args.no_deps {
            out.push("--no-deps".to_string());
        }
    }
    out
}

/// Prioridade: `--script` > `--url`/`KNUDGE_SCRIPT_URL` > embutido.
fn source_of(session: &Session, args: &WatchServiceArgs) -> script::Source {
    if let Some(path) = &args.script {
        return script::Source::Local(PathBuf::from(path));
    }
    if let Some(url) = args
        .url
        .clone()
        .or_else(|| session.env().var("KNUDGE_SCRIPT_URL"))
    {
        return script::Source::Remote {
            url,
            sha256: args.sha256.clone(),
        };
    }
    script::Source::Embedded {
        name: "knudge-idle.sh",
        body: SCRIPT_BODY,
    }
}

/// Plano do `--dry-run` (não materializa nem executa).
fn plan(action: Action, source: &script::Source, worker: &[String]) -> Output {
    let executable = match source {
        script::Source::Embedded { .. } => "<embutido>".to_string(),
        script::Source::Local(path) => path.display().to_string(),
        script::Source::Remote { .. } => "<baixado>".to_string(),
    };
    let command = format!("bash {executable} {}", worker.join(" "));
    Output::new(
        format!("dry-run: {command}\npróximos: {}", next_step(action)),
        json!({
            "dry_run": true,
            "action": action.name(),
            "source": source.kind(),
            "reference": source.reference(),
            "command": command,
        }),
    )
}

/// Passo seguinte sugerido por ação (D165).
fn next_step(action: Action) -> &'static str {
    match action {
        Action::Install => "verifique com `kd drain service --status`",
        Action::Subscribe | Action::Unsubscribe => "confira com `--status`",
        Action::Uninstall => "nada a fazer",
        Action::Status => "`--install`/`--subscribe` são opcionais",
    }
}
