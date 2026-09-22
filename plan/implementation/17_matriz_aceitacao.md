# 17 — Matriz de aceite por tool (E13-T06)

> Para cada verbo/subcomando: o **formato pipe**, o envelope `--json`, o **erro** e seu **exit
> code**, e o **estado esperado de `.knudge/`**. É o critério de "pronto" de uma tool (D78).
>
> Convenções: stdout = dados; stderr = logs. Exit codes de `ErrorKind`: `2` invalid_input,
> `3` not_found, `4` conflict, `5` io, `6` timeout, `7` config, `8` schema, `9` unsafe_blocked,
> `70` internal (`101` reservado a panic). `EPIPE` → `0` (D73).
> Onde há teste automatizado, a coluna **Teste** aponta o arquivo.

## Verbos

| Tool | Pipe (texto) | `--json` (`data`) | Erro típico (exit) | Estado em `.knudge/` | Teste |
|---|---|---|---|---|---|
| `kd` / `kd prime` | protocolo estático, byte-idêntico | `{protocol, version}` | — | nada (read-only) | `golden::prime_matches_golden`, `cli::no_args_equals_prime` |
| `kd prime --long` | protocolo + gramática TOON | idem + schema | — | nada | `cli::json_envelope_for_prime_is_valid` |
| `kd init` | `projeto <nome> fundado em <root>/.knudge` | `{project, root}` | `config` (7) se já existe sem `--force` | cria `notas/`, `eventos/`, `.idx/`, `.locks/`, `config.toml`, `AGENTS.md` | `golden::init_message_matches_golden`, `cli::init_write_ask_roundtrip` |
| `kd rewind` | manifest/escopo formatado | `{context_id, items[], embeddings_pending}` | `io` (5) se store ausente | escreve `.idx/contexts/<context_id>.json` | `handoff::tests::rewind` |
| `kd rewind --resume <id>` | handoff 1:1 | idem | `not_found` (3) se `context_id` desconhecido | lê `.idx/contexts/` | `handoff::tests::context` |
| `kd ask` | `id\|statement\|score\|why` (1/linha) | `{hits[], warnings[]}` | `io` (5) se índice ilegível | nada (read-only) | `retrieval::tests::recall`, `cli::init_write_ask_roundtrip` |
| `kd ask --id <id>` | corpo da nota | `{notes[]}` | `not_found`/`io` (3/5) se id ausente | nada | `golden::json_error_envelope_matches_golden` |
| `kd ask --around <id>` | subgrafo formatado | `{nodes[], edges[]}` | `not_found` (3) | nada | `graph::tests::*` |
| `kd ask --anchor <path...>` | `id\|statement\|score\|why` das notas ancoradas (só o path basta; repetível, aceita vírgula) | `{hits[]}` | — (vazio se nada casa) | nada (read-only) | `retrieval::tests::recall::anchor_channel_recalls_with_empty_text`, `cli::ask_anchor_finds_note_without_query`, `cli::ask_anchor_accepts_comma_separated_and_repeated` |
| `kd write` | `created\|merged\|rejected\|unchanged\|<id>\|r<N>` | `{action, id, revision}` | `schema` (8)/`invalid_input` (2) | nota nova em `notas/<id>.md` + evento `write` | `write::tests::*`, `cli::init_write_ask_roundtrip` |
| `kd write --update <id>` | `updated\|<id>\|r<N>` | `{action, id, revision}` | `conflict` (4) se id inexistente/`forgotten` | nova revisão + evento `update` | `write::tests::update` |
| `kd write --link` | `linked\|<aresta>\|<from>-><to>` | `{edge, from, to}` | `invalid_input` (2) se id inválido | atualiza aresta no frontmatter + evento `link` | `write::tests::lifecycle` |
| `kd write --dry-run` | igual ao write | igual, sem persistir | idem | **nada** | `write::tests::dedup` |
| `kd write --outcome <status> <ID>` | `outcome\|<id>\|<status>\|r<N>` | `{action, id, outcome, revision}` | `not_found` (3) se id ausente; `invalid_input` (2) se combinado com `--update`/`--link`/`--dry-run` | anexa `outcomes[]` + evento `outcome` | `write::tests::outcome::*`, `cli::write_outcome_on_note_returns_outcome_action` |
| `kd task new [--source F]` | `task_xxx` | `{id, scope, status}` | `invalid_input` (2) se `scope` inválido/profundidade > 4 | `notas/<id>.md` (`type=task`/`container`) + evento | `task::tests::hierarchy` |
| `kd task list [--ready\|--blocked [--explain]]` | `id\|scope\|status\|statement[\|motivo]` | `{tasks[]}` | `invalid_input` (2) se `--ready`+`--blocked` ou `--explain` sem `--blocked` | nada | `retrieval::tests::views::block_reason_*`, `cli::task_list_ready_blocked_and_explain` |
| `kd task graph --program <path>` | path + `id\|scope\|status\|statement` indentado | `{program, root, nodes[]}` | `not_found` (3) se sem Épico-raiz | nada (read-only) | `task::tests::program::*`, `cli::task_graph_program_renders_subtree` |
| `kd task show` | nota + histórico | `{task, history[]}` | `not_found` (3) | nada | `task::tests::lifecycle` |
| `kd task update` | `updated\|<id>` | `{id, changed[]}` | `invalid_input` (2) | revisão + evento | `task::tests::lifecycle` |
| `kd task close` | `closed\|<id>\|<outcome>` | `{id, outcome, evidence[]}` | `invalid_input` (2) sem evidência | grava `outcomes[]`/`evidence` | `task::tests::lifecycle` |
| `kd task plan` | passos numerados | `{steps[]}` | `conflict` (4) em ciclo/self-ref | marcador `blocks` no corpo | `task::tests::hierarchy` |
| `kd maintenance doctor` | `ok\|<check>\|<msg>` por linha | `{checks[], ok}` | `internal` (70) se store corrompido | read-only (com `--fix`, corrige reversível) | `health::tests::doctor` |
| `kd maintenance doctor --fix` | idem + correções | idem + `{fixed[]}` | idem | reconstrói `.idx/`, remove resíduos | `health::tests::doctor` |
| `kd maintenance compact` | `propostas` (merge/supersede) | `{proposals[]}` | — | read-only (propõe; nada sem aceite) | `maintenance::tests::compact` |
| `kd maintenance eval --ab` | métricas | `{recall_at_k, ndcg_at_k, mrr}` | `config` (7) sem dataset | read-only | `embeddings::tests::eval` |
| `kd maintenance index --status` | `pending: N` | `{pending, indexed, stale}` | `config` (7) provider inválido | lê `.idx/embeddings.jsonl` | `embeddings::tests::state` |
| `kd maintenance index --drain` | `indexed=… pending=… stale=… cache_hits=…` | idem | degrada com `warnings[]` se provider offline | atualiza `.idx/embeddings.jsonl`/`emb_cache.jsonl` | `embeddings::tests::pipeline` |
| `kd maintenance learn` | `kind\|ids\|score\|why` | `{proposals[]}` | — | read-only | `maintenance::tests::learn` |
| `kd config get` | `chave = valor` | `{key, value, scope}` | `not_found` (3) se chave ausente | nada | `cli::*`, `config::tests` |
| `kd config set` | `chave = valor` | `{key, value, scope}` | `config` (7) se chave desconhecida | grava `.knudge/config.toml` (ou global) | `config::toml::tests` |
| `kd config unset` | `chave removida` | `{key, scope}` | `not_found` (3) | remove override | `config::tests` |
| `kd config list` | `chave = valor` por linha | `{entries[]}` | — | nada | `config::tests` |
| `kd forget <id>` | `forget\|<id>\|r<N>` | `{id, status}` | `not_found` (3) | `status=forgotten` + evento | `write::tests::lifecycle` |
| `kd forget --restore` | `restore\|<id>` | `{id, status}` | `conflict` (4) se não `forgotten` | `status=active` | `write::tests::lifecycle` |
| `kd forget --purge` | `purge\|<id>` | `{id, purged:true}` | `conflict` (4) antes da retenção | remove nota + derivado | `lifecycle::tests::retire` |
| `kd sync` | `sync\|<branch>\|<n> arquivos` | `{committed, files[]}` | `io` (5)/`config` (7) fora do git | commit de `notas/`+`eventos/` | `git::tests::sync` |
| `kd self version` | `kd <versão>` | `{name, version}` | — | nada | `golden::json_version_matches_golden` |
| `kd self completions` | script do shell | `{shell, script}` | `invalid_input` (2) shell desconhecido | nada | `cli::*` |
| `kd self setup` | `recipe <alvo> gravada em …` | `{target, path}` | `invalid_input` (2) alvo desconhecido | grava `.knudge/setup/<alvo>.json` | `cli::*` |
| `kd self upgrade` | orientação de upgrade | `{channel}` | — | nada | `cli::*` |

