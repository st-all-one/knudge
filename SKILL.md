---
name: knudge
description: Project-scoped durable memory for LLM agents via the `kd` CLI (knudge) — search before writing, record knowledge, plan/run tasks, resume context, maintain the base. Trigger on: knudge, kd, memory, memória, note, nota, recall, ask, write, task, tarefa, epic, épico, handoff, rewind, TOON, embedding, RRF, MCP.
---

# knudge — agent usage guide

**What:** `kd` gives the agent durable, project-scoped memory. Markdown notes in `notas/` are the
source of truth; the index (BM25 + anchors + optional embeddings + graph) is **derived** and
rebuildable. Single binary, no server, no DB. `knudge-mcp` exposes memory triggers over MCP.

**Load when:** the task involves remembering/recording decisions, facts, errors, risks or
questions; planning or tracking work; resuming context across sessions; or searching accumulated
project knowledge.

**Never edit `notas/` by hand** — use `kd write --update`.

**Deep docs:** CLI surface `plan/implementation/16_cli_surface.md` · acceptance
`plan/implementation/17_matriz_aceitacao.md` · byte contract `wiki/specs/TOON.md` · architecture
`wiki/specs/ARCHITECTURE.md` · decisions `plan/03_decisoes-fechadas.md` · embeddings
`plan/04_embeddings.md` · MCP `plan/implementation/18_mcp_transporte.md` · full CLI guide
`wiki/usage/`.

## Golden rules

1. **Search before writing.** `kd ask "<draft>"` → dedup: `<0.75` create, `0.75–0.92` merge,
   `≥0.92` reject. Never write blind.
2. **One assertion per note.** Short, self-contained `--summary`; it derives the `id`
   (`type + statement`). Reclassifying never rewrites the id. **Never invent an id** — copy it.
3. **Body = the "why" the summary can't carry.** Use when the summary alone can't drive action
   (`decision`/`error`/`risk`): `Why:` / `Evidence:` / `Consequence:` (2–4 lines).
4. **Anchor code.** Any note/task about a file gets `--anchor PATH` (globs: `src/**`).
   `kd ask --anchor PATH` finds by file.
5. **Evidence ≠ body.** Task completion uses `--outcome`; facts/decisions get anchors.
6. **Nothing changes without acceptance.** `doctor`/`learn`/`compact`/`prune` only **propose**;
   apply via `write`/`write --link`/`forget`.
7. **stdout = data, stderr = logs.** In `--json`, stdout is only the envelope. EPIPE → exit 0.

## Cycle

```
kd prime  →  kd ask  →  kd write  →  kd task  →  kd sync
(protocol)   (search)    (record)     (execute)    (commit)
```

`kd` alone == `kd help`. `kd prime` is the static, byte-identical protocol (once per session);
`kd prime --long` appends the TOON grammar + schema.

## Commands

Top-level: `init prime rewind ask write task map maintenance doctor drain config forget sync self`
(+ `help`).

