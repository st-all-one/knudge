# Changelog

Todas as mudanças relevantes do knudge. Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).

## [Não publicado]

### Corrigido
- **Views `ready`/`blocked`, `impact` e `next:` ignoravam espécies de trabalho (D120).**
  `compute_views_at`/`block_reason`/`impact` filtravam `type == task`, então itens criados com
  `--kind error|question|risk|decision` (D113) — que têm `scope` — sumiam das views embora
  aparecessem no `task list`/`graph`. Agora o critério é **espécie de trabalho com `scope`**
  (`NoteType::is_work_kind` + `Graph::is_work_item`); containers e conhecimento ficam de fora.
  Regressão: `retrieval::tests::views::error_kind_work_item_is_ready_but_knowledge_error_is_not`,
  `task::tests::impact::error_kind_work_item_counts_as_dependent`.
- **O canal lexical casava stopwords e afogava o vetorial (D122).** Termos funcionais (`de`,
  `a`, `o`…) e fragmentos de 1 caractere (o tokenizador ASCII quebra `são` → `s`,`o`) geravam
  votos lexicais espúrios: em `ask "tempestade de requisições"` o topo era um erro casado só por
  `de`. Agora `retrieval::token::content_terms` os descarta. Regressão:
  `retrieval::tests::token::content_terms_drop_stopwords_and_short_fragments`,
  `retrieval::tests::token::stopwords_are_sorted_for_binary_search`.
- **`kd ask --anchor <path>` voltava vazio sem query textual e ignorava âncoras-glob.** Agora
  `--anchor` alimenta o **canal** de âncoras (D81) — a consulta funciona só com o path, sem
  query — e `Filter.anchors` casa nas **duas direções** (o pedido como glob e a âncora da nota
  como glob sobre o caminho pedido), alinhado a `rewind --files`. Regressão:
  `retrieval::tests::filter::anchor_filter_accepts_note_glob_matching_requested_path`,
  `retrieval::tests::recall::anchor_channel_recalls_with_empty_text`,
  `cli::ask_anchor_finds_note_without_query`.
- **`kd ask --brief` e `--with-body` eram flags mortas.** Agora `--brief` emite `id|statement`
  (2 colunas) e `--with-body` anexa o corpo de cada hit (no pipe e em `data.hits[].body` no
  `--json`); `--id` continua trazendo o corpo, e `--id --brief` o omite. Regressão:
  `cli::ask_with_body_and_brief_contract`.
- **`kd ask` devolvia notas `forgotten`/`superseded` por padrão.** Como `forget` é soft-delete
  (D43), o `ask` sem `--status` agora exclui esses dois estados; `--status forgotten` (ou
  `superseded`) continua disponível para inspecionar a linhagem. Regressão:
  `cli::forgotten_note_is_hidden_from_default_ask`.
- **`kd prime` entregava só um placeholder de 5 linhas.** Agora imprime o protocolo estático
  completo (D57): tipos, classificação/status, fluxo de escrita em duas fases (0.75/0.92),
  pesquisa, estado/handoff e orçamento, tarefas, manutenção, ciclo de vida, config, formato de
  saída/exit codes e regras de `id`/TOON. `--long` anexa tipos e as 28 chaves canônicas.
  Goldens `prime.txt`/`json_prime.json` atualizados; `kd` continua byte-idêntico a `kd prime`.

### Alterado
- **`recall.default_limit` cai de 10 para 5** (D121): o `ask` devolvia hits demais para contexto
  de LLM. Ajuste por config (`kd config set recall.default_limit N`).
- **`kd prime`** passa a listar `write --batch`, `ask --rank`, `task show` multi-id,
  `task close --note`, os filtros `--tag`/`--anchor` de `task list` e `maintenance prune`;
  goldens `prime.txt`/`json_prime.json` regenerados.
- **`kd prime`** passa a listar `ask --tags` e as linhas `next:`/`fresh:` do `rewind`; goldens
  `prime.txt`/`json_prime.json` regenerados.
- **`kd prime`** passa a listar `task list --sort impact`; goldens regenerados.
- **`rewind`**: a linha `embeddings_pending=N` vira `fresh: stale=… expiring=… pending=N`
  (D106).
- **Confiança derivada**: `ConfidenceInput` ganha `task_confirmation` (X1/D108); o `ask` reflete
  a confirmação por tarefa em `hits[].confidence`. Novo campo `Meta.scope` (persistido no índice
  derivado; opcional em índices antigos — D15).

### Adicionado
- **`why = semantic` no `ask`** (D121): um hit que veio pelo canal vetorial deixa de ser rotulado
  `recent` e passa a mostrar `semantic`, com precedência acima da recência genérica (mas abaixo
  de `stars`/`file_match`/`anchor_match`/`tracker_match`). Regressão:
  `retrieval::tests::recall::vector_channel_labels_hit_as_semantic`,
  `retrieval::tests::recall::stars_take_precedence_over_semantic`.
