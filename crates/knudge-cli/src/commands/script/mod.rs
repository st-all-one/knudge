//! Wrapper fino para scripts acionáveis (E18/D184).
//!
//! O binário **não reimplementa** a lógica de ações essencialmente scriptadas (`drain service`,
//! `self upgrade`): resolve o script (embutido/local por padrão; remoto com **checksum SHA-256**
//! obrigatório), executa com o shell do SO, faz **stream do stderr** (logs verbosos, R20) e
//! captura o **stdout** (dados, para o envelope). Falha do script vira erro (exit não-zero).
//!
//! A resolução é **local por padrão** — nada de rede no caminho quente (herança de E15).

use std::ffi::OsStr;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::ports::{Env, Fs};
use sha2::{Digest, Sha256};

use crate::session::Session;

/// De onde vem o script acionável.
pub enum Source {
    /// Corpo embutido no binário (default; sem rede).
    Embedded {
        /// Nome do arquivo no staging de cache.
        name: &'static str,
        /// Conteúdo do script.
        body: &'static str,
    },
    /// Script local (`--script`).
    Local(PathBuf),
    /// Remoto (`--url`/`KNUDGE_SCRIPT_URL`); exige checksum SHA-256 (D184).
    Remote {
        /// URL HTTPS.
        url: String,
        /// SHA-256 esperado (hex); ausente ⇒ recusa o download.
        sha256: Option<String>,
    },
}

impl Source {
    /// Rótulo de máquina da origem (`embedded`/`local`/`download`).
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Embedded { .. } => "embedded",
            Self::Local(_) => "local",
            Self::Remote { .. } => "download",
        }
    }

    /// Referência legível da origem (para `--dry-run` e mensagens).
    #[must_use]
    pub fn reference(&self) -> String {
        match self {
            Self::Embedded { name, .. } => format!("scripts/{name}"),
            Self::Local(path) => path.display().to_string(),
            Self::Remote { url, .. } => url.clone(),
        }
    }
}

/// Resultado da execução de um script.
pub struct ScriptRun {
    /// stdout capturado (dados, R20).
    pub stdout: String,
}

/// Resolve o script para um caminho executável (materializa o embutido; baixa+verifica o remoto).
///
/// # Errors
/// `InvalidInput` quando o remoto não traz checksum; `Io` em falha de I/O/`curl`.
pub fn resolve(session: &Session, source: &Source) -> Result<PathBuf> {
    match source {
        Source::Embedded { name, body } => {
            let dir = cache_dir(session).join("staging");
            session.fs().create_dir_all(&dir)?;
            let dest = dir.join(name);
            session.fs().write_atomic(&dest, body.as_bytes())?;
            Ok(dest)
        }
        Source::Local(path) => Ok(path.clone()),
        Source::Remote { url, sha256 } => download(session, url, sha256.as_deref()),
    }
}