## Invariantes transversais

| Invariante | Como é verificado |
|---|---|
| `--json 2>/dev/null` é JSON válido (nenhum log em stdout) | `cli::json_envelope_for_prime_is_valid`, `golden::*` |
| `kd` == `kd prime` (byte-idêntico) | `cli::no_args_equals_prime`, `golden::prime_matches_golden` |
| EPIPE → exit 0 | `golden::epipe_is_exit_zero`, `cli::broken_pipe_exits_zero` |
| Comando desconhecido → exit 2 | `cli::unknown_command_exits_two` |
| `strict` só via config (sem `--strict`) | `write::tests::strict`, `config::tests` |
| `learn`/`compact`/`doctor` não escrevem sem aceite | `maintenance::tests::*`, `health::tests::doctor` |
| Exit code = `ErrorKind::exit_code()` | `golden::json_error_envelope_matches_golden` |

## MCP (`knudge-mcp`, E14)

Transporte JSON-RPC 2.0 sobre stdio; **uma linha JSON por mensagem**; stdout só tem protocolo.

| Tool | `params.arguments` | `result.structuredContent` | Erro | Teste |
|---|---|---|---|---|
| `knudge_pre_write` | `{candidates:[{id,statement?,score}]}` | `{hints:[{kind:"duplicate",ids,score,why,observed}]}` | argumento inválido → `isError` | `tests::tools::pre_write_defaults_statement_to_id` |
| `knudge_pre_edit` | `{items:[{id,statement?,score}]}` | `{hints:[{kind:"context",…}]}` | idem | `tests::server::tools_call_pre_write_returns_pointers` |
| `knudge_session_end` | `{writes,proposals:[{kind,ids,why?,score?}]}` | `{hints:[…]}` e fecha a sessão | idem | `tests::tools::session_end_advances_session` |
| `knudge_status` | `{}` | `{observing,sessions_seen,observation_sessions,cap}` | — | `tests::tools::status_reports_observation` |

