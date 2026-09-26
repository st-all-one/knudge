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
| `kd` / `kd prime` | protocolo **compacto** estático, byte-idêntico (D166) | `{protocol, version}` | — | nada (read-only) | `golden::prime_matches_golden`, `golden::json_prime_matches_golden`, `cli::no_args_equals_prime` |
| `kd prime --long` | protocolo + gramática TOON | idem + `types`/`keys` | — | nada | `golden::prime_long_matches_golden`, `cli::json_envelope_for_prime_is_valid` |
| `kd init` | `projeto <nome> fundado em <root>/.knudge` | `{project, root}` | `config` (7) se já existe sem `--force` | cria `notas/`, `eventos/`, `.idx/`, `.locks/`, `config.toml`, `AGENTS.md` | `golden::init_message_matches_golden`, `cli::init_write_ask_roundtrip` |
| `kd rewind [--scope C] [--tag T] [--anchor P] [--type T] [--class C] [--around ID] [--depth N]` | manifest/escopo formatado (`next:` + `fresh:` no manifest; filtros de corpus escopam o handoff — D143) | `{context_id, items[], dropped, embeddings_pending}` | `io` (5) se store ausente | escreve `.idx/contexts/<context_id>.json` | `handoff::tests::rewind`, `cli::rewind_manifest_shows_next_and_fresh`, `cli::d143_rewind_filters_scope_handoff` |
| `kd rewind --files <path...>` | itens ranqueados (`id\|statement\|tier`; notas confirmadas por tarefa viram `star` — X1/D108) | `{context_id, items[], dropped}` | `io` (5) se store ausente | escreve `.idx/contexts/<context_id>.json` | `handoff::tests::manifest`, `cli::rewind_files_promotes_task_confirmed_note` |
| `kd rewind --resume <id>` | handoff 1:1 | idem | `not_found` (3) se `context_id` desconhecido | lê `.idx/contexts/` | `handoff::tests::context` |
| `kd ask` | `id\|statement\|score\|why` (1/linha; `why = semantic` se veio do vetor — D121; `limit` default 5; canal vetorial se `recall.semantic`; `hits[].confidence` inclui confirmação por tarefa — X1/D108; `hits[].channels` com parcelas RRF+boosts no `--json` — D151; só conhecimento por padrão, `--with-task` inclui trabalho — D146; busca vazia → `[no_results]` — D152) | `{query, hits[], warnings[]}` | `io` (5) se índice ilegível | nada (read-only) | `retrieval::tests::recall`, `cli::d146_ask_knowledge_only_and_with_task`, `cli::d151_ask_json_exposes_channel_contributions`, `cli::d152_empty_search_prints_no_results` |
| `kd ask --id <id>` | corpo da nota | `{notes[]}` | `not_found`/`io` (3/5) se id ausente | nada | `golden::json_error_envelope_matches_golden` |
| `kd ask --around <id>` | subgrafo formatado | `{nodes[], edges[]}` | `not_found` (3) | nada | `graph::tests::*` |
| `kd ask --as-of <TS>` | banner `as_of=<TS>` + `id\|statement\|score\|why` do corpus **ativo em T** | `{query, as_of, hits[]}` (cada hit com `historical: true`) | `invalid_input` (2) `T` no futuro; `[no_results]` se vazio | nada (read-only) | `retrieval::tests::temporal::*` |
| `kd knowledge tags [--limit N]` | `tag\|count` (1/linha; `count` desc, `tag` asc) | `{tags[]}` | — | nada (read-only) | `retrieval::tests::tags::*`, `cli::d146_knowledge_rank_and_tags` |
| `kd ask --anchor <path...>` | `id\|statement\|score\|why` das notas ancoradas (só o path basta; repetível, aceita vírgula) | `{hits[]}` | — (vazio se nada casa) | nada (read-only) | `retrieval::tests::recall::anchor_channel_recalls_with_empty_text`, `cli::ask_anchor_finds_note_without_query`, `cli::ask_anchor_accepts_comma_separated_and_repeated` |
| `kd knowledge rank [--type T] [--class C] [--tag T] [--anchor P] [--around ID] [--universe] [--limit N]` | `id\|statement\|score\|why` (`confidence` desc, `id` asc) | `{ranked[]}` | `invalid_input` (2) sem escopo/`--universe` (D146) | nada (read-only) | `retrieval::tests::rank::*`, `cli::d146_knowledge_rank_and_tags` |
| `kd knowledge suggest [--top-k N] [--relation R] [--limit N]` | `relação\|from\|to\|score` (`duplicate`/`contradiction`/`link`) | `{suggestions[]}` | degrada com `warnings[]` sem índice vetorial; `[no_results]` se vazio | nada (read-only; advisory — D158) | `embeddings::tests::semantic::classify_pair_*` |
| `kd knowledge promote recommend [--universe] [--limit N]` | `id\|conf\|statement` | `{candidates[]}` | `invalid_input` (2) se `rules.enabled=false` | nada (read-only) | `knowledge::tests::promote::*` |
| `kd knowledge promote approve\|edit\|remove <ID>` | `id\|statement` do bloco | `{action, promoted[]}` | `not_found` (3) id fora do bloco; `conflict` (4) bloco cheio | escreve o bloco `knudge:rules` no `AGENTS.md` (nunca a nota — D157) | `knowledge::tests::promote::*` |
| `kd knowledge promote list` | `id\|statement` (1/linha) | `{promoted[], count, max, enabled}` | — | nada (read-only) | `knowledge::tests::promote::*` |
| `kd write` | `created\|merged\|rejected\|unchanged\|<id>\|r<N>` | `{action, id, revision}` | `schema` (8)/`invalid_input` (2) | nota nova em `notas/<id>.md` + evento `write` | `write::tests::*`, `cli::init_write_ask_roundtrip` |
| `kd write --update <id> [--params J] [--clear-anchors]` | `updated\|<id>\|r<N>` (id legado revisa no lugar; id derivável muda `type`/`statement` → `created`/supersede) | `{action, id, revision}` | `conflict` (4) se id inexistente/`forgotten`; `invalid_input` (2) com âncora vazia | nova revisão + evento `update` | `write::tests::update`, `cli::regressions_031::update_params_body_revises_in_place`, `cli::regressions_031::clear_anchors_removes_them` |
| `kd write --link` | `linked\|<aresta>\|<from>-><to>` | `{edge, from, to}` | `invalid_input` (2) se id inválido | atualiza aresta no frontmatter + evento `link` | `write::tests::lifecycle` |
| `kd write --dry-run` | igual ao write | igual, sem persistir | idem | **nada** | `write::tests::dedup` |
| `kd write --outcome <status> <ID>` | `outcome\|<id>\|<status>\|r<N>` | `{action, id, outcome, revision}` | `not_found` (3) se id ausente; `invalid_input` (2) se combinado com `--update`/`--link`/`--dry-run` | anexa `outcomes[]` + evento `outcome` | `write::tests::outcome::*`, `cli::write_outcome_on_note_returns_outcome_action` |
| `kd write --batch <FONTE\|-> [--dry-run]` | `action\|id` (1/linha; `dry-run\|` prefixa) | `{dry_run, items[], warnings[]}` | `invalid_input` (2) acima de `write.batch_max`; item inválido vira `warnings[]` | grava os itens válidos (ou nada com `--dry-run`) | `write::tests::batch::*`, `cli::write_batch_jsonl_creates_and_dry_run` |
| `kd task new --summary <TXT> [<corpo>|-] --scope <epic\|issue\|task> [--kind K] [--tag T] [--anchor P] [--params J\|--batch F] [--dry-run]` (sem arestas diretas — D126/D140/D141; `id` no objeto = update e aplica `anchors`) | `key\|id\|scope\|status\|statement` (1/linha) | `{items[], edges, keys}` | `invalid_input` (2) se `scope` inválido/profundidade > 4; `schema` (8) se `--kind` incoerente | `notas/<tipo>/<id>.md` (`type` = `kind`) + evento | `task::tests::hierarchy`, `task::tests::batch::*`, `cli::task_batch_creates_with_parent_and_edges` |
| `kd task list <filtro> [--sort impact] [--full-content]` (filtro: `--scope` (nível `epic`/`issue`/`task` ou id do container-raiz)/`--status/--kind/--parent/--ready/--blocked/--tag/--anchor` ou `--universe`) | `id\|scope\|status\|statement[\|motivo][\|unblocks=N]` (bloco completo com `--full-content`) | `{tasks[]}` (`impact` por linha com `--sort impact`) | `invalid_input` (2) sem escopo/`--universe` (D144), `--ready`+`--blocked`, `--explain` sem `--blocked`/`--sort impact` | nada | `retrieval::tests::views::block_reason_*`, `task::tests::impact::*`, `cli::d144_task_list_requires_scope`, `cli::regressions_031::task_list_scope_accepts_container_id` |
| `kd task graph [--program <path>\|--root <id>]` | `id\|kind\|status\|statement (done/total)` indentado (floresta para `plan.md` com vários épicos — D139) | `{program, roots[], nodes[]}` (`progress{done,total}`) | `not_found` (3) se sem épico-raiz/raiz | nada (read-only) | `task::tests::program::*`, `cli::task_graph_program_renders_forest_for_multiple_epics` |
| `kd task show --id <ID> [<ID>...] [--history]` | bloco completo: `id\|statement`, `scope`, `tipo`, `status`, `corpo:`, `checks:`, `ancoras:`, `tags:`, `outcomes:`, `context:`, `historico:` (D125/D127/D137) | `{tasks[]}` com `kind`/`checks`/`anchors`/`tags`/`outcomes`/contexto | `not_found`/`io` (3/5) só se **nenhum** id existir; ausente parcial vira `warnings[]` | nada | `task::tests::context::*`, `cli::task_show_includes_full_fields`, `cli::task_show_lists_outcomes` |
| `kd task update --id <ID> [--anchor P...\|--clear-anchors]` | `updated\|<id>\|r<N>` | `{id, revision}` | `invalid_input` (2) com âncora vazia ou `--anchor`+`--clear-anchors` | revisão + evento; `--anchor` substitui, `--clear-anchors` limpa | `task::tests::lifecycle`, `cli::regressions_031::task_update_replaces_and_clears_anchors` |
| `kd task close --id <ID> [--outcome S] [--note TXT]` | `closed\|<id>\|<outcome>\|r<N>` + linha `epico: <id>\|<título> (<done>/<total>)` (D127) | `{id, outcome, evidence[], epic{done,total}}` | `invalid_input` (2) sem evidência ou `--note` sem `--outcome` | grava `outcomes[]`/`evidence` (com `notes`) | `task::tests::lifecycle`, `cli::task_close_note_records_outcome_reason`, `cli::task_close_reports_epic_progress` |
| `kd task plan <ID> --prompt [--template T]` | TOON derivado (`template`/`sections`/`min_steps`…) | `{template, seed, prompt}` | `invalid_input` (2) template desconhecido; `schema` (8) id não-container | nada (read-only) | `task::tests::plan::prompt_*`, `cli::task_plan_prompt_and_submit_from_file` |
| `kd task plan <ID> --submit --from -\|<arquivo>` | ids dos filhos (1/linha) | `{plan, template, children[]}` | `invalid_input` (2) seção/passos; `conflict` (4) id colide | cria N filhos (valida tudo antes) | `task::tests::plan::*`, `cli::task_plan_invalid_from_writes_nothing` |
| `kd doctor` | `ok\|warn\|fail <check> <msg>` por linha + `auditoria:`/`próximos:` | `{healthy,degraded,status,checks[],fixed[],audit{...},suggestions[]}` (`audit` traz `duplicate_pairs`/`broken_anchor_details`/`missing_edge_details`/`stale_lock_details`/`integrity_issues`; `--explain` acrescenta `explain[]` com `esperado`/`encontrado`/`ação`) | `internal` (70) se store corrompido | read-only (com `--fix`, corrige reversível) | `cli::doctor_*`, `health::tests::doctor`, `health::tests::body::body_check_is_advisory_and_counts_missing_body`, `cli::regressions_031::audit_json_exposes_duplicate_pairs`, `cli::regressions_031::program_anchor_warn_keeps_corpus_healthy` |
| `kd doctor --fix` | idem + correções | idem + `{fixed[]}` | idem | reconstrói `.idx/`, remove resíduos | `health::tests::doctor` |
| `kd maintenance compact [--verify]` | `propostas` (merge/supersede); com `--verify`, `...\|gate=passed\|failed` | `{proposals[]}` (com `gate` no `--verify`) | `invalid_input` (2) sem escopo/`--universe` (D144) | read-only (propõe; nada sem aceite; `--verify` é read-only — D156) | `maintenance::tests::compact`, `cli::d144_maintenance_requires_scope` |
| `kd drain --status` | `enabled=… provider=… mode=… dimensions=…` + `indexed=… pending=… stale=…` + recomendação | `{enabled,provider,mode,dimensions,indexed,pending,stale,recommendation}` | `config` (7) provider inválido | lê `.idx/embeddings.jsonl`; não muta | `cli::drain_status_reports_states_and_recommendation`, `cli::drain_status_explains_when_disabled` |
| `kd drain --digest [--force]` | `indexed=… batches=… cache_hits=…` | `{enabled,batches,indexed,cache_hits,rebuilt,removed}` | degrada com `warnings[]` se provider offline | atualiza `.idx/embeddings.jsonl` e o cache (D148/D153); `--force` apaga `.idx/` (derivado) e redigeri tudo (D84/D170); **auto-drain ocioso** ao fim de verbos não-`maintenance`/`doctor`/`drain` quando `mode=lazy` (D131) | `embeddings::tests::pipeline`, `embeddings::tests::versioned`, `cli::drain_digest_force_rebuilds_derived`, `cli::idle_lazy_drains_queue_after_command`, `cli::idle_manual_mode_leaves_queue_pending`, `cli::eager_mode_is_rejected` |
| `kd maintenance learn [--verify] [--scope C] [--tag ...|--anchor ...|--type ...|--class ...|--around ...|--universe]` | `kind\|ids\|score\|why` (inclui `create_note` de tarefa fechada — X2/D111; item com `scope` compara só o `statement`); com `--verify`, `...\|gate=passed\|failed` | `{proposals[]}` (com `gate` no `--verify`) | `invalid_input` (2) sem escopo/`--universe` (D144) | read-only (D47; `--verify` é read-only — D156) | `maintenance::tests::learn`, `cli::d144_maintenance_requires_scope` |
| `kd maintenance prune [--scope C] [--tag ...|--anchor ...|--type ...|--class ...|--around ...|--universe]` | `forget\|id\|motivo` (1/linha) | `{proposals[]}` | `invalid_input` (2) sem escopo/`--universe` (D144) | read-only (propõe; aplica só com `kd forget`) | `cli::maintenance_prune_proposes_forget_for_expired`, `cli::d144_maintenance_requires_scope` |
| `kd maintenance watch-service [--install\|--subscribe\|--unsubscribe\|--status\|--uninstall] [--yes] [--dry-run]` | saúde ou resumo da ação | `{action, done, script}` ou `{dry_run, action, source, reference, command}` | `io` (5) se pré-flight/execução falhar; uso (2) se flags de ação conflitarem | instala/gerencia o agendador (`systemd --user`/`launchd`) e o servidor de embeddings persistente; multi-projeto; mutar exige confirmação (D132/D133) | `cli::watch_service_dry_run_reports_plan`, `cli::watch_service_defaults_to_status`, `cli::watch_service_action_flags_are_exclusive`, `cli::watch_service_declined_does_nothing`, `cli::watch_service_subscribe_runs_local_script` |
| `kd knowledge map [--axis A] [--scope C] [--semantic] [--members] [--write] [--tag T] [--anchor P] [--type T] [--class C] [--around ID] [--depth N] [--universe]` | `<axis>\|<key>\|<count>` (scope: `\|<título>`); membros indentados com `--members`; `semantic\|<axis>\|<key>\|groups=N` | `{docs, clusters[], semantic[]}` | `invalid_input` (2) sem escopo/`--universe` (D143) ou eixo desconhecido; degrada com `warnings[]` se sem índice de embeddings | read-only (D47); `--write` materializa `notas/MAP.md` + hubs (D150) | `lifecycle::tests::clusters::*`, `cli::d143_knowledge_map_scope_and_filters` |
| `kd config get` | `chave = valor` | `{key, value, scope}` | `not_found` (3) se chave ausente | nada | `cli::*`, `config::tests` |
| `kd config set` | `chave = valor` | `{key, value, scope}` | `config` (7) se chave desconhecida | grava `.knudge/config.toml` (ou global) | `config::toml::tests` |
| `kd config unset` | `chave removida` | `{key, scope}` | `not_found` (3) | remove override | `config::tests` |
| `kd config list` | `chave = valor` por linha | `{entries[]}` | — | nada | `config::tests` |
| `kd forget <id>` | `forget\|<id>\|r<N>` | `{id, status}` | `not_found` (3) | `status=forgotten` + evento | `write::tests::lifecycle` |
| `kd forget --restore` | `restore\|<id>` | `{id, status}` | `conflict` (4) se não `forgotten` | `status=active` | `write::tests::lifecycle` |
| `kd forget --purge [--force]` | `purge\|<id>` | `{id, action, forced}` | `invalid_input` (2) antes da retenção (use `--force` para tombstone `forgotten`/`superseded`) | remove nota + derivado | `lifecycle::tests::retire`, `cli::regressions_031::purge_force_releases_superseded_tombstone` |
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