- **`kd ask --rank`** (K2/D107): ranqueia por confiança **derivada** sem query textual —
  `id|statement|confidence|why` (ordem `confidence desc, id asc`); o universo é só conhecimento
  (notas sem `scope`), já que itens de trabalho têm `task list --sort impact`. Reusa
  `confidence_score` + a confirmação por tarefa (X1). Regressão:
  `retrieval::tests::rank::*`, `cli::ask_rank_orders_by_confidence`.
- **`kd write --batch -`** (K4/D110): aplica um lote de rascunhos **JSONL** pelo mesmo protocolo
  de dedup (0.75/0.92), uma linha `action|id` por item; linha inválida vira `warnings[]` e o
  lote continua (R33); `--dry-run` só avalia. Teto `write.batch_max` (int, 100) ⇒ `invalid_input`.
  `Draft::from_value` rejeita chave desconhecida. Regressão: `write::tests::batch::*`,
  `cli::write_batch_jsonl_creates_and_dry_run`.
- **`kd task list --tag/--anchor/--since`** (T5/D104): reusa `retrieval::Filter` (tags/âncoras) e
  filtra por `created_at`; `kd task new` passa a aceitar `--tag`. Regressão:
  `cli::task_list_filters_by_tag_anchor_and_since`.
- **`kd task show <ID> [<ID>…]`** (T5/D104): mostra vários ids separados por `\n---\n`; `--json`
  devolve `data.tasks[]`; id ausente vira `warnings[]` (parcial) sem derrubar os demais.
  Regressão: `cli::task_show_multiple_ids_separator_and_partial`.
- **`kd task close --note <TXT>`** (T6/D104): o motivo entra em `outcomes[].notes` (exige
  `--outcome`). Regressão: `cli::task_close_note_records_outcome_reason`.
- **`kd maintenance prune`** (K5/D112): propõe `forget|id|motivo` por shelf-life vencido ou
  âncoras decaídas, reusando `demotion_candidates`; membros de ciclo ficam de fora (D45) e nada
  é gravado (D47) — a aplicação é `kd forget`. Regressão:
  `cli::maintenance_prune_proposes_forget_for_expired`.
- **`learn` com sinal de tarefa** (X2/D111): tarefa com `outcomes` de sucesso cuja âncora não tem
  nota ancorada vira proposta `create_note` (`why="tarefa fechada sem nota"`); read-only (D47).
  Regressão: `maintenance::tests::learn::success_task_*`.
- **`kd task list --sort impact`** (D109): ordena o caminho crítico por `(impacto desc,
  created asc, id asc)`, onde impacto = tarefas **abertas** que dependem transitivamente
  (`depends_on` reverso); `--explain` acrescenta `unblocks=N` e o `--json` traz `impact`. O
  modo `--sort impact` ignora `closed`/`superseded`/`forgotten` (ao contrário da view `--ready`,
  que os mantém — D104). `task::is_actionable` é compartilhado com o `next:` do `rewind`.
  Regressão: `task::tests::impact::adding_dependency_never_decreases_impact`,
  `task::tests::impact::actionable_excludes_terminal_statuses`,
  `cli::task_list_sort_impact_orders_critical_path`, `cli::task_list_sort_impact_skips_closed`.
- **Feedback derivado tarefa→conhecimento (X1/D108)**: tarefas com `outcomes` de sucesso que
  compartilham `anchors` confirmam a nota — `task_confirmation` entra no boost do BM25 (canal
  lexical), na confiança derivada (`hits[].confidence`) e promove a `star` no manifest de
  `rewind`. Peso em `recall.confirmation_from_tasks` (float, 0.1); sem `write` (D87). Regressão:
  `lifecycle::tests::from_tasks::*`, `lifecycle::tests::confidence::monotone_in_task_confirmation`,
  `retrieval::tests::bm25::task_boost_raises_score`,
  `retrieval::tests::recall::task_confirmation_raises_confidence_and_rank`,
  `cli::task_outcome_promotes_anchored_note_in_ask`,
  `cli::rewind_files_promotes_task_confirmed_note`.
- **`rewind` com `next:` e `fresh:`** (D106): o manifest dinâmico lista as tarefas `ready`
  **abertas** de maior impacto (`next:`) e o frescor do corpus
  (`fresh: stale/expiring/pending`); `K` deriva do orçamento e o excedente vira `dropped`.
  Novos `task::impact` (tarefas abertas que dependem transitivamente) e `lifecycle::freshness`
  (shelf-life + fila). Regressão: `task::tests::impact::*`, `handoff::tests::next::*`,
  `lifecycle::tests::shelf_life::freshness_counts_stale_expiring_and_pending`,
  `cli::rewind_manifest_shows_next_and_fresh`.
- **`kd ask --tags`** (D107): lista o vocabulário de tags (`tag|count`, `count` desc, `tag` asc),
  ignorando `forgotten`/`superseded`; `--limit N` e `--json` (`data.tags[]`). Regressão:
  `retrieval::tests::tags::*`, `cli::ask_tags_lists_vocabulary`.
