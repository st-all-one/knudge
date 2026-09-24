# E11 — Embeddings

> **Fase 3.** Vetores são **derivados e opcionais**, consumidos de forma **assíncrona e lazy**.
> O projeto **nunca** trava por causa do modelo: notas recém-criadas ficam “dark” no espaço
> vetorial até serem digeridas — gap tolerado. Falha marca `pending`, nunca descarta.
>
> **Decisões:** D42, D79, D80, D83, D84, D85, D89, D90, D101. Detalhes em `../04_embeddings.md`.
> **Políticas:** R11, R12, R14, R16, R32 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Um provedor de vetores plugável, barato de manter e impossível de derrubar o restante do
sistema — com a escolha de modelo decidida por **medição no corpus**, não por chute.

## Pré-requisitos

E06, E07.

## Tarefas

### E11-T01 ☑ Provedor plugável
- **Objetivo:** `config [embeddings]` com `provider = http|lightweight|none`; default
  **`ibm-granite/granite-embedding-97m-multilingual-r2`** (384d, cosseno, multilíngue — D123),
  `revision` pinada,
  `dimensions`/`similarity` no cabeçalho `meta` do `.idx/embeddings.jsonl`.
- **Entregáveis:** trait `Embedder` (port); impls `http` (OpenAI-compatible — o usuário sobe um
  `llama-server`/TEI/Ollama com o GGUF), `lightweight` (hash, D89) e `none` (BM25 puro); meta do
  índice e invalidação por modelo.
- **Decisões:** D42, D79, D101, D123.
- **Aceite:** trocar modelo/revisão **invalida** o índice (força re-embed); `none` cai para
  BM25 sem erro. (A inferência `local` in-process foi recusada — D101/R16/R43.)

### E11-T02 ☑ Cache por `body_hash` com teto
- **Objetivo:** `.idx/emb_cache` (hash→vetor); hit **pula inferência**; lote com hits/misses
  mistos; falha de cache degrada para pass-through.
- **Entregáveis:** cache; wrapper que consulta antes de inferir; `max_bytes`/TTL com **eviction
  LRU**; `doctor` reporta o tamanho do cache.
- **Decisões:** D83. **Políticas:** R14.
- **Aceite:** re-embed do mesmo conteúdo não chama o modelo; cache respeita o teto; falha de
  cache não é fatal.

### E11-T03 ☑ Fila assíncrona/lazy + auto-drain ocioso
- **Objetivo:** `mode = lazy|manual` (D131), `async = true`, `max_pending`; estado derivado
  `indexed|pending|stale` por nota; **nunca descarta nota**; `rewind` reporta
  `embeddings_pending`.
- **Entregáveis:** fila; estados; contador no `rewind`; **auto-drain ocioso** — com `mode=lazy`,
  ao fim de cada invocação não-`maintenance` o CLI drena **um lote** *best-effort*, **depois** de
  emitir a saída; falha não muda exit code nem `warnings[]`; `KNUDGE_NO_IDLE` desliga;
  `kd maintenance watch-service` gerencia o worker contínuo (timer systemd) com consentimento:
  `--install`/`--subscribe`/`--unsubscribe`/`--status`/`--uninstall`, multi-projeto (D132).
- **Decisões:** D80, D83, D131.
- **Aceite:** rajada de 10–20 notas não bloqueia `write`/`recall`; backlog visível e
  drenável; `lazy` esvazia a fila durante o uso, `manual` só com `--drain`.

### E11-T04 ☑ Worker de reconcile
- **Objetivo:** re-embedar do **corpo canônico** as pendentes; I/O externo com **timeout
  curto** e falha rápida → `pending`.
- **Entregáveis:** worker (chamado pelo daemon/CLI ocioso/`kd maintenance index`).
- **Decisões:** D80, D83.
- **Aceite:** endpoint lento não trava; pendentes viram `indexed` depois; nenhuma nota perdida.

### E11-T05 ☑ Purga do vetor na remoção
- **Objetivo:** toda remoção (supersede/merge/`compact`/TTL/dedup) usa o caminho único de E03-T07.
- **Entregáveis:** integração com o derivado.
- **Decisões:** D84.
- **Aceite:** após remoção, o índice vetorial não contém o id; `doctor` confirma.

### E11-T06 ☑ Flush coalescido
- **Objetivo:** debounce (`flush_ms = 2000`) com *dirty flag*; **flush forçado na saída**.
- **Entregáveis:** flush agendado; flush no shutdown.
- **Decisões:** D85.
- **Aceite:** rajada gera um único write O(N); matar o processo após flush preserva o índice.

### E11-T07 ☒ `eval --ab` removido (D145)
- **Objetivo original:** comparar dois modelos com **Recall@k, nDCG@k, MRR** sobre um golden pequeno.
- **Decisão (v0.3.0):** o `maintenance eval` era **stub** e o módulo puro `embeddings/eval.rs` só
  era usado em testes; ambos foram **removidos** (D145). A avaliação de modelo segue na bancada
  **externa `bench/`** (foi como o `granite` foi escolhido — D123). A fila de embeddings virou
  `kd knowledge digest` (ex-`maintenance index`).
- **Aceite:** `kd maintenance eval` → exit 2; nenhum re-export de `eval` quebrado.

### E11-T08 ☑ `lightweight` para testes/offline
- **Objetivo:** embedder determinístico por hash (SHA-256 → vetor normalizado), sem pesos.
- **Entregáveis:** impl `lightweight`; uso em CI.
- **Decisões:** D89.
- **Aceite:** testes rodam sem download e sem rede; determinístico.

### E11-T09 ☑ Dedup semântico e descoberta de links
- **Objetivo:** alimentar o dedup eventual (E07-T06) e sugerir links não-declarados, sempre
  como **proposta**.
- **Entregáveis:** consultas ao espaço vetorial; integração com `audit`/`compact`.
- **Decisões:** D42, D47.
- **Aceite:** sugestões revisáveis; nada é fundido/ligado sem aceite.

### E11-T10 ☑ Runtime mínimo, backpressure e timeouts
- **Objetivo:** a digestão não introduz runtime pesado nem trava o comando.
- **Entregáveis:** worker bloqueante + **canal bounded** (`max_pending` como backpressure); pool
  limitado a `available_parallelism()`; inferência local em thread dedicada/`spawn_blocking`;
  timeout tipado e retry/backoff só em HTTP idempotente; nenhum lock atravessa `.await`
  (cancellation safety); poison tratado com `into_inner` (R32).
- **Decisões:** D79, D80. **Políticas:** R11, R12, R16, R32.
- **Aceite:** `cargo tree` sem `tokio full`; endpoint lento não trava; rajada acima de
  `max_pending` não estoura memória.

## Definition of Done

- [x] Nenhum caminho do sistema bloqueia por embedding.
- [x] Cache, fila, reconcile, purge e flush travados por teste.
- [x] `kd knowledge digest` disponível para digerir a fila; avaliação de modelo é offline (`bench/`, D145).
- [x] Runtime mínimo e backpressure demonstrados.

## Não-objetivos

- Clusters semânticos (E10-T07) — consome este épico.
- Modelo fixo embutido (recusado: plugável com default pinado).
