# Busca e ranking

O pipeline de retrieval do knudge: da consulta textual ao hit ranqueado. A cascata vai **do mais
barato ao mais caro** — filtros determinísticos → BM25 no resíduo → âncoras como canal → fusão
RRF — e **degrada graciosamente** quando um canal falta.

- Código: `crates/knudge-core/src/retrieval/`
- Decisões: D35–D42, D81, D107, D121–D124, D143–D146, D151, D155, D161, D172, D173, D177,
  D179, D192, D206
- Fórmulas: [`matematica.md`](matematica.md) §1 (BM25), §2 (RRF).

## Cascata

```
consulta
  → filtros determinísticos (D41/D53)         [barato, corta o universo]
  → BM25 no resíduo (D35–D38)                 [lexical]
  → canal de âncoras (D81/D86)                [match exato de path/glob]
  → canal vetorial (D102/D123)                [opcional, off-path]
  → canal PPR (D192)                          [opcional, default desligado]
  → fusão RRF (D81/D123/D179)                 [soma canais presentes]
  → penalidade de contradição (D177)
  → confiança derivada / idade / drift (D87/D175/D203)
  → snippet / why / formatação (D39/D161)
```

Canal ausente/falho **degrada para o lexical com `warnings`**, nunca aborta (R33) — a menos que
`strict` esteja ligado.

## BM25 (D35–D38)

- `k1 = 1.5`, `b = 0.75`; **IDF por campo** (`statement` domina) e **peso por tipo**
  (`type_weight`, D37).
- **Boost por confirmação** (D38): `score × (1 + CONFIRMATION_STEP × confirmação)`,
  `CONFIRMATION_STEP = 0.1`. A confirmação é derivada (outcomes + tarefas — D87/D108).
- **Corte de alta frequência** (D173): termos com `df/N ≥ recall.max_term_ratio` (default `0.9`)
  são descartados via `Index::score_with`/`term_ratio`. **Desligado para corpora < 64 notas**
  (`MIN_CUTOFF_CORPUS`), onde `df/N` é alto para quase todo termo; `0` desliga.
- **Índice invertido** em memória (`postings::Postings`, E15-T06): peneira do BM25, nunca
  persistido.

## Tokenização (D36/D122/D172/D206)

A cadeia do canal lexical (índice + consulta) aplica, nesta ordem:

1. **ASCII explícita** (replica `\w`).
2. **Fold de diacríticos** (D172): NFD + descarte de marcas combinantes → `café ≡ cafe`.
   Casamento por **termo inteiro** (não prefixo). O `normalize` do schema **não muda**:
   `id`/`body_hash` seguem NFC.
3. **Stopwords PT+EN** e fragmentos de 1 caractere (`content_terms`, D122/D173).
4. **Stemming PT conservador** (D206): corta sufixos flexionais/derivacionais, radical mínimo de
   4 bytes, normalização de plural antes do corte derivacional.

O índice persiste o **radical**; o snippet continua com **termos crus** (`content_terms`).
`INDEX_FORMAT = "retrieval-v4"` invalida índices antigos (rebuild por `mtime`).

## Âncoras (D81/D86/D135)

- A âncora (`anchors`, único link externo canônico) alimenta um canal próprio: match exato de
  path/glob (`anchor::GlobPattern`, DP de uma linha, E15-T07).
- O `content_hash` é derivado (`.idx/anchors.jsonl`); `verify-on-hit` invalida âncora citada.
- O canal de âncoras só entra na fusão quando há interseção com o working set.

## Fusão RRF (D81/D123/D179)

`rrf::fuse` soma `peso_canal / (rrf_k + rank + 1)` por canal presente, com **tie-break
determinístico `(score desc, id asc)`**. `rrf_k = 60`.

Pesos (`FusionWeights`, config `recall.*_weight`):

| Canal | Default | Decisão |
|---|---|---|
| lexical | `1.0` | D123 |
| âncoras | `2.0` | D179 (um match exato pesa mais que um rank-1 lexical ruidoso) |
| vetorial | `30.0` | D123 (alto: comprime ranks no corpus pequeno) |
| PPR | `0.0` (desligado) | D192 (só compensa em corpora com arestas) |