- **Plano preenchível por LLM (`kd task plan --prompt`/`--from`)** (D105): `--prompt` deriva um
  prompt TOON read-only do template (`feature`/`bug`/`refactor`, com `min_steps`/`min_acceptance`);
  `--submit --from -|<arquivo>` lê o plano TOON, valida tudo **antes** de escrever (seções
  obrigatórias, passos, colisão de id) e cria os filhos. Templates em `.knudge/templates.toml`
  (subset TOML próprio, D97) sobrepõem os built-ins. Regressão: `task::tests::template::*`,
  `task::tests::plan::*`, `cli::task_plan_*`.
- **Papel derivado (`Role`)** (D115): `role(scope, type, tem_filhos)` projeta
  Initiative/Epic/Feature/Story/Sub-task/Bug/Spike/Risk/Decision — nunca armazenado.
  Regressão: `task::tests::role::*`.
- **Modo derivado (`Mode`)** (D116): `mode(container)` classifica
  sequential/concurrent/supervisor/handoff/magentic a partir de dono, filhos e sinais do log;
  `kd task graph [--root ID]` imprime `role|kind|status|owner|mode|statement` (antes só
  `--program`). Regressão: `task::tests::mode::*`, `cli::task_graph_reports_supervisor_mode`.
- **Canal vetorial no `kd ask`** (D102): quando `recall.semantic=true` (default) e há índice
  vetorial, o `ask` embute a query, ranqueia por similaridade (`rank_query`) e funde o canal via
  RRF — sem flag nova (D94). O canal é **filtrado** pelos filtros determinísticos
  (`--type`/`--class`/`--status`/`--tag`/`--anchor`), então não fura views. Provedor fora do ar
  degrada para BM25 com `warnings` (`strict` promove a erro). Config: `recall.semantic`,
  `recall.semantic_top_k`. Regressão: `embeddings::tests::semantic::rank_query_*`,
  `retrieval::tests::recall::vector_channel_respects_deterministic_filters`,
  `cli::ask_semantic_channel_reads_vector_index`.
- **Espécie do item de trabalho (`kd task new --kind`)** (D113): `scope` = nível, `type` =
  espécie. `--kind error|question|risk|decision|task` grava o `type` mantendo o `scope`;
  `scope` passa a ser aceito por qualquer item de trabalho e continua **exigido** para
  `task`/`container`. `kd task list --kind <K>`. Regressão: `task::tests::kind::*`,
  `cli::task_kind_sets_type_and_filters`.
- **Dono derivado de eventos (`kd task claim`)** (D114): `kd task claim <ID> --by <agente>` e
  `--release` gravam eventos `op=claim`/`release`; o dono é a projeção `ownership(events, id)`
  (último `claim` sem `release`/`close`). `kd task list --owner <A> | --mine` (usa
  `KNUDGE_AGENT`). Sem chave canônica. Regressão: `task::tests::ownership::*`,
  `cli::task_claim_sets_and_clears_owner`.
- **`kd write --outcome <status> <ID> [--note TXT]`** anexa evidência (`outcomes[]`) a
  **qualquer** nota (D103), não só a tarefas: a confiança derivada (D87) e o boost BM25 (E06)
  passam a valer para conhecimento confirmado por trabalho. Core: `write::outcome` (generaliza o
  `task::lifecycle::outcome`, sem `ensure_task`); evento `op=outcome`. Regressão:
  `write::tests::outcome::*`, `cli::write_outcome_on_note_returns_outcome_action`.
- **`kd task list --ready|--blocked [--explain]`** (D104): filtra pelas views derivadas
  `ready`/`blocked` (dependências + `not_before`); `--explain` (só com `--blocked`) acrescenta o
  motivo (`blocked_by=<id>`, `not_before=<ts>` ou `cycle`). Core: `retrieval::views::block_reason`.
  Regressão: `retrieval::tests::views::block_reason_*`, `cli::task_list_ready_blocked_and_explain`.
- **Programas externos (`plan/*.md`) como raiz de trabalho** (D119): o **Programa** é um arquivo
  markdown real (o "porquê"), ancorado ao **Épico-raiz** (`scope=epic`, sem pai) via `anchors`
  (D86) — nenhum `scope`/chave TOON nova. `kd task new … --source <arquivo>`; `kd task graph
  --program plan/<slug>.md` imprime a subárvore; `kd rewind --files plan/<slug>.md` inclui a
  subárvore; `doctor` ganha o check `program-anchor` (épico-raiz sem programa, programa órfão);
  config `programs.glob` (default `plan/*.md`). Core: `task::program::{root_for_path, program_of,
  subtree}`. Regressão: `task::tests::program::*`, `health::tests::doctor::program_anchor_*`,
  `cli::task_graph_program_renders_subtree`.
- **`kd ask --anchor` é repetível e aceita lista com vírgula.** `--anchor a,b --anchor c`
  consulta várias âncoras de uma vez; cada valor alimenta o canal de âncoras (D81). O `prime`
  e os goldens passam a documentar `[--anchor PATH...]`. Regressão:
  `cli::ask_anchor_accepts_comma_separated_and_repeated`.
