---
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
kd prime [--long]                          # usage protocol (--long = full; once per session)
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
kd task new --summary "<s>" --scope <epic|issue|task> [--kind K] [--parent ID] [--anchor P]
kd task list --ready [--sort impact]       # or --blocked [--explain]
kd task show --id <ID> [--history]         # body/checks/outcomes + epic rollup
kd task close --id <ID> --outcome S        # only declares with evidence
kd task flow                               # cycle/lead/throughput + critical path
kd rewind [--budget N] [--files P...]      # resume context between sessions
kd doctor [--fix] [--explain]              # base health
kd map [--axis A] [--communities]          # knowledge map
kd maintenance <learn|compact|prune>       # proposes only (read-only)
kd drain [--status | --digest [--force]]   # embedding queue: state / process
kd sync [--message M]                      # commit notas/ + eventos/
```

## Work (tasks)

Work is a derived view: `epic ⊃ { issue ⊃ task | task }` — `epic` is the root (may stand
alone), `issue` is optional, `task` is the leaf. `--scope` sets the level
(`epic`/`issue`/`task`); `--kind` sets the species (`task`/`error`/`question`/`risk`/
`decision`). All work lives in `kd task`; `kd write --type task` is rejected.

Anchoring the root epic is strongly recommended (`kd doctor` warns without it). Parent is any
ancestor with a strictly lower rank, so `epic → task` directly is valid. Closing requires an
`--outcome`; `kd task show`/`close` report the epic rollup (`done/total`).

## Embeddings (derived, optional)

Vectors are derived and never block the corpus. `kd drain --status` shows the queue,
`kd drain --digest` processes it, and `--force` wipes `.idx/` and rebuilds from scratch. A down
provider degrades gracefully to lexical search (`warnings`, never a hard error).

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

<!-- knudge:skill:version: 3 -->