/// Executa `script` com o shell do SO: stream do stderr, stdout capturado, exit propagado.
///
/// # Errors
/// `InvalidInput` em SO sem shell; `Io` quando o processo não inicia ou termina não-zero.
pub fn run(script: &Path, args: &[String]) -> Result<ScriptRun> {
    let mut command = shell_command(script)?;
    let child = command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| Error::io(script, error))?;
    let output = child
        .wait_with_output()
        .map_err(|error| Error::io(script, error))?;
    if !output.status.success() {
        // O código exato do processo é do contrato do binário (R30–R35); aqui propagamos a falha.
        return Err(Error::io(
            script,
            std::io::Error::other(format!("script terminou com {}", output.status)),
        ));
    }
    Ok(ScriptRun {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}

/// Executa o **corpo** de um script embutido **sem materializar** (stdin do shell) — para ações
/// read-only como `--status` (P6/E17-T07). Evita escrever no staging de cache.
///
/// # Errors
/// `InvalidInput` em SO sem shell compatível; `Io` quando o processo não inicia ou falha.
pub fn run_body(body: &str, args: &[String]) -> Result<ScriptRun> {
    let mut command = shell_command_stdin()?;
    let mut child = command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| Error::io(EMBEDDED_REF, error))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(body.as_bytes())
            .map_err(|error| Error::io(EMBEDDED_REF, error))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| Error::io(EMBEDDED_REF, error))?;
    if !output.status.success() {
        return Err(Error::io(
            EMBEDDED_REF,
            std::io::Error::other(format!("script terminou com {}", output.status)),
        ));
    }
    Ok(ScriptRun {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}

/// Referência de diagnóstico do script embutido (não é um caminho real).
const EMBEDDED_REF: &str = "knudge-idle.sh (embedded)";

/// Comando de shell que lê o script do **stdin** (`bash -s`); Windows recusa (D185).
fn shell_command_stdin() -> Result<Command> {
    if std::env::consts::OS == "windows" {
        return Err(Error::invalid_input(
            "no Windows, o worker embutido é shell Unix; use `--script` com um `.ps1` (D185)",
        ));
    }
    let mut command = Command::new("bash");
    command.arg("-s");
    Ok(command)
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

/// Baixa o script para o staging e **verifica o SHA-256** antes de devolvê-lo.
fn download(session: &Session, url: &str, expected: Option<&str>) -> Result<PathBuf> {
    let Some(expected) = expected.map(str::trim).filter(|hex| !hex.is_empty()) else {
        return Err(Error::invalid_input(
            "download remoto de script exige `--sha256 <hex>` (supply-chain, D184)",
        ));
    };
    let dir = cache_dir(session).join("staging");
    session.fs().create_dir_all(&dir)?;
    let dest = dir.join("knudge-script.remote");
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
    let actual = sha256_hex(&session.fs().read(&dest)?);
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(Error::invalid_input(format!(
            "checksum do script não confere: esperado {expected}, obtido {actual}"
        )));
    }
    Ok(dest)
}

/// Comando do shell do SO para executar `script` (D185).
fn shell_command(script: &Path) -> Result<Command> {
    let (program, args) = shell_spec(script, std::env::consts::OS)?;
    let mut command = Command::new(program);
    command.args(args);
    Ok(command)
}

/// Shell do SO para executar `script` (D185). Pura: recebe o SO para ser testável.
///
/// - Unix (Linux/macOS/BSD): `bash <script>`.
/// - Windows: `powershell -NoProfile -File <script>` quando o script é `.ps1`; caso
///   contrário, recusa e aponta o caminho manual (o embutido é Unix).
fn shell_spec(script: &Path, os: &str) -> Result<(String, Vec<String>)> {
    let path = script.display().to_string();
    if os == "windows" {
        if is_powershell(script) {
            return Ok((
                "powershell".to_string(),
                vec!["-NoProfile".to_string(), "-File".to_string(), path],
            ));
        }
        return Err(Error::invalid_input(format!(
            "no Windows, scripts acionáveis exigem `.ps1` (PowerShell): `{path}` é shell Unix — \
             veja o caminho manual em wiki/usage/18_embeddings.md (D185)"
        )));
    }
    Ok(("bash".to_string(), vec![path]))
}

/// `true` quando o script tem extensão `.ps1` (sem diferenciar maiúsculas).
fn is_powershell(script: &Path) -> bool {
    script
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ps1"))
}

/// Prefixo de shell do SO atual, para exibição (`--dry-run`; D185).
#[must_use]
pub fn shell_prefix() -> &'static str {
    if std::env::consts::OS == "windows" {
        "powershell -NoProfile -File"
    } else {
        "bash"
    }
}

/// Linha de comando do `--dry-run` (ou o caminho manual quando o SO não suporta o script).
#[must_use]
pub fn plan_command(reference: &str, args: &[String]) -> String {
    let Ok((program, mut parts)) = shell_spec(Path::new(reference), std::env::consts::OS) else {
        return format!("{reference} (manual: sem shell compatível neste SO — D185)");
    };
    parts.extend(args.iter().cloned());
    format!("{program} {}", parts.join(" "))
}

/// SHA-256 em hexadecimal minúsculo (verificação de supply-chain).
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len().saturating_mul(2));
    for byte in digest {
        let _ignored = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests;