- **Instalação** (`install.sh` + `make install`):
  - `make install` compila em release, instala `kd` e `knudge-mcp` em `~/.local/bin`
    (`PREFIX`/`BINDIR` mudam o destino), cria a config global
    (`~/.config/local/knudge/config.toml`), instala completions de bash/zsh/fish e ajusta o PATH.
  - `install.sh` no estilo `curl | bash`: baixa o release pré-compilado, verifica o checksum
    SHA-256 e instala; com `--from-source` (ou rodando de dentro do repositório) usa o build
    local. Suporta `--install-dir`, `--prefix`, `--version`, `--no-path`, `--no-completions` e
    `--uninstall`.
  - Nada é apagado de forma irreversível: artefatos antigos são **movidos** para
    `${XDG_CACHE_HOME:-~/.cache}/knudge/trash`.
  - `.github/workflows/release.yml` empacota `kd` + `knudge-mcp` (Linux musl, macOS e Windows)
    e publica `sha256sums.txt` no GitHub Release.
- **E14 — MCP: transporte JSON-RPC (stdio) e tools** (concluído):
  - **Codec JSON-RPC 2.0** puro (`crates/knudge-mcp/src/jsonrpc.rs`): `Request`/`Id`/`RpcError`,
    `parse` e emissores `result`/`error` com os códigos canônicos (`-32700`…`-32603`).
  - **Handshake MCP** (`src/protocol.rs` + `src/server.rs`): `initialize` negocia
    `protocolVersion` (suportadas `2025-06-18`/`2025-03-26`/`2024-11-05`), `notifications/initialized`
    e `ping`; notificações não geram resposta.
  - **Tools** (`src/tools.rs`): `knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end` e
    `knudge_status`, com `inputSchema`; argumento inválido vira `isError` sem derrubar o servidor.
  - **Transporte stdio** (`src/transport.rs` + `src/main.rs`): binário `knudge-mcp`, **uma linha
    JSON por mensagem**, stdout só protocolo, `EPIPE`/EOF → exit 0.
  - **Config**: `mcp.observation_sessions` (nova chave, default 3); o binário lê
    `mcp.hints_cap`/`mcp.observation_mode`/`mcp.observation_sessions` de `.knudge/config.toml`.
  - `kd self setup` passou a incluir o bloco `mcp` (`knudge-mcp --stdio`); `MODULE.md`,
    matriz de aceite e `DIVERGENCES.md` atualizados.
- **E13 — Testes e qualidade** (transversal, concluído):
  - **Golden** do binário (`crates/knudge-cli/tests/golden.rs` + `tests/golden/`): `prime`,
    `--json`, envelope de erro, erro em texto, `init` e EPIPE, com normalização de
    `<ROOT>`/`<NAME>`.
  - **Property tests** ampliados: TOON round-trip, RRF (determinismo, monotonicidade, união),
    confiança/decay (`[0,1]`, monotonicidade) e `id`/`body_hash` sob normalização.
  - **Stress de concorrência** sobre adaptadores reais (`crates/knudge-core/tests/stress.rs`):
    lock sem *lost update*, escritas concorrentes e leitor de índice durante rebuild.
  - **Crash-injection** com `FaultyFs`: nota-sem-evento, escrita atômica e crash no rebuild.
  - [`DIVERGENCES.md`](DIVERGENCES.md) — 21 bordas catalogadas com o teste que trava cada
    uma; [`plan/implementation/17_matriz_aceitacao.md`](plan/implementation/17_matriz_aceitacao.md)
    — matriz por tool (pipe/`--json`/erro/exit/estado).
  - **CI** (`.github/workflows/ci.yml`): `fmt`+`clippy`+`test`+linhas, `nextest`, doc-tests,
    `cargo deny`/`audit`/`machete`/`typos`, `miri` (core puro) e fuzz smoke (`fuzz/`).
  - Alvos extras no `Makefile`: `nextest`, `deny`, `audit`, `machete`, `typos`, `miri`, `fuzz`,
    `coverage`, `ci`.

### Corrigido
- **Lock advisory**: um lock recém-criado, ainda sem conteúdo visível (janela entre
  `create_exclusive` e a escrita do `at`), podia ser reclamado por outro processo. Agora o
  `mtime` decide e, sem `mtime`, o lock **não** é reclamado (E13-T03). Regressão em
  `store::tests::lock::fresh_unreadable_lock_is_not_reclaimed`.
- **`StdFs::rename`**: `NotFound` era mapeado para `ErrorKind::Io`, abortando o reclaim de lock;
  agora vira `ErrorKind::NotFound` (E13-T03).