`rrf_k` foi medido **inerte** no corpus rotulado (`bench/t09_fusao.md`); o lever real é
`anchor_weight` (D179).

## Canais opcionais

- **Vetorial** (D102/D123): `rank_query` sobre o índice de embeddings, filtrado pelos filtros
  determinísticos; `recall.semantic`/`semantic_top_k`.
- **PPR** (D192): `graph/rank.rs` calcula `PageRank`/`Personalized PageRank` sobre as arestas de
  autoridade (`references`/`supports`/`extends`/`replaces`), semeado pelo working set; iteração de
  potência determinística (damping 0,85, ≤32 iterações, tol. L1 1e-8, ordem canônica).

## Ranking sem query (D107/D175)

`retrieval::rank::rank` ranqueia por **confiança derivada**, sem texto: `confidence_score`
(evidência Beta — D189 + idade + drift), ordem `(confidence desc, id asc)`. A idade entra
**aditiva** (`AGE_WEIGHT = 0.05`) quando `similarity = 0` (D175); o drift desconta o score por
`drift_factor` (D203).

## Contradição (D177)

O lado **perdedor** de uma aresta `contradicts` declarada (menor confiança derivada) é rebaixado
por `CONTRADICTION_PENALTY` (0,1). Empate **não** elege perdedor (determinístico). `losers`
pré-indexa por id (O(C)); `rank` recebe `&Graph`.

## Views e filtros

- **Filtros determinísticos** (`filter::Filter`) aplicados **antes** da estatística (D41/D53):
  `--type`/`--class`/`--tag`/`--status`/`--scope`/`--anchor`/`--since`/`--until`.
- **Views** `ready`/`blocked` (`views.rs`, D53/D104): motivo de bloqueio derivado
  (`blocked_by`/`not_before`/`cycle`). `Status::VISIBLE` é a fonte única do default (D176) —
  `forgotten`/`superseded` ficam fora.

## Consulta temporal (D155)

`kd ask --as-of <TS>` reconstrói o conjunto **ativo em `T`** a partir do log de eventos
(`forget`/`restore` + `link replaces`) e roda o pipeline sobre o subconjunto. `T` no futuro é
`invalid_input` (2); `T` sem eventos é `[no_results]`. O `--json` ganha `as_of`/`historical`.

## Snippet e revelação (D161)

- `snippet::body_snippet` devolve o trecho do corpo que casou (busca no texto dobrado, devolve o
  original); `body_matches` indica match.
- Revelação progressiva: 1º hit com corpo completo, 2–5 truncado a `recall.preview_chars`
  (default 280), 6+ no padrão `id|statement|score|why`.

## Por que (`why`, D39/D121)

`Why` é um **conjunto fechado** (D39): `file_match > anchor_match > tracker_match > stars >
semantic > recent > universal`. Cada hit expõe `HitChannels` (parcelas RRF/boost — D151) no
`--json`, sem mudar o pipe.

## Saída

- Pipe: `id|statement|score|why` (D39). `ask --rank`: `id|statement|confidence|why` (D107).
- `--json`: `hits` com `channels`, `body_match`/`body_snippet`, `as_of`/`historical`; busca vazia
  → stdout `[no_results]` (D152). `recall.default_limit = 5` (D121).

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Orquestração `recall`/`get` | `retrieval/mod.rs` |
| Índice derivado | `retrieval/index/` |
| Índice invertido | `retrieval/postings.rs` |
| BM25 | `retrieval/bm25.rs` |
| Tokenização/stem | `retrieval/{token,stem}.rs` |
| Âncoras/globs | `retrieval/anchor.rs` |
| Filtros/views | `retrieval/{filter,views}.rs` |
| RRF/pesos | `retrieval/{rrf,weights}.rs` |
| Rank/contradição | `retrieval/{rank,contradiction}.rs` |
| Temporal/snippet/why | `retrieval/{temporal,snippet,why}.rs` |
| Pipeline/format | `retrieval/{pipeline,format}.rs` |

## Testes

`retrieval/tests/` (bm25, token, stem, anchor, filter, index, rank, rrf, recall, tags, views) +
`bench/src/quality.rs` (nDCG@k/MRR/Recall@k, E16-T01). `DIVERGENCES.md` #96–#109.
