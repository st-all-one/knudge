//! Testes do bloco de `AGENTS.md` (E04-T04).

use std::path::Path;

use super::*;
use crate::Result;
use crate::git::agent_md;
use crate::ports::Fs;

#[test]
fn inserts_once_and_is_idempotent() -> Result<()> {
    let fs = MemFs::new();
    let root = Path::new("/work/proj");
    assert!(agent_md::apply(&fs, root)?);
    let path = root.join("AGENTS.md");
    let text = String::from_utf8(fs.read(&path)?).unwrap_or_default();
    assert_eq!(agent_md::version_in(&text), Some(agent_md::VERSION));
    assert!(!agent_md::apply(&fs, root)?, "segunda vez não muda");
    Ok(())
}

#[test]
fn upgrades_old_version_and_preserves_user_content() -> Result<()> {
    let fs = MemFs::new();
    let root = Path::new("/work/proj");
    let path = root.join("AGENTS.md");
    let original = "\
# Meu AGENTS

<!-- knudge:start -->
<!-- knudge:version: 0 -->
conteúdo velho
<!-- knudge:end -->

Fim do usuário
";
    fs.write_atomic(&path, original.as_bytes())?;

    assert!(agent_md::apply(&fs, root)?);
    let text = String::from_utf8(fs.read(&path)?).unwrap_or_default();
    assert_eq!(agent_md::version_in(&text), Some(agent_md::VERSION));
    assert!(
        text.contains("# Meu AGENTS"),
        "preserva conteúdo do usuário"
    );
    assert!(text.contains("Fim do usuário"));
    assert!(!text.contains("conteúdo velho"), "substitui o bloco antigo");
    assert!(!agent_md::apply(&fs, root)?, "idempotente após upgrade");
    Ok(())
}