### Adicionado (continuação)
- **E12 — CLI, MCP, hooks e distribuição** (Fase 4, concluído):
  - Superfície v2 completa: os 12 verbos (`init`, `prime`, `rewind`, `ask`, `write`, `task`,
    `maintenance`, `config`, `forget`, `sync`, `self`) wireados ao domínio via
    `knudge_cli::session::Session` (resolve projeto, carrega config efetiva, monta
    store/eventos/índice/grafo).
  - Envelope de máquina `{success, command, data?, error{code,message,retryable}, warnings?}`
    (D71/R31); `strict` de projeto promove `warnings[]` a erro (D94); EPIPE → exit 0 (D73).
  - **Hooks de ciclo de vida** (D59): porta `HookRunner` + `adapters::ProcessHookRunner`
    (sem shell, timeout e kill do grupo de processos); `pre-record` pode bloquear/mutar,
    `post-record`/`pre-prune`/`pre-compact` são executados na borda.
  - **MCP proativo estreito** (D68): `knudge-mcp::triggers::HintEngine` com 3 gatilhos, hints
    **ponteiro**, cap 3, dedup por sessão e modo observação.
  - `kd self completions <bash|zsh|fish>` e `kd self setup <claude|cursor|codex|pi>` (D69).
  - Novas chaves `hooks.*` na config; `TaskSpec` ganha `expires_at`/`not_before`; 11º check do
    `doctor` reporta o tamanho do índice/cache vetorial.
- **AGENTS.md** — guia de contribuição do repositório: padrões de desenvolvimento, erros,
  logs, testes, contrato de bytes e checklist de conclusão.
- **E11 — Embeddings** (Fase 3, concluído):
  - Porta `Embedder` (`ports`) e **provedor HTTP** OpenAI-compatible (`adapters::http`), cliente
    HTTP/1.1 bloqueante sobre `std::net` (sem `tokio`/`reqwest`), com timeout e retry idempotente.
    O modelo roda num **servidor local** (`llama-server` com o GGUF); **sem** inferência in-process
    (D101/R16/R43).
  - `EmbeddingMeta` (provider/model/revision/dimensões/similaridade) com *fingerprint*;
    `.idx/embeddings.jsonl` com cabeçalho `meta` que **invalida** o índice quando o modelo muda
    (D79).
  - `EmbeddingCache` por `body_hash` com teto e eviction **LRU**; falha de cache degrada para
    *pass-through* (D83/R14).
  - Fila derivada `indexed|pending|stale` + `EmbeddingMode`; `max_pending` como backpressure e
    catch-up; falha do provedor mantém `pending` (D80/D83).
  - Worker `drain` (reconcile off-path) + `FlushState` coalescido com *dirty flag* (D85).
  - Purga do vetor em toda remoção via `purge_derived` (D84).
  - Métricas puras `Recall@k`/`nDCG@k`/`MRR` e `ab_compare` (D90); `LightweightEmbedder`
    determinístico por SHA-256 para testes/CI (D89); consultas semânticas (vizinhos, duplicatas,
    sugestões de link) sempre como **proposta** (D42/D47).
  - `rewind` reporta `embeddings_pending` (D80).
- **E10 — Ciclo de vida, decay e clusters** (Fase 2, concluído):
  - `lifecycle::shelf_life`: TTL por `classification` — `foundational` nunca expira,
    `tactical`/`observational` com prazos configuráveis; `expires_at` explícito vence o
    derivado (D44).
  - `lifecycle::decay`: validade de âncoras (literal existe / glob casa) com varredura do
    projeto limitada; demolição após grace se a fração válida < threshold (D43).
  - `lifecycle::retire`: `retired_at` **derivado** dos eventos (`forget`/`supersede`); purga
    do conteúdo (nota + derivado) só após a janela de retenção (D48/D84).
  - `lifecycle::supersession`: demolição soft protegida — membros de ciclo de
    supersessão/dependência **não** demovem (D45).
  - `lifecycle::plan`: plano puro de demolição combinando shelf-life × decay × ciclos.
  - `lifecycle::clusters`: fase 1 estrutural determinística (`anchor`/`type`/`classification`/
    container) e fase 2 semântica **opcional e off-path**, com similaridade injetada (E11).
  - `not_before` como **28ª chave canônica** (D100): agendamento ortogonal à expiração;
    `compute_views_at` o considera, `compute_views` (prime) não (D56/D57).
  - Correções de contrato: `Classification` ganha `Ord`; `compute_views_at` exportado.
- **E09 — Validação, saúde e leitura tolerante** (Fase 2, concluído):
  - `health::validator`: catálogo `.knudge/validators.toml` (subset TOML — D99) e resolução
    `checks = explícitos ∪ globais ∪ por_âncora`; explícito ausente vira `missing[]` (D54).
  - `health::evidence`: fechamento por **evidência** — grava `evidence` + `outcomes[]`
    (`status/duration/agent/notes/recorded_at`) e **infere** o `outcome` pela severidade
    (`success`/`partial`/`failure`); sem evidência, não fecha (D48/D55).
  - `health::audit`: relatório puro de integridade, ciclos, âncoras quebradas, duplicatas,
    arestas sugeridas faltantes e locks stale (D46).
  - `health::doctor [--fix]`: 10 checks (schema, integridade, ciclos, âncoras, duplicatas,
    locks, config, `body_hash`, eventos, divergência canônico↔derivado) e reparo reversível
    **idempotente** (D19/D84). **E11** acrescenta o 11º check (tamanho do índice vetorial/cache).
  - `health::tolerant`: leitura Postel — chave desconhecida → warning; `type` desconhecido ou
    nota malformada → **skip + orientação**, sem derrubar o comando (D16–D18); `Config::strict`.
  - `health::anchors`: `content_hash` derivado em `.idx/anchors.jsonl` e verify-on-hit —
    `cited` invalida, `context` não; stale **sinaliza**, nunca apaga (D86).
  - `lifecycle::confidence`: confiança **derivada** (`sim × drift × idade + feedback`, pisos,
    `[0,1]`) com proptest de monotonicidade; exposta em `RecallHit.confidence` (D87).
  - Correções de contrato: `task::outcome` usa `notes`/`recorded_at` (D48) e `task` exporta
    `validate_transition`.
