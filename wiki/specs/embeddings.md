# Embeddings

Vetores são **derivados e opcionais** (D42/D79). O sistema **nunca bloqueia** por embedding: notas
recém-criadas ficam *dark* até a fila derivada drená-las, e o provedor é plugável via config.

- Código: `crates/knudge-core/src/embeddings/`
- Decisões: D42, D79–D85, D89, D101, D102, D123, D131–D133, D148, D153, D158, D182, D183, D202

## Provedor plugável (D79/D101/D202)

`embeddings.provider` ∈ `http` (default) | `lightweight` | `none`:

- **`http`**: servidor local **OpenAI-compatible** (`llama-server … --embeddings`, TEI/Ollama/vLLM)
  via `embeddings.endpoint` (default `http://127.0.0.1:8889/v1/embeddings` — D202), com
  `timeout_ms`, `retries` e `api_key_env`. Cliente **HTTP/1.1 bloqueante sobre `std::net`**
  (sem `tokio`/`reqwest` — R16/R43); `https://` exige proxy/TLS terminator. **Sem inferência
  in-process** (R16/R43).
- **`lightweight`**: embedder determinístico por hash (D89) — CI/offline, sem rede.
- **`none`**: cai para BM25 puro.

Modelo default: `ibm-granite/granite-embedding-97m-multilingual-r2` (384d, D123). `revision`
pinada; o índice é **invalidado** quando modelo/revisão/dimensão mudam (D79).

## Identidade e álgebra

- `meta.rs`: `EmbeddingMeta { provider, model, revision, dimensions, similarity }` + `Similarity`
  (cosseno/dot). `from_config` lê `embeddings.*`.
- `vector.rs`: `cosine`, `dot`, `normalize`, `is_normalized`, `l2_norm`, `similarity`.

## Índice (D80/D84)

- `index.rs`: `EmbeddingIndex` (`.idx/embeddings.jsonl`) com `IndexedVector` (id + vetor + meta).
- **Purga** (D84): toda remoção de nota descarta o vetor (`purge_derived`); o `doctor` detecta
  divergência canônico↔derivado e orienta rebuild.

## Cache (D83/D148/D153)

- `cache/`: cache por **`(body_hash, modelo)`** com teto e eviction **LRU**.
- **Versionável** (opt-in `embeddings.version_cache`, D148): mora em `.knudge/emb_cache.jsonl`
  (fora do `.idx/`), com `merge=union` (D31) + dedup e **sem eviction** quando versionado.
- Chave lógica `(body_hash, model)`, loader **idempotente**, **model-aware** e com desempate
  determinístico (`created_ms`) — D153 (multi-dev).
- Falha de embedding marca `pending`, **nunca descarta a nota** (D83).

## Estado e modo (D80/D131)

- `state.rs`: `EmbeddingState` ∈ `Indexed`/`Pending`/`Stale` (derivado, em `.idx/`);
  `is_backlogged`; `classify`.
- `EmbeddingMode` ∈ `lazy` (default) | `manual`. `eager` é rejeitado (config = 7). Em `lazy`, ao
  fim de cada invocação não-`maintenance` o CLI drena **um lote** *best-effort* (`idle::maybe_drain`,
  D131), **depois** de emitir a saída — nunca altera exit code nem `warnings[]`.

## Dreno e reconciliação (D80/D85/D182)

- `pipeline.rs::drain` consome a fila e reconcilia: cache → provedor → índice. `DrainInput`/
  `DrainOutcome`.
- `vectors.rs` resolve vetores do dreno (cache, lote e isolamento).
- `flush.rs`: flush **coalescido** (debounce) do `.idx`/embeddings com *dirty flag*, e flush
  forçado na saída — rajadas de 10–20 notas causam um único rewrite (D85).

## Consultas semânticas (D102/D158)

- `semantic.rs`: vizinhos, duplicatas e links sobre o índice vetorial (`rank_query`).
- `suggest.rs` (D158): classifica pares em três relações **fechadas** — `duplicate` (score ≥
  `dedup.merge_below`), `link` (score ≥ `suggestions.contradiction_high`, ou na banda
  `contradiction_low..high` **com** âncora compartilhada) e `contradiction` (na banda **sem**
  âncora compartilhada). Pares já ligados nunca aparecem; `--relation` fora do enum é erro
  (exit 2). **Advisory** — persiste em `.idx/suggestions.jsonl` (D50), nunca vira aresta (D49),
  purgado (D84). Determinístico, zero-LLM.

## Worker e supply-chain (D131–D133/D182/D183/D186)

O worker contínuo fica **fora do binário**, em `scripts/knudge-idle.sh` (fonte da verdade);
`kd drain service` é wrapper fino (D186). O agendador é detectado em runtime (`systemd --user`/
`launchd`; senão, cron); o servidor llama.cpp vira unidade própria (D133). O GGUF vem de **revisão
pinada** + **SHA-256** e o instalador do llama.cpp é verificado antes de executar (D183).
`--reconcile` alinha `endpoint`/`model` e reindexa (D182).

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Identidade/provedor | `embeddings/meta.rs` |
| Álgebra | `embeddings/vector.rs` |
| Índice | `embeddings/index.rs` |
| Cache | `embeddings/cache/` |
| Dreno/reconcile | `embeddings/{pipeline,vectors,flush}.rs` |
| Estado/modo | `embeddings/state.rs` |
| Semântica/sugestão | `embeddings/{semantic,suggest}.rs` |
| Embedder de teste | `embeddings/lightweight.rs` |

## Testes

`embeddings/tests/` (index, cache, pipeline, state, semantic, vector, flush, meta, lightweight,
versioned). `DIVERGENCES.md` #98/#106.
