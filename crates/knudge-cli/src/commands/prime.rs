//! `kd prime` — protocolo estático, byte-idêntico por versão (D57, E12-T01).
//!
//! É o "help da IA": tipos, tools, regras, orçamento e formato de saída, **sempre a mesma
//! resposta** para uma dada versão do binário (cacheável). `prime` é **compacto por padrão**
//! (D166); `--long` anexa a gramática TOON e o schema completo. O conteúdo é em **inglês**,
//! token-optimized (a IA que o consome opera melhor com menos tokens).

use knudge_core::schema::{CANONICAL_KEYS, NoteType};
use serde_json::json;

use crate::output::Output;

/// Versão do binário.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Protocolo **compacto** (default): o essencial para operar, byte-idêntico por versão.
const PRIME_COMPACT: &str = "\
knudge (kd) — project memory for LLMs.
TYPES: {TYPES}
CLASSIFICATION: foundational, tactical, observational
STATUS: active, in_progress, blocked, closed, superseded, forgotten

CYCLE: kd ask (search) → kd write (record) → kd task (work) → kd sync (commit).
  Always search before writing (dedup); use --limit N --brief to save tokens.

WHEN (use / don't):
  kd ask <QUERY>     recall before acting (read-only); --anchor/--type/--limit/--brief/--full-content.
  kd write <...>     record fact/decision/error/risk/question; anchor code with --anchor PATH.
  kd task new <...>  plan/execute work (epic ⊃ {issue ⊃ task | task}); NOT for knowledge.
  kd rewind          rebuild session context/handoff (token budget).
  kd doctor          corpus health: 13 checks + audit (--fix repairs; --explain details).
  kd drain           embedding queue: --status | --digest [--force]; service = worker.
  kd forget --id <ID> soft-delete (--restore; --purge [--force] after retention).
  kd sync            commit notas/ + eventos/ (--message).
  kd init|config|self|prime|help — setup, config, binary, protocol, help.

ESSENTIAL:
  kd ask \"<QUERY>\" [--anchor PATH...] [--type T...] [--limit N] [--brief] [--full-content]
  kd write --summary \"<statement>\" [<body>|-] --type <fact|decision|error|risk|question> [--tag T...] [--anchor PATH...]
  kd task new --summary \"<...>\" --scope <epic|issue|task> [--kind error|question|risk|decision] [--parent ID] [--anchor PATH...]
  ANCHORS: --anchor PATH ties note/task to code (repeatable/comma; glob `src/**`).
  BODY: write when the statement alone can't drive action (Why / Evidence / Consequence).

ID: <type>_<base36(8)> = hash(type + U+001F + normalize(statement)); reclassifying never rewrites the id.
RULES: never invent an id; keep statement short and self-contained; anchor code with --anchor PATH.
OUTPUT: stdout = data, stderr = logs. --json = {success, command, data?, error{code,message,retryable}, warnings?}
EXIT: 2 invalid, 3 not_found, 4 conflict, 5 io, 6 timeout, 7 config, 8 schema, 70 internal
";

/// Protocolo **completo** (`--long`): todos os verbos e detalhes. `{TYPES}` vem do schema.
const PRIME_FULL: &str = "\
knudge (kd) — project memory for LLMs.
TYPES: {TYPES}
CLASSIFICATION: foundational, tactical, observational
STATUS: active, in_progress, blocked, closed, superseded, forgotten

CYCLE: kd ask (search) → kd write (record knowledge) → kd task (work) → kd sync (commit).
  Always search before writing (dedup); use --limit N --brief to save tokens.

WHEN (use / don't):
  kd ask <QUERY>     recall before acting; start any task here.
                     NOT for create/edit (read-only) nor history (use rewind).
  kd write <...>     record fact/decision/error/risk/question. First `kd ask \"<draft>\"`.
                     NOT for work (`--type task` is rejected) — use kd task.
  kd task new <...>  plan/execute work (epic ⊃ {issue ⊃ task | task}).
                     NOT for knowledge/observation — use kd write.
  kd rewind          rebuild context at session start (token budget).
                     NOT as targeted search — use kd ask.
  kd map             cluster view (structural/semantic); needs scope or --universe.
                     NOT to find one note (kd ask) nor ranking (kd ask --rank).
  kd doctor          corpus health (13 checks + audit; --fix repairs, --explain details).
  kd maintenance ..  cleanup (compact/learn/prune; only PROPOSE, need scope or --universe).
  kd forget --id <ID> soft-delete (--restore; --purge only after retention, [--force]).
  kd sync            commit notas/ + eventos/ (--message).
  kd init|config|self|prime — setup, config, binary, this protocol.

ANCHORS (--anchor PATH) — ties memory to code (high value):
  Every note/task about a file or module should be anchored:
    kd write --summary \"Cache uses LRU\" --type decision --anchor src/cache.rs --anchor src/cache/**
    kd task new --summary \"Migrate cache\" --scope task --anchor plan/016.md
  Repeatable; comma ok (`--anchor a,b --anchor c`). Glob (`src/**`) matches subtrees.
  `kd ask --anchor PATH` finds anchored notes even without a text query (anchor channel).
  Don't use: global concept with no file, or a path that doesn't exist yet.
  Maintenance: `kd doctor` lists broken anchors (file removed).

KNOWLEDGE (kd write — facts, decisions, errors, risks, questions):
  1. kd ask \"<draft>\"                # lexical dedup (BM25) before creating
  2. score < 0.75 create | 0.75-0.92 merge | >= 0.92 reject
  kd write --summary \"<...>\" [<body>|-] [--type <fact|decision|error|risk|question>] [--tag T...] [--anchor PATH...]
  kd write --update <ID> --summary \"<...>\"   # versioned; changing type/statement makes new id + supersede
  kd write --update <ID> --params '<json>'  # patch (body/tags/anchors/...); --clear-anchors clears
  kd write --link <FROM:EDGE:TO>        # explicit edge (single path; includes depends_on)
  kd write --outcome <success|partial|failure|abandoned> --id <ID> [--note TXT]   # evidence (D55)
  kd write --batch - [--dry-run]          # JSONL draft batch (D110)
  kd write --params '<json>' [--dry-run]  # single item (D147)
  --type task is rejected: use kd task.

BODY (when and why) — the \"why\" the statement can't carry:
  Write a body when the statement alone can't drive action. Template (2–4 lines):
    Why: <reason/decision>
    Evidence: <command, output, error, link>
    Consequence: <what changes in practice>
  Expected for decision/error/risk; optional for an obvious fact.
  `ask` shows the body (1st hit full, 2–5 truncated); read with --id/--full-content.

SEARCH (kd ask — knowledge only by default; --with-task includes work):
  kd ask <QUERY> [--type T...] [--class C...] [--tag T...] [--status S...] [--scope ID]
        [--anchor PATH...] [--since TS] [--until TS] [--limit N] [--brief] [--full-content]
  kd ask --params '<json>'                # query + filters (D147); '-' reads query from stdin
  kd ask --id <ID>...                     # bodies by id (includes work)
  kd ask --around <ID> [--via EDGE] [--depth N]   # expand the graph
  Pipe (LLM): id|statement|score|why (1 hit/line); 1st hit with body, 2–5 truncated (D161).
              Full body of all with --full-content; --brief is id|statement only.
  Empty search → stdout [no_results] (D152); --json has channels per hit (D151).
  forgotten/superseded are out of ask by default; use --status to include them.
  Rank (`kd ask --rank`) and tag vocabulary (`kd ask --tags`) — D146/D209.

WORK (kd task — epic ⊃ { issue ⊃ task | task }; epic is the root, anchor it):
  kd task new --summary \"<...>\" [<body>|-] --scope <epic|issue|task> [--kind error|question|risk|decision]
        [--parent ID] [--checks NAME] [--tag T] [--anchor PATH] [--params '<json>'|--batch SOURCE]
  kd task list <filter> [--sort impact] [--full-content]   # filter: --scope/--status/--kind/--parent/--ready/--blocked/--tag/--anchor, or --universe
  kd task show --id <ID> [<ID>...] [--history]   # + body/checks/anchors/tags/outcomes (D125/D127/D137)
  kd task update --id <ID> ...              # edit in place (--anchor replaces; --clear-anchors clears)
  kd task close --id <ID> [--outcome S] [--note TXT]   # only declares with evidence (D55); + epic and progress (D127)
  kd task graph [--program plan/<slug>.md|--root ID]   # id|kind|status|statement (done/total) (D139)
  kd task plan <ID> --prompt [--template T] | --submit --from -   # TOON plan (D105)

STATE/HANDOFF: kd rewind [--scope SCOPE] [--files PATH...] [--budget N] [--resume ID]
  Corpus filters: --tag/--anchor/--type/--class/--around ID (D143).
  Budget ceil(len/4) tokens (default 4000); resumable context_id 1:1; next:/fresh: (D106).
MAP/RANK: kd map ...  ·  kd ask --rank|--tags ...  (map/rank need scope or --universe; D143/D146)
  kd map [--axis anchor|type|classification|scope] [--scope C] [--semantic] [--members] [--write]
  kd drain --status|--digest  (embedding queue: rich state and digest; --force reindexes all)
  kd drain service ..  (auto-drain worker: install/subscribe/unsubscribe/status/uninstall)
HEALTH: kd doctor [--fix] [--explain]  (13 checks + audit; reversible repair; --explain details each finding)
MAINTENANCE: kd maintenance compact|learn|prune  (propose only; need scope or --universe)
LIFECYCLE: kd forget --id <ID>  (soft; --restore; --purge [--force] after retention)
CONFIG: kd config get --key K | set --key K --value V | unset --key K | list [--global]  (strict is config, not a flag)
INIT/SYNC: kd init  (create .knudge/ + AGENTS.md)  ·  kd sync  (commit notas/ + eventos/)

ID: <type>_<base36(8)> = hash(type + U+001F + normalize(statement)); reclassifying never rewrites the id.
TOON: frontmatter in canonical order; optional keys omitted, never null.
RULES: never invent an id; keep statement short and self-contained; anchor code with --anchor PATH.
OUTPUT: stdout = data, stderr = logs. --json = {success, command, data?, error{code,message,retryable}, warnings?}
EXIT: 2 invalid, 3 not_found, 4 conflict, 5 io, 6 timeout, 7 config, 8 schema, 70 internal
";

/// Formato de `prime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimeFormat {
    /// Protocolo compacto (default; byte-idêntico).
    Compact,
    /// Protocolo completo + schema/gramática (`--long`).
    Long,
}

/// Resposta de `prime` (estática; `Long` anexa schema/gramática).
#[must_use]
pub fn run(format: PrimeFormat) -> Output {
    let text = match format {
        PrimeFormat::Compact => prime_text(PRIME_COMPACT),
        PrimeFormat::Long => format!("{}{}", prime_text(PRIME_FULL), long_section()),
    };
    let data = match format {
        PrimeFormat::Compact => json!({ "protocol": &text, "version": VERSION }),
        PrimeFormat::Long => json!({
            "protocol": &text,
            "version": VERSION,
            "types": NoteType::ALL.iter().map(|t| t.as_str()).collect::<Vec<_>>(),
            "keys": CANONICAL_KEYS,
        }),
    };
    Output::new(text, data)
}

/// Protocolo com os tipos vindos do schema (fonte única).
fn prime_text(body: &str) -> String {
    let types = NoteType::ALL
        .iter()
        .map(|note_type| note_type.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    body.replace("{TYPES}", &types)
}

fn long_section() -> String {
    let types: Vec<&str> = NoteType::ALL.iter().map(|t| t.as_str()).collect();
    let keys: Vec<&str> = CANONICAL_KEYS.to_vec();
    format!(
        "\ntypes: {}\ncanonical keys ({}): {}\n",
        types.join(", "),
        keys.len(),
        keys.join(", ")
    )
}