- **E08 — Prime, handoff, diff e learn** (MVP, concluído):
  - `handoff::rewind`: família de estado/handoff — manifest (~30 tokens), escopo (container) e
    working set (âncoras), com ranking por trust-tier
    (`star*100 + foundational*50 + tactical*20 + observational*10` — D57).
  - `handoff::budget`: orçamento sem tokenizer (`ceil(chars/4)`, default 4000), truncando o
    último item e ignorando sobra < 100 tokens (D40/D82); `apply_into` escreve direto no destino
    (streaming) sem montar saída gigante.
  - `handoff::scope`: auto-context-scope a partir dos arquivos tocados e auto-flip
    (`>100 notas` ou `>5 containers` — D41).
  - `handoff::context`: `context_id` derivado e guardado em `.idx/contexts/`; `--resume` devolve
    **bytes idênticos** (D88).
  - `maintenance::diff`: passado derivado da auditoria de eventos por intervalo/escopo (D21/D33).
  - `maintenance::learn`: propostas determinísticas (`create_note`/`merge`/`supersede`/`link`) a
    partir de eventos + âncoras — nunca escreve (D33/D47).
  - `maintenance::compact`: propõe `concat`/`keep_latest`/`merge_outcomes`; aplica só sob aceite,
    fundindo no `keep` e esquecendo (soft) os demais (D47).
  - `task`: hierarquia fechada `plan ⊃ epic ⊃ issue ⊃ task` (máx. 4), `plan`/`epic` como
    `container` sem verdade própria, pai por marcador no corpo + aresta `results_in`, `blocks`
    1-based; ciclo de vida `adopt`/`release`/`review`, `outcome` e `reorder` (D52/D53/D93).
- **E07 — Escrita e protocolo** (MVP, concluído):
  - `write::Draft`: rascunho tipado que vira frontmatter válido; opcionais vazios **omitidos**
    (D05); `scope` só para `task`/`container` (D93).
  - `write::dedup`: decisão em três faixas (`<0.75` cria, `0.75–0.92` merge, `≥0.92` rejeita —
    D26), similaridade **Dice** sobre termos em `[0,1]` calibrada para os limiares, configurável
    por `[dedup]` (D80).
  - `write::write`: idempotente por conteúdo (D01) — retry devolve o mesmo id sem duplicar;
    `task`/`container` rejeitados; merge funde tags/âncoras/corpo e incrementa `revision`;
    rejeição não escreve.
  - `write::update`: `Patch` versionado; mesma chave → edita no lugar; `type`/`statement` novos
    → **supersede** (novo id + `replaces`/`superseded_by` — D01/D48); `history` caminha a cadeia.
  - `write::lifecycle`: `forget`/`restore` soft (`status`), `link` de arestas explícitas com
    ponteiro reverso em `replaces` (D46/D49/D52); transições protegidas.
  - `write::propose_merges`: reconciliação só **propõe** quase-duplicados — nunca funde em
    silêncio (D47/D80).
- **E06 — Retrieval: BM25, âncoras e RRF** (MVP, concluído):
  - `retrieval::token`: tokenização **ASCII explícita** `[a-z0-9_]` (`café` → `caf` — D36),
    com `Cow` no caminho quente e termos de consulta deduplicados.
  - `retrieval::index`: índice derivado `.idx/retrieval.jsonl` (uma linha JSON por nota) com
    frequências por campo (`statement`/`body`/`tags`); reconstruível **byte a byte**; ausente →
    reconstrói e grava; teto de tamanho emite aviso (D15/D27).
  - `retrieval::bm25`: BM25 (`k1=1.5`, `b=0.75`) com **IDF por campo**, **peso por tipo** e
    **boost por confirmação** `1 + 0.1*(success + partial*0.5)` (D35/D37/D38).
  - `retrieval::anchor`: canal de âncoras por `path`/`id` com globs `?`/`*`/`**` (D81/D86).
  - `retrieval::rrf`: fusão `1/(k+rank+1)` com `k=60` e desempate `(score desc, id asc)` (D81).
  - `retrieval::filter`: filtros determinísticos (`type`/`classification`/`status`/`tags`/`anchors`)
    aplicados antes do BM25; `container` resolvido no grafo via `depends_on` transitivo (D41).
  - `retrieval::views`: `ready`/`blocked` computadas do `depends_on` transitivo; ciclo de
    dependência = `blocked` (D53).
  - `retrieval::why` + contrato `id|statement|score|why` (4ª coluna com conjunto fechado — D39);
    `get(ids)` devolve corpo só dos ids pedidos.
  - Degradação graciosa: canal falho retorna resultado parcial + `warnings`; `strict` (D94)
    promove a erro (E06-T07).
