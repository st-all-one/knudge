# Git, worktree e onboarding

Como o knudge se ancora no repositório: resolve o **worktree principal**, exclui os derivados via
`.git/info/exclude`, escreve blocos idempotentes no `AGENTS.md`, instala a skill do projeto e
comita o conhecimento com `sync`.

- Código: `crates/knudge-core/src/git/`
- Decisões: D29–D34, D60, D91, D162

## Resolução do projeto (D29/D91)

- `.knudge/` resolve no **worktree principal** (`git rev-parse --git-common-dir`); um submódulo
  **não** conta.
- O projeto é identificado por **nome lógico**: worktrees do mesmo repo **compartilham** o mesmo
  `.knudge/`.
- `Project` (`git/project.rs`) expõe `KNUDGE_DIR`, `is_valid_name`, `logical_name`.
- **Segredos só no global** (D91): o projeto nunca carrega credenciais.

## Exclusão via `info/exclude` (D30/D34)

- A exclusão é **absoluta** e vive em `.git/info/exclude` — **nunca** `.gitignore`.
- `persist_in_project=true` (default): versiona `notas/`+`eventos/`, mas exclui derivados
  (`.idx/`, `cache/`, `*.lock`) via `DERIVED_PATTERNS`/`KNUDGE_PATTERN`.
- `persist_in_project=false`: exclui o `.knudge/` inteiro (local-only).
- `git/exclude.rs` mantém a exclusão **idempotente** (não duplica linhas).

## `AGENTS.md` e blocos idempotentes (D60)

- `git/block.rs::upsert` insere/substitui blocos delimitados por marcadores
  (`<!-- knudge:start -->`/`<!-- knudge:end -->`) sem duplicar.
- O **protocolo** do knudge (D60) e o bloco de **regras governadas** (`knudge:rules`, D157) são
  blocos irmãos: `init`/`onboard` reescrevem só o protocolo, preservando as regras.
- `git/agent_md.rs::protocol_block` renderiza o protocolo com *version marker*.

## Skill do projeto (D162)

`kd init`/`onboard` cria/atualiza `.agents/skill/kd/SKILL.md` (`git/skill.rs`), uma skill
otimizada para uso real (idempotente, com version marker) e referenciada no `AGENTS.md`. O corpo é
em **inglês** (idioma das skills), token-optimized; o marker governa a versão (`VERSION = 2`).

## `sync` (D32)

- Comita `notas/` + `eventos/` no worktree principal, com **guard de worktree**.
- Mensagem gerada a partir do evento.
- Executa `git -C <raiz>` (D97) — nunca um shell (R12); o adaptador `StdGit` invoca o binário
  `git` diretamente.

## `.gitattributes` (D31)

`git/attributes.rs` define regras explícitas para todos os arquivos do `.knudge/`:
`merge=union` para `events.jsonl` (e cache vetorial — D148), garantindo que eventos concorrentes
não se percam no merge.

## `onboard` (D60/D97)

`git/onboard.rs::onboard` cria/atualiza `.knudge/` de forma **idempotente**: clona o config
global, aplica exclusões, cria `notas/`/`eventos/` e escreve o protocolo. Sem config global,
**degrada para defaults** (D97).

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Worktree/nome lógico | `git/project.rs` |
| Exclusão | `git/exclude.rs` |
| Atributos | `git/attributes.rs` |
| Blocos/marcadores | `git/block.rs`, `git/agent_md.rs` |
| Skill | `git/skill.rs` |
| `sync` | `git/sync.rs` |
| `onboard` | `git/onboard.rs` |
| Política de persistência | `git/persistence.rs` |

## Testes

`git/tests/` (agents, attributes, exclude, onboard, project, sync) — todos sobre a porta `FakeGit`,
sem tocar o repositório real.
