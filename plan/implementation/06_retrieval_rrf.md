# E06 — Retrieval: BM25, âncoras e RRF

> **MVP.** Estrutura antes de estatística: filtros determinísticos primeiro, BM25 no resíduo,
> âncoras como canal, fusão **RRF** determinística. `recall`/`get`/`expand` são o que o LLM
> consome o tempo todo.
>
> **Decisões:** D35, D36, D37, D38, D39, D81, D87 (consumo), D15, D27.
> **Políticas:** R04, R14, R15, R33 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Um `recall` **barato, determinístico e explicável**, que funde canais lexical + âncoras +
vetorial (quando houver) e nunca falha por canal ausente.

## Pré-requisitos

E02, E05.

## Tarefas

### E06-T01 ☐ Índice derivado (invertido + forward + grafo)
- **Objetivo:** construir `.idx/` (JSON; binário quando justificar) a partir de `notas/` +
  `eventos/`; reconstruível; troca atômica (E03-T06).
- **Entregáveis:** builder do índice; esquema do `.idx/`; `rebuild`.
- **Decisões:** D15, D27.
- **Aceite:** rebuild a partir do canônico reproduz o índice byte a byte; leitura tolera
  índice ausente (reconstrói).

### E06-T02 ☐ BM25
- **Objetivo:** BM25 `k1=1.5, b=0.75`; **tokenização ASCII explícita** (replicar `\w`;
  `café`→`caf` documentado); **IDF por campo** (`statement` domina) e por `type`; **boost por
  confirmação derivada**: `score * (1 + 0.1 * (success + partial*0.5))`.
- **Entregáveis:** scorer BM25; tokenizador ASCII; pesos por campo/tipo.
- **Decisões:** D35, D36, D37, D38.
- **Aceite:** goldens de ranking; teste de tokenização com acentos; boost altera ordem como
  esperado.

### E06-T03 ☐ Âncoras como canal de recall
- **Objetivo:** match determinístico por `path`/`id` das `anchors` — não é só um campo.
- **Entregáveis:** canal de âncoras; interseção com working set.
- **Decisões:** D81 (origem arags A3), D86 (dados).
- **Aceite:** nota ancorada em arquivo do working set aparece sem depender de BM25.

### E06-T04 ☐ Fusão RRF determinística
- **Objetivo:** fundir canais por `1/(k+rank+1)` (k=60, config `recall.rrf_k`); ordenar por
  **`(score desc, id asc)`**; canais ausentes/falhos degradam para o lexical, com `warn`.
- **Entregáveis:** `rrf_fuse`; tie-break por id.
- **Decisões:** D81.
- **Aceite:** proptest de **determinismo** (duas execuções idênticas), união preservada,
  monotonicidade de rank; canal desligado não quebra.

### E06-T05 ☐ Filtros determinísticos e views `ready`/`blocked`
- **Objetivo:** filtrar por `type`, `classification`, `tags`, `status`, `container`, `anchors`
  **antes** da estatística; `ready`/`blocked` são views computadas via `depends_on` transitivo
  (não tools).
- **Entregáveis:** filtros O(1); views derivadas.
- **Decisões:** D41 (pré-requisito), D53 (view).
- **Aceite:** filtro reduz N antes do BM25; `ready`/`blocked` corretos em grafo com
  dependências encadeadas.

### E06-T06 ☐ Contratos `recall` / `get` / `expand`
- **Objetivo:** `recall` em pipe com **4ª coluna `why`** (`id|statement|score|why`, ~15
  tokens/hit); `get(ids)` devolve corpo só dos ids pedidos; `expand(id, kind, depth)` caminha
  o grafo (explícito).
- **Entregáveis:** formatação pipe; conjunto fechado de `why`
  (`file_match|anchor_match|tracker_match|stars|recent|universal`).
- **Decisões:** D39.
- **Aceite:** formato congelado por golden; `why` pertence ao conjunto; `expand` respeita
  `depth` e ciclos.

### E06-T07 ☐ Orçamento de índice e resultados parciais
- **Objetivo:** o retrieval não estoura memória nem aborta por causa de um canal.
- **Entregáveis:** limiar documentado de índice (acima → mmap/streaming ou aviso); `try_reserve`
  e `Cow<'_, str>` no parsing; canais ausentes/falhos retornam **resultado parcial + `warnings[]`**
  (nunca aborta); `--strict` promove warning a erro; `doctor` reporta tamanho do `.idx/`.
- **Decisões:** D39, D81. **Políticas:** R04, R14, R15, R33.
- **Aceite:** canal desligado retorna resultados + warning; índice acima do limiar não OOM.

## Definition of Done

- [ ] `recall` determinístico e explicável, com degradação graciosa.
- [ ] BM25 + âncoras + RRF travados por golden e proptest.
- [ ] Formato de retorno congelado (E13-T01).
- [ ] Índice com teto e canal falho não derruba o comando.

## Não-objetivos

- Orçamento do `prime` e auto-scope (E08).
- Embeddings (E11).