- **E05 — Grafo e arestas** (Fase 0, concluído):
  - `schema::edge`: enum fechado `EdgeKind` (8 valores) com chaves de frontmatter em
    `snake_case`, `Edge` e `EDGE_KEYS` (D49/D51).
  - `schema::frontmatter`: as 8 chaves de aresta entram na **ordem canônica** (após
    `superseded_by`, antes de `revision` — **27 chaves**, D98), com `string_list`, `edges()` e
    validação de formato dos ids.
  - `graph`: projeção das notas (`Graph`), `link()` idempotente (rejeita id inválido e
    auto-aresta) e `expand` BFS determinística que só percorre o **explícito** (D49).
  - `graph::integrity`: arestas penduradas, auto-arestas e bidirecionalidade
    `replaces ↔ superseded_by` (D46).
  - `graph::cycles`: SCC (Kosaraju iterativo, sem dependências) para supersessão (`replaces`)
    e dependência (`depends_on`); `cycle_members()` protege os membros de demolição (D45).
  - `graph::extract` + `graph::suggestions`: extração conservadora (ids, wikilinks, verbos) com
    armazenamento derivado em `.idx/suggestions.jsonl` — separado e auditável (D49/D50).
  - `store::purge` passou a podar ids dentro de listas JSONL (não só descartar a linha), para
    limpar `targets` de sugestões de um alvo removido (D84).
  - Decisão **D98** registrada.
- **E04 — Config, Git e worktree** (Fase 0, concluído):
  - `config`: config em dois níveis (global template + projeto com precedência — D61), schema
    fechado com tipos/enums/defaults, merge profundo, `set/unset/list/get` com validação (D64),
    poda de ancestrais vazios e sanitização de segredos (D91).
  - `config/toml`: codec TOML próprio (subset) com leitura que **preserva ordem** (diff mínimo)
    e escrita canônica (D63/D97); comentários, seções, chaves pontilhadas, strings de uma
    linha, números, listas multilinha; rejeita `[[...]]`/multilinha/`null`.
  - `git::project`: resolve o **worktree principal** (`--git-common-dir`), compartilha `.knudge/`
    entre worktrees ligados, usa o top-level do **submódulo** e valida o nome lógico (D29/D91).
  - `git::exclude`: exclusão idempotente via `.git/info/exclude` (nunca `.gitignore` — D30),
    versionando `notas/`/`eventos/` e excluindo só o derivado, com reversão ao alternar o modo
    (D34); no-op fora de repo.
  - `git::attributes` + `git::block`: bloco `merge=union` para `eventos*.jsonl` em
    `.gitattributes`, delimitado por marcadores e idempotente (D31/D60).
  - `git::agent_md`: gera/atualiza o `AGENTS.md` do projeto-alvo com version marker, sem
    duplicar nem sobrescrever o conteúdo do usuário (D57/D60).
  - `git::onboard`: cria a árvore `.knudge/`, clona o global **literalmente** (ou usa defaults),
    aplica exclude/attributes/AGENTS e é idempotente (D62).
  - `git::sync`: guard de worktree com `git -C <raiz>`, commit de `notas/`+`eventos/` e mensagem
    gerada do último evento (D32).
  - Porta `Git` ampliada (`common_dir`, `top_level`, `superproject_root`, `run`) e adaptador
    `StdGit` sem shell (R12); `MemFs` passou a modelar diretórios explícitos.
  - Decisão **D97** registrada.
- **E03 — Store, notas e eventos** (Fase 0, concluído):
  - `jsonl`: codec JSON próprio (`encode`/`decode` canônicos, chaves ordenadas e sem
    dependência externa) e leitor de linhas tolerante a CRLF/linhas em branco.
  - `store`: `Note` (frontmatter + corpo) com render/parse byte-exato, `Store` (write atômico,
    list, read, `update` com `revision`, `remove`) e ordem de commit **nota → evento** (D21).
  - `store/events`: `Event` com `id` derivado do conteúdo (`evt_<base36(8)>`), `EventLog`
    append-only com **dedup on-read** (D26/D28), tolerância a linha malformada, rotação por
    tamanho (`events-NNNN.jsonl`), checkpoint derivado (`.idx/events.checkpoint`) e `history`.
  - `store/lock`: lock advisory por arquivo-alvo (`create_exclusive`), stale 30 s, reclaim por
    rename sidecar e liberação RAII (D23–D25/R05).
  - `store/rebuild`: `Staging` double-buffer (`.idx.new/` + rename atômico — D27).
  - `store/purge`: `purge_derived` remove o id de `*.jsonl` e `index.json` em toda remoção
    (D84), usado por `Store::remove`.
  - `store/sweep`: varredura de resíduos `*.tmp`/`*.lock`/`*.stale` por idade, com `warn`
    (R10), **sem** remover lock fresco de processo vivo.
  - Porta `Fs` estendida com `append`, `create_exclusive`, `rename`, `sync`, `modified_ms`,
    `is_dir` e `remove_dir_all`; `MemFs` virou um FS fiel (tmp+rename, diretórios implícitos) e
    `FaultyFs` injeta falhas para testes de crash.
  - `ARCHITECTURE.md` §5 documenta a persistência; decisão **D96** registrada.