| Método | Pipe | `--json`/resultado | Erro (código JSON-RPC) | Teste |
|---|---|---|---|---|
| `initialize` | protocolo | `{protocolVersion,capabilities,serverInfo}` | — | `tests::server::initialize_negotiates_supported_version` |
| `notifications/initialized` | — | sem resposta | — | `tests::server::initialized_notification_sets_flag_without_response` |
| `ping` | protocolo | `{}` | — | `tests::server::ping_returns_empty_object` |
| `tools/list` | protocolo | `{tools:[…]}` | — | `tests::server::tools_list_has_four_tools` |
| `tools/call` | protocolo | `{content,structuredContent,isError}` | `name` ausente → `-32602` | `tests::server::missing_tool_name_is_invalid_params` |
| método desconhecido | protocolo | — | `-32601` | `tests::server::unknown_method_is_not_found` |
| linha inválida | protocolo | — | `-32700` com `id: null` | `tests::transport::parse_error_gets_null_id` |

Invariantes: `EPIPE`/EOF → exit 0; `--help`/`--version` → exit 0; config ausente → defaults.

## Como manter

1. Ao adicionar/alterar um verbo, adicione/atualize a linha **e** o teste apontado.
2. Se a saída mudar de propósito, atualize `crates/knudge-cli/tests/golden/`.
3. A linha só vale como "pronta" com o teste verde no `make check`.