| Goal | Command |
|---|---|
| Protocol | `kd prime [--long]` |
| Search | `kd ask "<q>" [--brief] [--limit N] [--full-content] [--with-task]` |
| Get bodies | `kd ask --id <ID>...` |
| Expand graph | `kd ask --around <ID> [--via <EDGE>] [--depth N]` |
| Most trusted | `kd ask --rank --universe` |
| Tag vocabulary | `kd ask --tags` |
| Semantic suggestions | `kd ask --suggest [--top-k N] [--relation duplicate\|contradiction\|link]` |
| Filters | `--type --class --tag --status --scope --anchor --since --until --as-of` |
| Record | `kd write --summary "<s>" [<body>\|-] [--type T] [--tag T] [--anchor P] [--class C] [--status S]` |
| Version | `kd write --update <ID> --summary "<s>" [--clear-anchors]` |
| Edge (existing) | `kd write --link <FROM:EDGE:TO>` |
| Edge (new note) | `kd write --summary "<s>" --edge <EDGE:ID>` |
| Claim/provenance | `kd write --claim <S:R:O> [--agent N] [--activity N]` |
| Evidence | `kd write --outcome <success\|partial\|failure\|abandoned> --id <ID> [--note TXT]` |
| Batch | `kd write --batch <file.jsonl\|->` · `--params '<json>'` · `--dry-run` |
| New task | `kd task new --summary "<s>" --scope <epic\|issue\|task> [--parent ID] [--kind K] [--checks C] [--anchor P]` |
| Batch tasks | `kd task new --batch <file\|->` |
| List | `kd task list [--ready\|--blocked [--explain]] [--sort impact] [--scope ID] [--kind K]` |
| Show | `kd task show --id <ID>... [--history]` |
| Edit | `kd task update --id <ID> [--statement S] [--status S] [--parent ID] [--checks C] [--anchor P...] [--clear-anchors]` |
| Close | `kd task close --id <ID> --outcome success --note "..."` |
| WBS tree | `kd task graph [--root ID\|--program plan/<slug>.md]` |
| Flow / critical path | `kd task flow [--window-days N]` |
| Plan | `kd task plan <slug> [--prompt\|--submit] [--template <feature\|bug\|refactor>] [--step S] [--from ID]` |
| Handoff | `kd rewind [--scope C] [--files P...] [--budget N] [--resume <context_id>]` |
| Map | `kd map [--axis <anchor\|type\|classification\|scope>] [--semantic] [--communities] [--members] [--write]` |
| Health | `kd doctor [--fix] [--explain]` |
| Propose maintenance | `kd maintenance <learn\|compact\|prune> [--universe] [--verify] [--dry-run]` |
| Forget (soft) | `kd forget --id <ID> [--restore \| --purge [--force]]` |
| Embeddings queue | `kd drain [--status \| --digest [--force]]` |
| Worker | `kd drain service [--install\|--status\|--subscribe\|--unsubscribe\|--reconcile\|--uninstall] [--every 1h] [--port 8889]` |
| Config | `kd config <get\|set\|unset\|list\|promote> [--key K] [--value V] [--global]` |
| Install / integrate | `kd init` · `kd self <version\|setup <client>\|completions <shell>\|upgrade>` |
| Commit | `kd sync [--message M]` |

## Closed sets (D212)

Every flag with a fixed value list **rejects an invalid value** with the full list + closest match
(`did you mean …`). An **absent** flag validates nothing (no error, no list).

| Flag/field | Values |
|---|---|
| `--type` | `fact decision question task def error snippet link meta risk` |
| `--class` | `foundational tactical observational` |
| `--status` | `active in_progress blocked closed superseded forgotten` |
| `--scope` | `epic issue task` |
| `--kind` | `task error question risk decision` |
| `--outcome` | `success partial failure abandoned` |
| edges (`--link`/`--edge`/`--via`) | `references depends_on contradicts supports extends replaces rejects results_in same_as broader narrower related` |
| `--relation` | `duplicate contradiction link` |
| `--axis` | `anchor type classification scope` |
| `--sort` | `impact` |
| `--template` | `feature bug refactor` |
| `self setup` | `claude cursor codex pi` |
| `self completions` | `bash zsh fish` |
| `--log-level` | `error warn info debug trace off` |

## List syntax (D210)

List flags accept **repetition or comma** (equivalent): `--tag a --tag b` ≡ `--tag a,b`. Applies to
`--id --type --class --tag --anchor --edge --claim --checks --files`. The **space form**
(`--id a b`) does **not** exist (exit 2); in `ask`, `--id`/`--around` **conflict** with a textual
query. **Free text** (query, body, `--step`) is never split — use `--params '<json>'` for arrays.

## Anchors × edges

- **Anchor** (`--anchor src/x.rs`): ties the note to a **file/glob**. The only link to code; powers
  `kd ask --anchor` and anchor drift. Free vocabulary.
- **Edge** (`--link "A:contradicts:B"`): ties the note to **another note** with a **type** (12
  edges, closed). Directed. Powers `ask --around --via`, `task list --ready`/impact, curation
  (`contradicts`/`replaces`), the light ontology (`same_as`/`broader`/`narrower`/`related`) and
  `doctor`.

Anchor = "where"; edge = "how it connects to another note". Both are complementary.

## Output contract

- **stdout = data; stderr = logs.** In `--json`, stdout is exactly
  `{success, command, data?, error?, warnings?}`.
- Empty search → `[no_results]`, exit 0. Broken pipe → exit 0 (D73).
- `warnings[]` = graceful degradation (e.g., embeddings down → BM25). With `behavior.strict`,
  warnings become errors.

Exit codes: `0` ok · `2` invalid usage · `3` not found · `4` conflict · `5` io · `6` timeout ·
`7` bad config · `8` bad schema/note · `70` internal (`101` reserved for panic).

## Reading output