- **E02 — Contrato de bytes: TOON, schema e IDs** (Fase 0, concluído):
  - `schema/types`: enums fechados `NoteType` (11), `Scope`, `Classification`, `Status`.
  - `schema/hash`: hash curto `SHA-256 → u32`, `hex8` e `base36(8)` (D95).
  - `schema/body`: `normalize` (NFC + trim + colapso) e `body_hash` (D06).
  - `schema/id`: `id` endereçado por conteúdo (`type + U+001F + statement`) e validador (D01–D03).
  - `schema/text`: contagem de `statement` em escalares Unicode, limite 120 (D08).
  - `schema/frontmatter`: ordem canônica das 19 chaves, omissão de opcionais (D05), leitura
    tolerante com warning (D16) e validação (T01/T07).
  - `toon`: parser/emissor próprios (`lex`/`flow`/`parse`/`emit`), round-trip byte-exato,
    comentários, escapes, zero bytes, `1.0 → 1` e detecção de `schema_version` (D74/D75).
  - `TOON.md` publica a gramática e as regras de bytes.
- **E01 — Fundação** (Fase 0, concluído):
  - Workspace Cargo (`knudge-core`, `knudge-cli`, `knudge-mcp`) com `edition = "2024"`,
    MSRV `1.97`, `[workspace.lints]` (R44), perfil de release (R41) e `make check` (E01-T01/T03).
  - Portas determinísticas (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`),
    adaptadores `std` e fakes (`FixedClock`, `SeqRng`, `FakeEnv`, `FakeGit`, `MemFs`,
    `RecordingLogger`) (E01-T02).
  - Modelo de erro `Error`/`ErrorKind` com mapa código→exit, `thiserror`, poison via
    `into_inner` e contexto de I/O com path (E01-T06; R30–R35).
  - `Timestamp` UTC com milissegundos, formatação/parsing próprios e round-trip por proptest
    (E01-T02; D07).
  - Redação de segredos no layer de log (allowlist + chaves sensíveis) (E01-T07; R22).
  - Binário `kd` com a **superfície v2** (E01-T04; `16_cli_surface.md`): `kd` = `prime`,
    `--json` (`{success, command, data?, error?, warnings?}`), exit codes estáveis e
    `EPIPE → exit 0`.
  - Política de memória: `#![forbid(unsafe_code)]`, rejeição de symlink em `.knudge/` (E01-T08).
  - `ARCHITECTURE.md`, `MODULE.md` por crate e módulos temáticos de `knudge-core` (E01-T05).

### Decidido (superfície CLI v2)
- **D57** reescrita: `prime` é protocolo estático byte-idêntico; estado/handoff → `rewind`.
- **D88** ajustada: `rewind` emite `context_id`; `kd rewind --resume <id>`.
- **D93**: `task` com `scope` fechado (`plan|epic|issue|task`), hierarquia máx. 4.
- **D94**: `strict` é config de projeto (`[behavior] strict`), sem flag.
- **D95**: hash curto `SHA-256 → u32`; chave/derivação de `id` e gramática TOON v1 (`TOON.md`).
- **D96**: registro de evento (`id` derivado + dedup on-read), rotação por tamanho e
  semântica de `revision`.
- **D97**: subset TOML (ordem preservada/canônica), guard `git -C` do `sync` e degradação do
  `onboard` sem config global.
- **D98**: arestas como chaves de frontmatter (27 chaves na ordem canônica), ponteiro reverso
  `superseded_by`, ciclo de supersessão sobre `replaces` e sugestões derivadas.

### Testes
- 397 testes de unidade no `knudge-core` (erro, tempo/proptest, redação, fakes, symlink,
  schema/hash/ID/arestas, TOON, JSONL/JSON, store, config/TOML, git/onboard/sync,
  grafo/integridade/ciclos/sugestões, retrieval/token/BM25/âncoras/RRF/views,
  escrita/dedup/update/supersede/forget, rewind/orçamento/context_id, diff/learn/compact,
  tarefas/hierarquia/ciclo de vida, validators/evidência/audit/doctor/âncoras/confiança,
  shelf-life/decay/purga/ciclos/clusters, embeddings/meta/vector/cache/índice/fila/eval/http,
  hooks/timeout/kill de grupo, lock/rebuild) e 3 testes de **stress** sobre adaptadores reais.
- 37 testes do servidor MCP (`knudge-mcp`: gatilhos, codec JSON-RPC, handshake, tools e
  transporte) + 2 de integração stdio (`tests/stdio.rs`).
- 12 testes de integração do binário + 7 **golden** (`--help`, `kd == kd prime`, `--json`, exit
  codes, EPIPE, comando desconhecido, `init`+`write`+`ask`, `task`, `config`, `forget`).
