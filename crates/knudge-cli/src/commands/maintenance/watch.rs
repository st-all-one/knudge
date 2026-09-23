//! `kd maintenance watch-service` — instala o worker de auto-drain ocioso (E11-T03/D132).
//!
//! Pergunta ao usuário e, se aprovado, baixa `scripts/knudge-idle.sh` (na tag da versão) e o
//! aciona com `install` — que cria um timer `systemd --user` para drenar a fila quando o `kd`
//! não está em uso. Nunca age sem confirmação (`--yes` pula a pergunta; stdin não-TTY ⇒ cancela).
//! O prompt vai para **stderr** (stdout segue só dados, R20).

use std::path::{Path, PathBuf};
use std::process::Command;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::ports::{Env, Fs};
use serde_json::json;

use crate::cli::WatchServiceArgs;
use crate::output::{Output, emit_stderr};
use crate::session::Session;

/// Repositório oficial (o mesmo do `install.sh`).
const REPO: &str = "st-all-one/knudge";
/// Versão do binário; o script é baixado na tag correspondente.
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Caminho do script no repositório.
const SCRIPT: &str = "scripts/knudge-idle.sh";

/// De onde vem o worker.
enum Source {
    /// Baixado da URL.
    Download(String),
    /// Script local (`--script`).
    Local(PathBuf),
}

/// Executa `kd maintenance watch-service`.
///
/// # Errors
/// Propaga falha de download/execução do worker e de I/O.
pub fn run(session: &Session, args: &WatchServiceArgs) -> Result<Output> {
    let root = session.project_root().to_path_buf();
    let install = install_args(&root, args);
    let source = source_of(session, args);

    if args.dry_run {
        return Ok(plan(&source, &install));
    }
    if !args.yes && !confirm("Instalar o worker de auto-drain (timer systemd) deste projeto?")? {
        return Ok(Output::new(
            "cancelado: worker não instalado",
            json!({ "installed": false, "reason": "declined" }),
        ));
    }

    let script = match &source {
        Source::Local(path) => path.clone(),
        Source::Download(url) => download(session, url.as_str())?,
    };
    run_worker(&script, &install)
}

/// Argumentos repassados ao `knudge-idle.sh install`.
fn install_args(root: &Path, args: &WatchServiceArgs) -> Vec<String> {
    let mut out = vec![
        "install".to_string(),
        "--project".to_string(),
        root.display().to_string(),
        "--every".to_string(),
        args.every.clone(),
        "--port".to_string(),
        args.port.to_string(),
    ];
    if let Some(model) = &args.model {
        out.push("--model".to_string());
        out.push(model.clone());
    }
    out
}

/// Resolve a origem do script (`--script`, `--url`/`KNUDGE_SCRIPT_URL`, ou a tag da versão).
fn source_of(session: &Session, args: &WatchServiceArgs) -> Source {
    if let Some(path) = &args.script {
        return Source::Local(PathBuf::from(path));
    }
    let url = args
        .url
        .clone()
        .or_else(|| session.env().var("KNUDGE_SCRIPT_URL"))
        .unwrap_or_else(default_url);
    Source::Download(url)
}

/// URL default do script, na tag `v<versão>`.
fn default_url() -> String {
    format!("https://raw.githubusercontent.com/{REPO}/v{VERSION}/{SCRIPT}")
}

/// Plano do `--dry-run` (não baixa nem executa).
fn plan(source: &Source, install: &[String]) -> Output {
    let (kind, reference) = match source {
        Source::Download(url) => ("download", url.clone()),
        Source::Local(path) => ("local", path.display().to_string()),
    };
    let script = match source {
        Source::Download(_) => "<baixado>".to_string(),
        Source::Local(path) => path.display().to_string(),
    };
    let command = format!("bash {script} {}", install.join(" "));
    Output::new(
        format!("dry-run: {command}"),
        json!({
            "dry_run": true,
            "source": kind,
            "reference": reference,
            "command": command,
        }),
    )
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

/// Executa `bash <script> <install...>` e resume o resultado.
fn run_worker(script: &Path, install: &[String]) -> Result<Output> {
    let output = Command::new("bash")
        .arg(script)
        .args(install)
        .output()
        .map_err(|error| Error::io("bash", error))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::io(
            script,
            std::io::Error::other(format!(
                "worker não instalado ({}): {}",
                output.status,
                stderr.trim()
            )),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = if stdout.trim().is_empty() {
        "worker instalado".to_string()
    } else {
        stdout.trim().to_string()
    };
    Ok(Output::new(
        text,
        json!({ "installed": true, "script": script.display().to_string() }),
    ))
}
