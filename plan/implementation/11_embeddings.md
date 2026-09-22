# E11 — Embeddings

> **Fase 3.** Vetores são **derivados e opcionais**, consumidos de forma **assíncrona e lazy**.
> O projeto **nunca** trava por causa do modelo: notas recém-criadas ficam “dark” no espaço
> vetorial até serem digeridas — gap tolerado. Falha marca `pending`, nunca descarta.
>
> **Decisões:** D42, D79, D80, D83, D84, D85, D89, D90. Detalhes em `../04_embeddings.md`.
> **Políticas:** R11, R12, R14, R16, R32 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Um provedor de vetores plugável, barato de manter e impossível de derrubar o restante do
sistema — com a escolha de modelo decidida por **medição no corpus**, não por chute.

## Pré-requisitos

E06, E07.

## Tarefas

### E11-T01 ☐ Provedor plugável
- **Objetivo:** `config [embeddings]` com `provider = local|http|lightweight|none`; default
  **`sentence-transformers/msmarco-MiniLM-L12-cos-v5`** (384d, cosseno), `revision` pinada,
  `dimensions`/`similarity` no `.idx/meta`.
- **Entregáveis:** trait `Embedder` (port); impls `local` (ONNX/candle), `http`
  (OpenAI-compatible), `lightweight`, `none`; meta do índice.
- **Decisões:** D42, D79.
- **Aceite:** trocar modelo/revisão **invalida** o índice (força re-embed); `none` cai para
  BM25 sem erro.

### E11-T02 ☐ Cache por `body_hash` com teto
- **Objetivo:** `.idx/emb_cache` (hash→vetor); hit **pula inferência**; lote com hits/misses
  mistos; falha de cache degrada para pass-through.
- **Entregáveis:** cache; wrapper que consulta antes de inferir; `max_bytes`/TTL com **eviction
  LRU**; `doctor` reporta o tamanho do cache.
- **Decisões:** D83. **Políticas:** R14.
- **Aceite:** re-embed do mesmo conteúdo não chama o modelo; cache respeita o teto; falha de
  cache não é fatal.

### E11-T03 ☐ Fila assíncrona/lazy
- **Objetivo:** `mode = lazy|eager|manual`, `async = true`, `max_pending`; estado derivado
  `indexed|pending|stale` por nota; **nunca descarta nota**; `prime` reporta
  `embeddings_pending`.
- **Entregáveis:** fila; estados; contador no `prime`.
- **Decisões:** D80, D83.
- **Aceite:** rajada de 10–20 notas não bloqueia `write`/`recall`; backlog visível e
  drenável.

### E11-T04 ☐ Worker de reconcile
- **Objetivo:** re-embedar do **corpo canônico** as pendentes; I/O externo com **timeout
  curto** e falha rápida → `pending`.
- **Entregáveis:** worker (chamado pelo daemon/CLI ocioso/`kd embed`).
- **Decisões:** D80, D83.
- **Aceite:** endpoint lento não trava; pendentes viram `indexed` depois; nenhuma nota perdida.

### E11-T05 ☐ Purga do vetor na remoção
- **Objetivo:** toda remoção (supersede/merge/`compact`/TTL/dedup) usa o caminho único de E03-T07.
- **Entregáveis:** integração com o derivado.
- **Decisões:** D84.
- **Aceite:** após remoção, o índice vetorial não contém o id; `doctor` confirma.

### E11-T06 ☐ Flush coalescido
- **Objetivo:** debounce (`flush_ms = 2000`) com *dirty flag*; **flush forçado na saída**.
- **Entregáveis:** flush agendado; flush no shutdown.
- **Decisões:** D85.
- **Aceite:** rajada gera um único write O(N); matar o processo após flush preserva o índice.

### E11-T07 ☐ `kd eval --ab`
- **Objetivo:** comparar dois modelos com **Recall@k, nDCG@k, MRR** sobre um golden pequeno;
  decide L6 vs L12 vs multilíngue no corpus real.
- **Entregáveis:** métricas puras; comando `eval --ab`.
- **Decisões:** D90.
- **Aceite:** métricas testadas sem modelo (golden fixo); relatório A/B reproduzível.

### E11-T08 ☐ `lightweight` para testes/offline
- **Objetivo:** embedder determinístico por hash (SHA-256 → vetor normalizado), sem pesos.
- **Entregáveis:** impl `lightweight`; uso em CI.
- **Decisões:** D89.
- **Aceite:** testes rodam sem download e sem rede; determinístico.

### E11-T09 ☐ Dedup semântico e descoberta de links
- **Objetivo:** alimentar o dedup eventual (E07-T06) e sugerir links não-declarados, sempre
  como **proposta**.
- **Entregáveis:** consultas ao espaço vetorial; integração com `audit`/`compact`.
- **Decisões:** D42, D47.
- **Aceite:** sugestões revisáveis; nada é fundido/ligado sem aceite.

### E11-T10 ☐ Runtime mínimo, backpressure e timeouts
- **Objetivo:** a digestão não introduz runtime pesado nem trava o comando.
- **Entregáveis:** worker bloqueante + **canal bounded** (`max_pending` como backpressure); pool
  limitado a `available_parallelism()`; inferência local em thread dedicada/`spawn_blocking`;
  timeout tipado e retry/backoff só em HTTP idempotente; nenhum lock atravessa `.await`
  (cancellation safety); poison tratado com `into_inner` (R32).
- **Decisões:** D79, D80. **Políticas:** R11, R12, R16, R32.
- **Aceite:** `cargo tree` sem `tokio full`; endpoint lento não trava; rajada acima de
  `max_pending` não estoura memória.

## Definition of Done

- [ ] Nenhum caminho do sistema bloqueia por embedding.
- [ ] Cache, fila, reconcile, purge e flush travados por teste.
- [ ] `eval --ab` disponível para decidir o modelo.
- [ ] Runtime mínimo e backpressure demonstrados.

## Não-objetivos

- Clusters semânticos (E10-T07) — consome este épico.
- Modelo fixo embutido (recusado: plugável com default pinado).
