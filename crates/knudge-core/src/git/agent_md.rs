//! `AGENTS.md` com marcadores idempotentes e version marker (D60, E04-T04).

use std::path::Path;

use crate::Result;
use crate::ports::Fs;

use super::block::{read_text, upsert};

/// Início do bloco gerenciado.
pub const MARKER_BEGIN: &str = "<!-- knudge:start -->";
/// Fim do bloco gerenciado.
pub const MARKER_END: &str = "<!-- knudge:end -->";
/// Prefixo do version marker.
pub const VERSION_PREFIX: &str = "<!-- knudge:version:";
/// Versão atual do protocolo embutido no `AGENTS.md`.
pub const VERSION: u32 = 1;
/// Nome do arquivo.
pub const FILE: &str = "AGENTS.md";

/// Bloco de protocolo que o `kd init`/`onboard` escreve no projeto-alvo.
#[must_use]
pub fn protocol_block() -> String {
    format!(
        "{MARKER_BEGIN}\n\
         {VERSION_PREFIX} {VERSION} -->\n\
         # knudge — memória do projeto\n\
         \n\
         Este projeto usa **knudge** (`kd`). `notas/` é a fonte da verdade; não edite à mão.\n\
         \n\
         Antes de implementar:\n\
         1. `kd prime` — protocolo completo (uma vez por sessão).\n\
         2. `kd ask \"<pergunta>\"` — consulte antes de criar.\n\
         3. `kd write \"<afirmação>\"` — registre cada aprendizado (uma afirmação por nota).\n\
         4. `kd rewind` — situe a próxima sessão ao encerrar.\n\
         \n\
         Config: `.knudge/config.toml`. Diagnóstico: `kd doctor`.\n\
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

/// Garante o bloco de protocolo atual. Devolve `true` se o arquivo mudou.
///
/// # Errors
/// Retorna `ErrorKind::Io` em falha de escrita.
pub fn apply(fs: &dyn Fs, root: &Path) -> Result<bool> {
    let path = root.join(FILE);
    let original = read_text(fs, &path)?;
    let block = protocol_block();
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
