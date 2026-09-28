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
pub const VERSION: u32 = 2;
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
description: Project-scoped durable memory via `kd` (knudge) — search, record, plan tasks, resume context, maintain the base. Trigger on: knudge, kd, memory, memória, note, nota, ask, write, task, tarefa, epic, épico, rewind, handoff.
---

# knudge — real usage

`kd` is the project memory. The **Markdown note is the truth**; index/embeddings/graph are
**derived**. Never edit `notas/` by hand.

## Cycle

```
kd ask → kd write → kd task → kd sync
```

## Golden rules

1. **Search before writing.** `kd ask "<draft>"` avoids duplicates (dedup: `<0.75` create,
   `0.75–0.92` merge, `≥0.92` reject).
2. **One assertion per note.** Short, self-contained `statement`; it derives the `id`. Never
   invent an id — copy it from output. Reclassifying never rewrites the id.
3. **Body = the "why" the statement can't carry.** Use when the statement alone can't drive
   action (`decision`/`error`/`risk`):
   ```
   Why: <reason>
   Evidence: <command, output, error, link>
   Consequence: <what changes>
   ```
   `kd ask` shows the body: 1st hit full, 2–5 truncated; read with `--id`/`--full-content`.
4. **Anchor code.** Every note/task about a file gets `--anchor PATH` (glob `src/**` matches
   subtrees). `kd ask --anchor PATH` finds by file.
5. **Evidence ≠ body.** Task completion uses `--outcome`; facts/decisions get anchors. Body never
   replaces evidence.
6. **Nothing changes without acceptance.** `doctor`/`learn`/`compact`/`prune` only propose; apply
   via `write`/`write --link`/`forget`.
7. **stdout = data, stderr = logs.** In `--json`, stdout is only the envelope. EPIPE → exit 0.

## Essential commands

```
kd prime                                   # full protocol (once per session)
kd ask "<q>" [--brief] [--limit N]         # search; --brief is id|statement only
kd ask --id <ID>                           # full body of a note
kd ask --anchor src/x.rs                   # by file, no query
kd ask --around <ID> [--via <EDGE>]        # expand the graph
kd ask --suggest [--relation R]            # semantic suggestions
kd write --summary "<s>" [<body>|-] --type <fact|decision|error|risk|question>
        [--tag T] [--anchor P]
kd write --update <ID> --summary "<s>"     # versioned update (new id + supersede)
kd write --link <FROM:EDGE:TO>             # explicit edge (12 kinds)
kd write --outcome <success|partial|failure|abandoned> --id <ID> [--note TXT]
kd task new --summary "<s>" --scope <epic|issue|task> [--parent ID] [--anchor P]
kd task list --ready [--sort impact]       # or --blocked [--explain]
kd task close --id <ID> --outcome S        # only declares with evidence
kd rewind [--budget N] [--files P...]      # resume context between sessions
kd doctor [--fix] [--explain]              # base health
kd map [--axis A] [--communities]          # knowledge map
kd maintenance <learn|compact|prune>       # proposes only (read-only)
kd sync [--message M]                      # commit notas/ + eventos/
```

## Closed sets (D212)

Fixed-value flags reject an invalid value with the full list + closest match (`did you mean…`).
An absent flag validates nothing.

- `--type`: fact decision question task def error snippet link meta risk
- `--class`: foundational tactical observational
- `--status`: active in_progress blocked closed superseded forgotten
- `--scope`: epic issue task · `--kind`: task error question risk decision
- `--outcome`: success partial failure abandoned
- edges (`--link`/`--edge`/`--via`): references depends_on contradicts supports extends replaces
  rejects results_in same_as broader narrower related
- `--relation`: duplicate contradiction link · `--axis`: anchor type classification scope

## Lists (D210)

List flags accept repetition or comma: `--tag a --tag b` ≡ `--tag a,b`. The space form
(`--id a b`) does not exist. Free text is never split; use `--params '<json>'` for arrays.

## Anchors × edges

- **Anchor** (`--anchor PATH`): ties the note to a file/glob — the only link to code.
- **Edge** (`--link`): ties the note to another note, with a type (12, closed).

## Output

stdout = data, stderr = logs. `--json` =
`{success, command, data?, error{code,message,retryable}, warnings?}`. Empty search →
`[no_results]` (exit 0). Exit codes: 2 invalid · 3 not found · 4 conflict · 5 io · 6 timeout ·
7 config · 8 schema · 70 internal.

## Anti-patterns

- Writing without `kd ask` first (duplicate) or `--type task` in `write` (use `kd task`).
- Composite/ambiguous statement, or missing anchor when it talks about code.
- Storing secrets in the body (logs redact, notes don't).
- Editing `notas/` by hand — use `kd write --update`.
- Expecting `learn`/`compact`/`prune` to change the corpus — they only propose.
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
