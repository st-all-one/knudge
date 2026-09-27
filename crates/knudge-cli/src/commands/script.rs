//! Wrapper fino para scripts acionáveis (E18/D184).
//!
//! O binário **não reimplementa** a lógica de ações essencialmente scriptadas (`drain service`,
//! `self upgrade`): resolve o script (embutido/local por padrão; remoto com **checksum SHA-256**
//! obrigatório), executa com o shell do SO, faz **stream do stderr** (logs verbosos, R20) e
//! captura o **stdout** (dados, para o envelope). Falha do script vira erro (exit não-zero).
//!
//! A resolução é **local por padrão** — nada de rede no caminho quente (herança de E15).

use std::fmt::Write as _;
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

/// Comando do shell do SO para executar `script`.
#[allow(
    clippy::unnecessary_wraps,
    reason = "no Windows o mesmo caminho devolve Err (E18/T02)"
)]
fn shell_command(script: &Path) -> Result<Command> {
    #[cfg(unix)]
    {
        let mut command = Command::new("bash");
        command.arg(script);
        Ok(command)
    }
    #[cfg(not(unix))]
    {
        let _ignored = script;
        // E18/T02 generaliza (PowerShell/`.ps1`); por ora, só bash é suportado.
        Err(Error::invalid_input(
            "scripts acionáveis exigem bash; no Windows use o caminho manual (E18/T02)",
        ))
    }
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
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_known_vector() {
        // Vetor conhecido: SHA-256 de "abc".
        let digest = sha256_hex(b"abc");
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn source_kind_and_reference() {
        let embedded = Source::Embedded {
            name: "knudge-idle.sh",
            body: "",
        };
        assert_eq!(embedded.kind(), "embedded");
        assert_eq!(embedded.reference(), "scripts/knudge-idle.sh");
        let local = Source::Local(PathBuf::from("/tmp/x.sh"));
        assert_eq!(local.kind(), "local");
        let remote = Source::Remote {
            url: "https://example.invalid/x.sh".to_string(),
            sha256: None,
        };
        assert_eq!(remote.kind(), "download");
        assert_eq!(remote.reference(), "https://example.invalid/x.sh");
    }
}
