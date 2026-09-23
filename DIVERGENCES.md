# DIVERGENCES — bordas do knudge

> O knudge não tem um produto de referência para *harness* diferencial. O papel equivalente é
> este catálogo: cada **borda** onde a implementação poderia divergir de si mesma (bytes,
> ordem, tempo) tem uma **mitigação** e o **teste que a trava** (D77, E13-T05).
>
> Regra: toda mudança que toque uma linha abaixo precisa continuar verde no teste citado — e
> atualizar o golden correspondente se for intencional.

| # | Borda | Risco | Mitigação | Teste que trava |
|---|---|---|---|---|
| 1 | **Unicode / NFC** | duas grafias equivalentes gerarem ids/hashes distintos | `normalize` = NFC + trim + colapso de whitespace, alimentando `id` e `body_hash` (D06/D95) | `schema::tests::normalize_is_idempotent`, `note_id_ignores_surrounding_whitespace`, `body_hash_ignores_surrounding_whitespace` |
| 2 | **Contagem de escalares** | `statement` longo ser medido em bytes, não em caracteres | `count_scalars` conta *chars* Unicode, não bytes | `schema::tests::*` (validação de statement) |
| 3 | **Hash do corpo** | `body_hash` divergir entre plataformas | SHA-256 sobre `normalize(statement)+LF+normalize(body)`, hex8 minúsculo (D06) | `schema::tests::body_hash_*`, `health::tests::doctor` (`body_hash` em dia) |
| 4 | **IDs** | id mudar ao reclassificar `type` | id = `<prefixo>_<base36(8)>` endereçado por `type + U+001F + normalize(statement)`; prefixo é **histórico** (D01/D95) | `schema::tests::note_id_is_deterministic`, `note_id_ignores_surrounding_whitespace` |
| 5 | **Ordem de chaves (TOON)** | frontmatter reordenar e quebrar diffs/bytes | ordem canônica das 28 chaves (`CANONICAL_KEYS`); opcionais omitidos, nunca `null` (D98) | `toon::tests::*` (round-trip byte-exato), `schema::tests` |
| 6 | **TOON round-trip** | parser/emissor não fecharem | gramática linha-a-linha; round-trip byte-exato | `toon::tests::round_trip_is_byte_exact`, proptest `emit_parse_round_trips_*` |
| 7 | **Tie-break RRF** | empate de score produzir ordem instável | desempate determinístico `(score desc, id asc)` (D81) | `retrieval::tests::rrf::fuse_is_deterministic`, `fuse_contains_union_of_ids` |
| 8 | **Timestamps** | usar relógio global e quebrar reprodutibilidade | porta `Clock` (`Timestamp` UTC ms); core nunca chama `SystemTime::now` (D07/R03) | `time` proptest + fakes `FixedClock` em todo o core |
| 9 | **Atomicidade da escrita** | leitor ver arquivo pela metade | tmp + `rename` no mesmo diretório, `fsync` antes (D20/D22) | `store::tests::commit::crashed_atomic_write_keeps_old_file` |
| 10 | **Ordem nota/evento** | evento apontando para nota inexistente | **nota antes do evento**; crash deixa nota sem evento (recuperável) (D21) | `store::tests::commit::crash_between_note_and_event_leaves_note_without_event` |
| 11 | **Rebuild double-buffer** | leitor ver índice parcial | escreve `.idx.new/` e troca por `rename`; crash preserva o antigo (D27) | `store::tests::rebuild::reader_sees_old_or_new_never_partial`, `crash_during_rebuild_keeps_old_index`, `tests/stress.rs::reader_never_sees_partial_index` |
| 12 | **Lock advisory** | dois escritores no mesmo alvo → *lost update*; lock recém-criado roubado | `create_exclusive` + reclaim por `rename`; **lock sem conteúdo legível usa `mtime`** e, sem `mtime`, não é reclamado (E13-T03) | `store::tests::lock::*`, `tests/stress.rs::lock_prevents_lost_updates_on_real_fs` |
| 13 | **`StdFs::rename` ausente** | `NotFound` virar `Io` e abortar o reclaim | `StdFs::rename` mapeia `NotFound` para `ErrorKind::NotFound` | `tests/stress.rs::lock_prevents_lost_updates_on_real_fs` |
| 14 | **Rotação de eventos** | crescimento ilimitado do JSONL | ativo → `events-NNNN.jsonl` acima de `max_bytes` (R13) | `store::tests::events` |
| 15 | **Determinismo de iteração** | `HashMap`/`HashSet` variarem a ordem | proibidos no core (BTree/IndexMap) via `clippy.toml` (R03) | `clippy -D warnings` no gate |
| 16 | **Config TOML** | defaults/override divergirem entre níveis | `Config::effective` = defaults + global + projeto; segredos só no global (D61/D91) | `config::tests`, `config::toml::tests` |
| 17 | **Saída de máquina** | log vazar para stdout e corromper `--json` | stdout = dados, stderr = logs; redação no layer (R20–R23) | `tests/golden.rs`, `tests/cli.rs` |
| 18 | **Exit codes** | tradução de erro mudar silenciosamente | `ErrorKind::code()`/`exit_code()` congelados; 101 reservado a panic (R35) | `tests/golden.rs`, `tests/cli.rs::unknown_command_exits_two` |
| 19 | **EPIPE** | pipe fechado derrubar o processo | `output::emit_stdout` trata `BrokenPipe` → exit 0 (D73) | `tests/golden.rs::epipe_is_exit_zero`, `tests/cli.rs::broken_pipe_exits_zero` |
| 20 | **Embeddings** | índice servir vetores de outro modelo | cabeçalho `meta` (provider/model/revision/dimensões) invalida ao mudar (D79) | `embeddings::tests::meta`, `index` |
| 21 | **Framing MCP** | delimitador de mensagem divergir entre cliente e servidor | JSON-RPC 2.0 **uma linha por mensagem** (sem `Content-Length`); parse inválido responde com `id: null`; `EPIPE`/EOF → exit 0 (E14/D71/D73) | `tests::transport::*`, `tests/stdio.rs::handshake_and_tools_over_stdio` |
| 22 | **Glob de âncora bidirecional** | `ask --anchor` voltar vazio sem query textual e ignorar âncoras-glob | `--anchor` é **repetível e aceita vírgula**, alimenta o **canal** de âncoras (D81) e `Filter.anchors` casa nas **duas direções** (pedido↔âncora) | `retrieval::tests::filter::anchor_filter_accepts_note_glob_matching_requested_path`, `retrieval::tests::recall::anchor_channel_recalls_with_empty_text`, `cli::ask_anchor_finds_note_without_query`, `cli::ask_anchor_accepts_comma_separated_and_repeated` |
| 23 | **Ordem da árvore do programa** | `task graph`/`rewind --files` divergirem entre execuções | `subtree` em pré-ordem com filhos `id asc`; `program_roots` ordenados (D119) | `task::tests::program::subtree_is_deterministic_preorder` |
| 24 | **Programa ↔ Épico-raiz** | épico sem programa ou programa órfão passarem batido | check `program-anchor` (warn): **menor id** no match; `programs.glob` define o programa (D119) | `health::tests::doctor::program_anchor_reports_epic_without_program`, `health::tests::doctor::program_anchor_reports_orphan_program` |
| 25 | **Motivo de bloqueio** | `--explain` escolher dependência não-determinística | **menor id** pendente na travessia transitiva (`blocked_by`); ciclo e `not_before` têm precedência (D104) | `retrieval::tests::views::block_reason_reports_smallest_pending_dependency` |
| 26 | **Canal vetorial vs filtros** | o `ask` semântico furar `--type`/`--status`/`--anchor` | o canal vetorial é intersectado com `allowed` em `recall` (D102) | `retrieval::tests::recall::vector_channel_respects_deterministic_filters` |
| 27 | **Espécie × nível** | `--kind` reescrever `scope` ou exigir `scope` para conhecimento | `scope` é nível, `type` é espécie; `scope` é exigido só para `task`/`container`, opcional nos demais itens de trabalho (D113) | `task::tests::kind::containers_only_accept_container_kind` |
| 28 | **Dono derivado** | `claim` virar campo e conflitar sob merge | dono é projeção de eventos (`op=claim`/`release`), dedup por `id` (D96/D114) | `task::tests::ownership::ownership_follows_last_claim_not_released`, `task::tests::ownership::close_clears_ownership` |

## Notas

- **`prime` é estático** (D57): não depende de estado nem de relógio — por isso é byte-idêntico.
  Qualquer prosa nova ali quebra `tests/golden/prime.txt` de propósito.
- **Testes de proptest** rodam com contagem fixa de casos (`with_cases(...)`) e seed padrão do
  `proptest`; regressões encontradas são salvas em `crates/knudge-core/proptest-regressions/`.
- **Concorrência**: o core não tem primitivos em memória compartilhada além de `Arc<Mutex<_>>`
  nos fakes; a sincronização real é por **arquivo** (lock advisory). Por isso não há alvo de
  `loom` — o stress é feito sobre o adaptador real (`tests/stress.rs`).
