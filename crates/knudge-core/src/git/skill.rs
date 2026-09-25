//! Skill do `kd` para o projeto-alvo (D162): `.agents/skill/kd/SKILL.md`.
//!
//! Escrita **idempotente e governada**: só cria se ausente ou se o version marker for nosso e
//! estiver desatualizado. Um arquivo editado pelo usuário (sem o marker) nunca é sobrescrito.

use std::path::Path;

use crate::Result;
use crate::ports::Fs;

/// Diretório relativo da skill.
pub const DIR: &str = ".agents/skill/kd";
/// Arquivo da skill.
pub const FILE: &str = "SKILL.md";
/// Versão atual do conteúdo.
pub const VERSION: u32 = 1;
/// Prefixo do version marker gerenciado.
pub const MARKER: &str = "<!-- knudge:skill:version:";

/// Conteúdo da skill (frontmatter + guia orientado a ação).
#[must_use]
pub fn content() -> String {
    format!("{SKILL_BODY}\n{MARKER} {VERSION} -->\n")
}

/// Escreve/atualiza a skill. Devolve `true` se o arquivo mudou.
///
/// # Errors
/// Retorna `ErrorKind::Io` em falha de leitura/escrita.
pub fn apply(fs: &dyn Fs, root: &Path) -> Result<bool> {
    let path = root.join(DIR).join(FILE);
    if fs.exists(&path) {
        let bytes = fs.read(&path)?;
        let text = String::from_utf8_lossy(&bytes);
        match version_in(&text) {
            // Arquivo do usuário (sem marker): nunca sobrescreve.
            None => return Ok(false),
            // Nossa versão atual: nada a fazer.
            Some(version) if version >= VERSION => return Ok(false),
            Some(_) => {}
        }
    }
    fs.create_dir_all(&root.join(DIR))?;
    fs.write_atomic(&path, content().as_bytes())?;
    Ok(true)
}

/// Extrai a versão do marker, se presente.
#[must_use]
pub fn version_in(text: &str) -> Option<u32> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(MARKER) {
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

/// Corpo estático da skill (sem o version marker).
const SKILL_BODY: &str = r#"---
name: knudge
description: Memória durável por projeto via `kd` (knudge). Use ao buscar, gravar, planejar tarefas, retomar contexto e manter a base. Dispare em: knudge, kd, memória, nota, ask, write, task, épico, rewind, handoff.
---

# knudge — uso real

`kd` é a memória do projeto. A **nota Markdown é a verdade**; índice/embeddings/grafo são
**derivados**. `notas/` não se edita à mão.

## Ciclo

```
kd ask → kd write → kd task → kd sync
```

## Regras de ouro

1. **Busque antes de gravar.** `kd ask "<rascunho>"` evita duplicata (dedup: <0.75 cria,
   0.75–0.92 faz merge, ≥0.92 rejeita).
2. **Uma afirmação por nota.** O `statement` é curto, autocontido e vira o `id` — não empilhe
   afirmações nem dependa de contexto externo.
3. **Corpo = o "porquê" que não cabe no statement.** Use quando o statement sozinho não permite
   agir (`decision`/`error`/`risk`). Template (2–4 linhas):
   ```
   Por quê: <motivo/decisão>
   Evidência: <comando, saída, erro, link>
   Consequência: <o que muda na prática>
   ```
   `kd ask` mostra o corpo: 1º hit completo, 2–5 truncado; leia com `--id`/`--full-content`.
4. **Ancore o código.** Toda nota/tarefa sobre um arquivo leva `--anchor PATH` (glob `src/**`
   casa subárvores). `kd ask --anchor PATH` acha pelo arquivo.
5. **Evidência separada do corpo.** Conclusão de tarefa usa `--outcome`; fato/decisão ganha
   âncora. Corpo não substitui evidência.

## Comandos essenciais

```
kd prime                                   # protocolo completo (1x por sessão)
kd ask "<query>" [--limit N] [--brief]     # recuperar; --brief só id|statement
kd ask --id <ID>                           # corpo completo de uma nota
kd ask --anchor src/x.rs                   # por arquivo, sem query
kd write --summary "<afirmação>" [<corpo>|-] --type <fact|decision|error|risk|question>
        [--tag T] [--anchor PATH]
kd write --update <ID> --summary "<...>"   # muda statement → novo id + supersede
kd write --outcome <success|partial|failure|abandoned> --id <ID> [--note TXT]
kd task new --summary "<...>" --scope <epic|issue|task> [--parent ID] [--anchor PATH]
kd task close --id <ID> [--outcome S]      # só declara com evidência
kd rewind [--budget N]                     # retomar contexto entre sessões
kd maintenance doctor [--audit]            # saúde da base
kd sync [--message M]                      # commit de notas/ + eventos/
```

## Anti-padrões

- Gravar sem `kd ask` antes (duplicata) ou `--type task` no `write` (use `kd task`).
- Statement composto/ambíguo, sem âncora quando fala de código.
- Guardar segredo no corpo (o log redige, mas a nota não deve conter segredo).
- Editar `notas/` à mão — use `kd write --update`.

## Saída

stdout = dados, stderr = logs. `--json` = `{success, command, data?, error{code,message,retryable}, warnings?}`.
Busca vazia → `[no_results]` (exit 0).
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::fakes::MemFs;
    use std::path::Path;

    #[test]
    fn content_has_marker_and_version() {
        let text = content();
        assert_eq!(version_in(&text), Some(VERSION));
        assert!(text.contains("knudge"));
    }

    #[test]
    fn apply_writes_once_then_is_idempotent() -> Result<()> {
        let fs = MemFs::new();
        let root = Path::new("/p");
        assert!(apply(&fs, root)?);
        assert!(!apply(&fs, root)?);
        assert!(fs.exists(&root.join(DIR).join(FILE)));
        Ok(())
    }

    #[test]
    fn apply_never_overwrites_user_owned_file() -> Result<()> {
        let fs = MemFs::new();
        let root = Path::new("/p");
        fs.create_dir_all(&root.join(DIR))?;
        fs.write_atomic(&root.join(DIR).join(FILE), b"skill do usuario")?;
        assert!(!apply(&fs, root)?);
        let bytes = fs.read(&root.join(DIR).join(FILE))?;
        let text = String::from_utf8_lossy(&bytes);
        assert_eq!(text, "skill do usuario");
        Ok(())
    }
}