| Signal | Meaning / action |
|---|---|
| `why = file_match` / `anchor_match` | Matched by working-set file / anchor id. |
| `why = tracker_match` | Belongs to the requested `--scope`. |
| `why = stars` | Derived confirmation (`outcomes` or successful tasks). |
| `why = semantic` | Vector match (paraphrase). |
| `why = recent` / `universal` | Recency / fallback. |
| `channels` (`--json`) | RRF shares (`lexical`/`anchor`/`semantic`) + boosts (`recent`/`stars`). |
| `score <0.75` / `0.75–0.92` / `≥0.92` | Create / merge / reject (dedup). |
| `stale` / `expiring` / `pending` (`rewind`) | Outdated / near expiry / embedding queued. |
| `epic: <id>\|<title> (done/total)` | Epic progress (closed work leaves). |
| `next:` (`rewind`) | Open `ready` tasks by impact. |

## Workflows

```bash
# Record
kd ask "gateway rate limit" --brief
kd write --summary "Rate limit is 100 rps per key" --type decision --tag gateway --anchor src/gateway.rs
kd write --link "decision_01m81b6h:extends:fact_01abc123"

# Plan + execute
kd task new --summary "Offline-first sync" --scope epic
kd task new --summary "Resolve merge conflict" --scope task --parent <epic>
kd task list --ready --sort impact
kd task close --id <task> --outcome success --note "tests green"

# Resume
kd rewind --budget 2000            # manifest + next:/fresh:
kd rewind --files src/gateway.rs   # working set only
kd rewind --resume <context_id>    # resume 1:1

# Audit / maintain
kd doctor
kd maintenance learn --universe    # what should become a note? (scope required)
kd map --axis scope --semantic --universe
kd maintenance prune --universe    # proposes forget by shelf-life
kd drain --status                  # embedding queue
kd drain service --status          # auto-drain worker health

# Integrate (MCP)
kd self setup claude|cursor|codex|pi
# tools: knudge_pre_write, knudge_pre_edit, knudge_session_end, knudge_status
```

The worker (`kd drain service`): `--install` sets up a `systemd --user`/`launchd` scheduler + the
`knudge-embed` server and registers the project; `--subscribe`/`--unsubscribe` multi-project;
`--reconcile` aligns `endpoint`/`model` + reindexes; `--uninstall` keeps the GGUF. Script embedded;
GGUF pinned-revision + SHA-256 (D183). `embeddings.mode=lazy` (default) drains a batch after each
verb.

## Setup

`kd init` creates `.knudge/`, writes the protocol block into `AGENTS.md`, and installs the skill
`.agents/skill/kd/SKILL.md`. Installer (`install.sh`) drops `kd` + `knudge-mcp` into `~/.local/bin`
(SHA-256 checked); needs `git`; embeddings optional.

## Anti-patterns

- Writing without `kd ask` first → duplicates.
- Inventing an id — ids are derived; copy them.
- `kd write --type task` — rejected; use `kd task`.
- Creating edges from a task flag — edges only via `kd write --link`.
- Expecting `learn`/`compact`/`prune` to mutate the corpus — they only propose.
- Logging to stdout — in `--json`, stdout is only the envelope.
- Declaring a task done without `--outcome`.
- Committing `.idx/`/`cache/`/`contexts/` — derived; `kd init` handles `.gitignore`.
- Storing secrets in the body (logs redact, notes don't).

## Known limitations

- Embeddings need a local OpenAI-compatible server; without it `ask` degrades to BM25.
- `forgotten`/`superseded` are hidden from `ask` by default (`--status` includes them).
- `kd write` rejects `task`/`epic` (D93/D149).
- Tasks root at the epic: `epic ⊃ { issue ⊃ task | task }`; issue optional; depth ≤ 4.
- `learn`/`compact`/`prune` are read-only (proposals).
- The index is derived/rebuildable; changing the embedding model invalidates and re-embeds
  everything.

## Checklist

- [ ] Searched before writing (`kd ask "<draft>"`)?
- [ ] Short, self-contained summary, with `--anchor` for code?
- [ ] Correct `--class`/`--status`?
- [ ] Id copied from output (not invented)?
- [ ] Edges via `kd write --link`?
- [ ] Task with a single parent and depth ≤ 4?
- [ ] Closed with `--outcome` (evidence)?
- [ ] `kd sync` at the end to version `notas/` + `eventos/`?
- [ ] `--json 2>/dev/null` still valid JSON (no log leak)?
