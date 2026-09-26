//! `kd maintenance watch-service` — gerencia o worker de auto-drain ocioso (E11-T03/D131/D132).
//!
//! As ações (`--install`, `--subscribe`, `--unsubscribe`, `--status`, `--uninstall`) são
//! delegadas ao `knudge-idle.sh` **embutido no binário** (sem download por padrão; `--script` ou
//! `--url`/`KNUDGE_SCRIPT_URL` sobrescrevem). Ações que mutam o host perguntam antes (`--yes`
//! pula; stdin não-TTY ⇒ cancela). O prompt vai para **stderr** (stdout segue só dados, R20).

use std::path::{Path, PathBuf};
use std::process::Command;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::ports::{Env, Fs};
use serde_json::json;

use crate::cli::WatchServiceArgs;
use crate::output::{Output, emit_stderr};
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

    /// Ações que mudam o host pedem confirmação.
    fn asks(self) -> bool {
        matches!(self, Self::Install | Self::Subscribe | Self::Uninstall)
    }

    /// Ações que operam sobre o projeto atual.
    fn scoped(self) -> bool {
        matches!(self, Self::Install | Self::Subscribe | Self::Unsubscribe)
    }

    fn question(self) -> &'static str {
        match self {
            Self::Install => {
                "Instalar o worker de auto-drain (systemd/launchd), o servidor de embeddings persistente e cadastrar este projeto? Baixa llama.cpp + modelo GGUF (~100 MB) se ausentes (--no-deps pula)."
            }
            Self::Subscribe => "Cadastrar este projeto no worker de auto-drain?",
            Self::Uninstall => {
                "Remover o worker de auto-drain e o servidor de embeddings (unidades/agentes + config + binário)?"
            }
            Self::Unsubscribe | Self::Status => "Continuar?",
        }
    }
}

/// De onde vem o worker.
enum Source {
    /// Embutido no binário (default).
    Embedded,
    /// Script local (`--script`).
    Local(PathBuf),
    /// Baixado da URL (`--url`/`KNUDGE_SCRIPT_URL`).
    Download(String),
}

/// Executa `kd maintenance watch-service`.
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
    if action.asks() && !args.yes && !confirm(action.question())? {
        return Ok(Output::new(
            format!("cancelado: {}", action.name()),
            json!({ "action": action.name(), "done": false, "reason": "declined" }),
        ));
    }
    let script = match &source {
        Source::Embedded => materialize(session)?,
        Source::Local(path) => path.clone(),
        Source::Download(url) => download(session, url.as_str())?,
    };
    run_worker(&script, action, &worker)
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
fn source_of(session: &Session, args: &WatchServiceArgs) -> Source {
    if let Some(path) = &args.script {
        return Source::Local(PathBuf::from(path));
    }
    if let Some(url) = args
        .url
        .clone()
        .or_else(|| session.env().var("KNUDGE_SCRIPT_URL"))
    {
        return Source::Download(url);
    }
    Source::Embedded
}

/// Plano do `--dry-run` (não materializa nem executa).
fn plan(action: Action, source: &Source, worker: &[String]) -> Output {
    let (kind, reference) = match source {
        Source::Embedded => ("embedded", "scripts/knudge-idle.sh".to_string()),
        Source::Local(path) => ("local", path.display().to_string()),
        Source::Download(url) => ("download", url.clone()),
    };
    let script = match source {
        Source::Embedded => "<embutido>".to_string(),
        Source::Local(path) => path.display().to_string(),
        Source::Download(_) => "<baixado>".to_string(),
    };
    let command = format!("bash {script} {}", worker.join(" "));
    Output::new(
        format!("dry-run: {command}\npróximos: {}", next_step(action)),
        json!({
            "dry_run": true,
            "action": action.name(),
            "source": kind,
            "reference": reference,
            "command": command,
        }),
    )
}

/// Passo seguinte sugerido por ação (D165).
fn next_step(action: Action) -> &'static str {
    match action {
        Action::Install => "verifique com `kd maintenance watch-service --status`",
        Action::Subscribe | Action::Unsubscribe => "confira com `--status`",
        Action::Uninstall => "nada a fazer",
        Action::Status => "`--install`/`--subscribe` são opcionais",
    }
}

/// Pergunta sim/não em stderr (stdin não-TTY ou vazio ⇒ não).
fn confirm(question: &str) -> Result<bool> {
    emit_stderr(format!("{question} [s/N] ").as_bytes());
    let mut line = String::new();
    let read = std::io::stdin()
        .read_line(&mut line)
        .map_err(|error| Error::io("stdin", error))?;
    if read == 0 {
        return Ok(false);
    }
    Ok(matches!(
        line.trim().to_ascii_lowercase().as_str(),
        "s" | "sim" | "y" | "yes"
    ))
}

/// Raiz de cache (`${XDG_CACHE_HOME:-~/.cache}/knudge`).
fn cache_dir(session: &Session) -> PathBuf {
    let env = session.env();
    env.var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env.var("HOME")
                .map(|home| PathBuf::from(home).join(".cache"))
        })
        .unwrap_or_else(std::env::temp_dir)
        .join("knudge")
}

/// Escreve o script embutido no staging de cache e devolve o caminho.
fn materialize(session: &Session) -> Result<PathBuf> {
    let dir = cache_dir(session).join("staging");
    session.fs().create_dir_all(&dir)?;
    let dest = dir.join("knudge-idle.sh");
    session.fs().write_atomic(&dest, SCRIPT_BODY.as_bytes())?;
    Ok(dest)
}

/// Baixa o script para o staging de cache via `curl` (HTTPS).
fn download(session: &Session, url: &str) -> Result<PathBuf> {
    let dir = cache_dir(session).join("staging");
    session.fs().create_dir_all(&dir)?;
    let dest = dir.join("knudge-idle.sh");
    let status = Command::new("curl")
        .args(["-fsSL", "--proto", "=https", "--tlsv1.2", "-o"])
        .arg(&dest)
        .arg(url)
        .status()
        .map_err(|error| Error::io("curl", error))?;
    if !status.success() {
        return Err(Error::io(
            url,
            std::io::Error::other(format!("curl falhou (status {status})")),
        ));
    }
    Ok(dest)
}

/// Executa `bash <script> <ação...>` e resume o resultado.
fn run_worker(script: &Path, action: Action, worker: &[String]) -> Result<Output> {
    let output = Command::new("bash")
        .arg(script)
        .args(worker)
        .output()
        .map_err(|error| Error::io("bash", error))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::io(
            script,
            std::io::Error::other(format!(
                "watch-service {} falhou ({}): {}",
                action.name(),
                output.status,
                stderr.trim()
            )),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = if stdout.trim().is_empty() {
        format!("watch-service {}: ok", action.name())
    } else {
        stdout.trim().to_string()
    };
    Ok(Output::new(
        format!("{text}\npróximos: {}", next_step(action)),
        json!({
            "action": action.name(),
            "done": true,
            "script": script.display().to_string(),
        }),
    ))
}
