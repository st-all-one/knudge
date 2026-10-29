//! `AGENTS.md` com marcadores idempotentes e version marker (D60, E04-T04).

use std::path::Path;

use crate::Result;
use crate::ports::Fs;

use super::block::{read_text, upsert};
use super::project::KNUDGE_DIR;

/// Início do bloco gerenciado.
pub const MARKER_BEGIN: &str = "<!-- knudge:start -->";
/// Fim do bloco gerenciado.
pub const MARKER_END: &str = "<!-- knudge:end -->";
/// Prefixo do version marker.
pub const VERSION_PREFIX: &str = "<!-- knudge:version:";
/// Versão atual do protocolo embutido no `AGENTS.md`.
pub const VERSION: u32 = 4;
/// Nome do arquivo.
pub const FILE: &str = "AGENTS.md";

/// Bloco de protocolo que o `kd init`/`onboard` escreve no projeto-alvo (layout default).
#[must_use]
pub fn protocol_block() -> String {
    protocol_block_with_layout(KNUDGE_DIR)
}

/// Bloco de protocolo para um diretório de conhecimento alternativo.
#[must_use]
pub fn protocol_block_with_layout(layout: &str) -> String {
    format!(
        "{MARKER_BEGIN}\n\
         {VERSION_PREFIX} {VERSION} -->\n\
         # knudge — project memory\n\
         \n\
         `notas/` is the source of truth; index/graph/embeddings are **derived** — never edit `notas/` by hand.\n\
         \n\
         Loop: `kd prime` (protocol; `--long` = full) → `kd ask \"<q>\"` before acting → `kd write` each\n\
         learning → `kd rewind` for handoff.\n\
         - `kd ask` dedups: score <0.75 create · 0.75–0.92 merge · ≥0.92 reject.\n\
         - One assertion per note; anchor code with `--anchor PATH` (glob `src/**`).\n\
         - Work lives in `kd task`: `epic ⊃ {{issue ⊃ task | task}}`; `--scope epic|issue|task`,\n\
           `--kind task|error|question|risk|decision`; `kd task list --ready`; close with `--outcome`.\n\
         - `kd write --type task` is rejected; `ask` excludes work unless `--with-task`.\n\
         - Health: `kd doctor` (`--fix`); `kd maintenance learn|compact|prune` propose only.\n\
         - Embeddings are derived: `kd drain --status` / `--digest`.\n\
         - Agent memory shards: `kd ai [memory|session-log] \"<q>\"` (read-only NDJSON,\n\
           `.knudge/ai/<area>/<agent_id>.jsonl`); `--sort time`, `--agent-id/--agent-role`; filter\n\
           `ask` with `--agent/--activity/--agent-id/--agent-role`.\n\
         - stdout = data, stderr = logs; `--json` = {{success, command, data?, error?, warnings?}}.\n\
         - If `kd` is missing on the host: `curl -fsSL https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash` (installs `~/.local/bin`), then `kd prime`.\n\
         Config: `{layout}/config.toml`.\n\
         {MARKER_END}\n"
    )
}

/// Extrai a versão do protocolo presente no texto, se houver.
#[must_use]
pub fn version_in(text: &str) -> Option<u32> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(VERSION_PREFIX) {
            let digits: String = rest
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            if let Ok(value) = digits.parse::<u32>() {
                return Some(value);
            }
        }
    }
    None
}

/// Garante o bloco de protocolo atual (layout default). Devolve `true` se o arquivo mudou.
///
/// # Errors
/// Retorna `ErrorKind::Io` em falha de escrita.
pub fn apply(fs: &dyn Fs, root: &Path) -> Result<bool> {
    apply_with_layout(fs, root, KNUDGE_DIR)
}

/// Garante o bloco de protocolo para `layout`. Devolve `true` se o arquivo mudou.
///
/// # Errors
/// Retorna `ErrorKind::Io` em falha de escrita.
pub fn apply_with_layout(fs: &dyn Fs, root: &Path, layout: &str) -> Result<bool> {
    let path = root.join(FILE);
    let original = read_text(fs, &path)?;
    let block = protocol_block_with_layout(layout);
    let updated = upsert(&original, MARKER_BEGIN, MARKER_END, Some(&block));
    if updated == original {
        return Ok(false);
    }
    fs.write_atomic(&path, updated.as_bytes())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_has_current_version() {
        let block = protocol_block();
        assert_eq!(version_in(&block), Some(VERSION));
        assert!(block.contains(MARKER_BEGIN));
        assert!(block.contains(MARKER_END));
    }
}
