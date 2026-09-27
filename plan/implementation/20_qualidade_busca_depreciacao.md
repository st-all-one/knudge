# E16 — Qualidade da busca e depreciação de conhecimento

> **Épico de evolução (pós-E15).** Consolida o plano aprovado em
> [`../proposals/qualidade_busca_depreciacao.md`](../proposals/qualidade_busca_depreciacao.md).
>
> **Versão alvo:** **0.5.0** — muda o *resultado* de busca (não os bytes de `notas/`; o índice é
> derivado, D15/D27/D84). Entradas deste épico vão para o `CHANGELOG.md` no fecho (T12).
>
> **Decisões propostas:** D172 (acentos), D173 (alta frequência), D175 (idade no rank), D176
> (status consistente), D177 (`contradicts`), D179 (fusão recalibrada). **Absorvidas por E19:**
> D174 (confiança → E19/D189) e D178 (retenção → E19/D190) — ver
> [`revisao_integrada.md`](../proposals/revisao_integrada.md) §3. **Stemming PT** é condicional
> (novo `Dxx` só se pagar). **Políticas:** R15 (hot path), R20–R23 (logs), R33 (degradação
> graciosa), R43 (dependências), D43 (status), D84 (derivado reconstruível), D87 (confiança
> derivada), D145 (avaliação na bancada), D151 (recalibração offline).

## Objetivo

Fazer o `ask`/`knowledge` **encontrar o que importa** em PT-BR (acentos, flexões, sinônimos) e
fazer a **depreciação de conhecimento** ser dirigida por **evidência** (confirmação, uso, drift de
âncoras, contradição) — não só por tempo. Toda mudança de comportamento é **medida** antes de
adotada; bytes de `notas/` permanecem intactos. A **matemática da confiança e da retenção** foi
movida para **E19** (D189/D190), que absorve os antigos D174/D178.

## Pré-requisitos

- E01–E15 ✅ (`make check` verde).
- Bancada `bench/` e alvo `make bench` (fora do workspace; observação, não gate — E13-T09).
- **Prioridade 1:** T01 (avaliação de qualidade) — é o **habilitador** de todas as decisões
  dependentes de medição (D172–D179). Sem ela, o guardrail "se não move o ponteiro, reverte" não
  é aplicável à busca.
- **Correção barata primeiro:** T02 (status consistente) — bug, sem dependência, alto impacto.

## Revisão do plano (achados e ajustes)

1. **A avaliação precede a otimização.** D145 tirou o `eval` do produto e o pôs na bancada, mas a
   bancada só mede latência; T01 adiciona **qualidade** (Recall@k/MRR/nDCG@k) para que T03–T09
   tenham número.
2. **`notas/` é contrato; `.idx/` é derivado.** D172 muda o tokenizador, que alimenta o índice
   derivado e o **dedup** — `notas/` e `id`/`body_hash` ficam intactos, mas goldens de retrieval
   e `write` mudam **com intenção** (e `DIVERGENCES.md` ganha linha).
3. **Confiança tem termos mortos.** D175 (`knowledge rank` passa a ver idade) fica; a **fórmula**
   (drift/feedback → Beta) vai para **E19/D189** (absorve D174).
4. **Consistência antes de sofisticação.** D176 (bug) precede D177 (depreciação por evidência)
   para não construir sobre base inconsistente; a **retenção** vai para **E19/D190** (absorve
   D178).
5. **Stemming é condicional.** Só entra se T01 provar ≥ 20 % sobre D172/D173; senão, rejeição
   medida (padrão T12).
6. **A fusão (D179) só depois de T01.** Recalibrar `rrf_k`/pesos sem métrica é adivinhação.
7. **`prune` continua propondo (D112).** D177 só **propõe** — nada é demolido sem aceite.

## Performance e orçamento (herança de E15)

> Baseline v0.4.0 em [`bench/RELATORIO.md`](../../bench/RELATORIO.md); bancada em `bench/`
> (`make bench`), **observação, não gate** (E13-T09). Regra: **não regredir o caminho quente sem
> ganho de qualidade medido**.

- **Caminhos quentes:** `ask`/`knowledge rank` (T02/T03/T04/T06/T07/T09), o `write`/dedup (T03)
  e o `prune` (T07). O `Corpus` (O1/O7) e o `Postings` (O2) já carregados devem ser **reusados**
  — proibido novo walk do disco por nota.
- **Orçamento (tolerância):** micro de `tokenize`/`content_terms`/`score` ≤ **+20 %**; e2e
  `--no-idle` (N=1167) `ask`/`rewind` ≤ **+15 %** por tarefa e ≤ **+25 %** acumulado no épico,
  **com** ganho de Recall@k/MRR/nDCG demonstrado em T01. Perda "absurda" (ex.: 2×) ⇒ cortar
  escopo ou reverter.
- **Protocolo de medição (por tarefa que toca o quente):** `make bench` antes/depois (N=1000,
  8 amostras) **com e sem `--no-idle`**; recorte `bench/<tarefa>.md`; micro
  `bench/micro-<tarefa>.md`; qualidade `bench/qualidade.md`; goldens só mudam **com intenção**
  (`DIVERGENCES.md`). Registrar **adotado** (com número) ou **rejeitado** (com número).
- **Padrões Rust (skill):** `Cow` para evitar clone na tokenização (T03); `entry`/`get_mut` no
  lugar de `contains_key`+`insert`; capacidade pré-alocada; `sort_unstable_by` onde o comparador
  é **total**; `#[cold]` em construtores de erro. Sem `HashMap`/`HashSet`/`rayon` (D92/R43);
  `BTreeMap`/`IndexMap`.
- **Dependência:** `unicode-normalization` é **existente** (D172) — nada novo; stemming (T11) só
  com A/B ≥ 20 % (R43), senão **rejeitado por medição**.
- **Laços quentes:** a idade (T06) e a contradição (T07) entram como fatores **O(1)** por nota,
  com arestas pré-indexadas (padrão `Graph.parents`), nunca varrendo o grafo por nota; a fusão
  (T09) mantém o RRF `O(K log K)`.

## Sequência de execução

```
Prioridade 1 (começa já): T01 (avaliação de qualidade) — habilitador.

A. Correção e base lexical
T01 → T02 (status consistente) → T03 (acentos) → T04 (alta frequência)

B. Depreciação por evidência
T06 (idade no rank) → T07 (contradicts)
   [T05 (drift+feedback) e T08 (retenção) → **E19/T01/T02**, absorvidos]

C. Calibração e fecho
T09 (fusão) → T10 (perf) → T11 (stemming, condicional) → T12 (docs/goldens/matriz/CHANGELOG)
```

## Tarefas

### E16-T01 ☑ Avaliação de qualidade na bancada (habilitador)
- **Objetivo:** medir **qualidade** de retrieval (não latência) na bancada externa.
- **Escopo:** modo `quality` em `bench/` (zero-dep, fora do workspace): conjunto rotulado
  `consulta → ids esperados` (derivado do corpus + sintético + amostra manual) e métricas
  **Recall@k**, **MRR**, **nDCG@k**; relatório `bench/qualidade.md`/`.json`. Baseline registrado.
- **Feito:** `bench/src/quality.rs` (corpus PT-BR de 12 tópicos × 8 notas + 96 distratores = 192
  notas; 24 consultas `com-acento`/`sem-acento`); modo `quality` em `bench/src/main.rs`;
  `make bench-quality`; alvo irmão `make bench` inclui a qualidade. **Baseline:**
  `com-acento` nDCG@k = 100 % (ranking perfeito); `sem-acento` nDCG@1 = 8,3 % — a régua exata
  de D172 (fold). Artefatos `bench/qualidade.md`/`.json` versionados.
- **Depende de:** nada.
- **Aceite:** `make bench` (ou alvo irmão) emite as métricas de qualidade; baseline gravado;
  `make check` inalterado (bancada não é gate, E13-T09).

### E16-T02 ☑ D176 — consistência de status em todo consumidor
- **Escopo:** `CorpusScope::select`/`Selection` e/ou `rank()`/`structural_clusters_filtered()`
  passam a excluir `Forgotten | Superseded` por padrão, alinhados a `ask`/`tags`/`dedup`/
  `body_check` (D43). `--status` explícito continua permitindo inspecioná-los.
- **Feito:** `Status::VISIBLE` (`schema/types.rs`) é a **fonte única** do default (D43);
  `CorpusScope::select` (`commands/corpus.rs`) aplica em `knowledge rank`/`map`, `rewind` e
  `maintenance learn/compact/prune`; `ask/query.rs` passou a reusar a constante (mesmo
  comportamento). Teste de regressão por consumidor (`real_usage`); linha #95 em
  `DIVERGENCES.md`.
- **Perf:** filtro O(N) sobre o vetor já carregado; **sem** novo walk do disco (reusa `Corpus`).
- **Depende de:** T01 (para não regredir métricas de ranking).
- **Aceite:** `knowledge rank`/`map` não incluem deprecados (teste de regressão por consumidor);
  goldens de `knowledge rank`/`map` atualizados; linha em `DIVERGENCES.md`.

### E16-T03 ☑ D172 — normalização de acentos na tokenização
- **Escopo:** `retrieval/token.rs` — `tokenize` dobra acentos (NFD + remoção de marcas
  combinantes) com fast-path ASCII (`Cow`), via `unicode-normalization` (dep existente);
  `content_terms`/`query_terms` herdam; `snippet.rs` acompanha. O `normalize` do schema
  **não muda**.
- **Feito:** `token.rs` ganhou `folded_chars`/`fold_ascii`/`folded_byte_to_original`;
  `tokenize` dispacha ASCII (emprestado) × não-ASCII (NFD). `snippet.rs` busca no texto dobrado e
  devolve o trecho **original**. O índice derivado ganhou o cabeçalho `INDEX_FORMAT`
  (`retrieval-v2`) em `serialize`/`parse` — sem ele, um `.idx/` pré-fold seria servido por
  engano (frescura é por `mtime`). Testes: proptest de idempotência/ASCII,
  `precomposed_and_decomposed_fold_equivalently`, `accents_fold_to_ascii`,
  `old_format_index_is_rejected`, `matches_accented_body_and_query_interchangeably`.
  **Ganho medido** (`bench/qualidade.md`): família `sem-acento` nDCG@1 **8,3 % → 100 %**,
  MRR 8,3 % → 100 %; `com-acento` segue 100 %. Sem regressão de latência (micro `tokenize`
  260 → 243 ns; `ask` N=1000 dentro do ruído). `DIVERGENCES.md` #96. `notas/`/`id`/`body_hash`
  intactos (goldens TOON/hash verdes).
- **Perf:** fast-path ASCII devolve `Cow::Borrowed` (zero alocação); NFD só em não-ASCII; micro
  `tokenize` (ASCII e acentuado) trava o custo; índice/dedup reconstruídos uma vez.
- **Depende de:** T01.
- **Aceite:** proptest de idempotência do fold; golden de retrieval regenerado com intenção;
  A/B na bancada de qualidade (ganho medido em PT-BR); `notas/` e `id`/`body_hash` inalterados
  (goldens TOON/hash verdes); linha em `DIVERGENCES.md`; golden de `write`/dedup regenerado. ✔

### E16-T04 ☑ D173 — termos de alta frequência
- **Escopo:** `STOPWORDS` ganha as formas dobradas do PT; o canal lexical descarta termos com
  `df/N` acima de um limiar (config, default conservador). Sem quebrar o determinismo do canal.
- **Feito:** `STOPWORDS` ganhou 63 formas dobradas do PT (`ja`, `sao`, `nao`, `tambem`, `ate`,
  `apos`, `entao`, `porem`, `voce`, …) mantendo a ordenação para busca binária. O corte por
  `df/N` entrou em `Index::score_with` (novo parâmetro `max_term_ratio` + `term_ratio`/`
  drop_high_frequency`), configurado por `recall.max_term_ratio` (default `0.5`) e desligado para
  corpora < `MIN_CUTOFF_CORPUS` (64 notas) — abaixo disso `df/N` é alto para quase todo termo.
  Testes: `term_ratio_counts_documents_with_the_term`,
  `high_frequency_terms_are_dropped_only_for_large_corpora`,
  `high_frequency_cutoff_is_noop_below_min_corpus`; bancada de qualidade **sem regressão**
  (100 %/100 %). O corte é o "cinto de segurança" de idioma; o ganho concreto veio das stopwords
  dobradas + fold (T03).
- **Perf:** `df` já está no `Index` (recomputado uma vez); corte O(1) por termo; sem alocação
  extra por token (o filtro reusa os `Cow`).
- **Depende de:** T03.
- **Aceite:** medição (qualidade + latência) na bancada; goldens de retrieval; teste de que o
  corte é estável e ordenado. ✔

### E16-T05 ◐ D174 — confiança derivada completa (`drift` + `feedback`) → **absorvida por E19/T01**
- **Escopo:** persistir a validade de âncoras como derivado (`.idx/`, off-path, purgável por
  D84); alimentar `ConfidenceInput.drift` em `recall`/`rank`; derivar `feedback` dos `outcomes`
  negativos. **Movida para E19/T01**, onde a fórmula vira o posterior Beta (D189) — evita editar
  `confidence.rs` duas vezes.
- **Depende de:** —
- **Aceite:** coberto por **E19/T01** (a persistência de `drift` entra como insumo do Beta).
- **Feito (D189):** `feedback` de `outcomes` negativos absorvido como `failures` do Beta.
  **Feito (D203):** `drift` persistido em `.idx/drift.jsonl` (walk único off-path) e aplicado à
  confiança (`rank`/`ask`) — ver **E19/T01b**.

### E16-T06 ☑ D175 — idade no ranking sem query
- **Escopo:** `retrieval/rank.rs` — a idade entra de forma **aditiva** quando `similarity = 0`
  (hoje o `age_factor` é anulado).
- **Perf:** idade O(1) por nota a partir do `mtime`/`created_at` já lido no `Corpus`; sem
  `stat` por nota no laço.
- **Depende de:** E19/T01 (o Beta é a base da confiança).
- **Aceite:** teste de que nota antiga e não confirmada rankeia abaixo de recente equivalente;
  golden de `knowledge rank`; A/B na bancada.
- **Feito (D175):** `AGE_WEIGHT = 0,05`; `recency = AGE_WEIGHT · age_factor · (1 − similarity)`
  em `confidence_score` (vale também no `recall`, mas lá não muda a ordem — o `score` é o RRF).
  Bancada de qualidade **sem regressão**; `knowledge rank` não tem golden numérico.

### E16-T07 ☑ D177 — `contradicts` no ranking e na depreciação
- **Escopo:** `recall`/`rank` rebaixam (ou avisam) o lado perdedor de uma aresta `contradicts`
  declarada, com a confiança derivada como desempate; `demotion_candidates` passa a considerar
  contradição como motivo (novo `DemotionReason`), mantendo D45 (ciclos protegidos) e D112
  (só propõe). Sequência: E16/T07 é o **mínimo**; E19/T10 (TMS) generaliza (Q12).
- **Perf:** arestas `contradicts` pré-indexadas por id (`BTreeMap`); rebaixamento O(1) por nota;
  `demotion_candidates` roda só no `prune` (raro).
- **Depende de:** E19/T01 (confiança), T06.
- **Aceite:** testes de ranking e de `prune` (proposta, não aplicação); goldens de `prune`/`rank`.
- **Feito (D177):** `retrieval/contradiction.rs` (lado perdedor por confiança, empate sem
  perdedor); `CONTRADICTION_PENALTY` no `rank`/`recall`; `rank` passa a receber `&Graph`;
  `DemotionReason::Contradicted` no `prune` (só propõe). **Follow-up:** check read-only do
  `doctor` (`CheckId::Contradictions`).
- **Candidato (análise de riscos):** check **read-only** do `doctor` (`CheckId::Contradictions`)
  que lista pares `X ⊣ Y` com ambos visíveis — fecha o ciclo propor→ranquear→**auditar**. Ver
  [`../proposals/riscos_memoria_duravel.md`](../proposals/riscos_memoria_duravel.md) §2.2.

### E16-T08 ☐ D178 — shelf-life consciente de evidência → **substituída por E19/T02**
- **Escopo:** `shelf_life.rs` — `effective_expiry` estende o prazo por confirmação/`outcomes`.
  **Substituída por E19/T02**, que troca o TTL plano pela curva de esquecimento (D190); o
  invariante "confirmação só estende" vira "revisão só aumenta a estabilidade".
- **Depende de:** —
- **Aceite:** coberto por **E19/T02**.

### E16-T09 ☑ D179 — recalibração da fusão
- **Escopo:** varrer `rrf_k` e pesos (`lexical`/`anchor`/`semantic`) na bancada de qualidade;
  ajustar os **defaults** (e documentar a calibração). Opcional: avaliar **normalização de score
  por canal** contra o RRF atual.
- **Perf:** fusão mantém `O(K log K)`; a normalização por canal não muda a complexidade; medir
  `ask` antes/depois.
- **Depende de:** T01 (medição), T03/T04 (canal lexical estável).
- **Aceite:** A/B com ganho demonstrado (nDCG/MRR); defaults justificados no plano; goldens
  atualizados com intenção.
- **Feito (D179):** famílias multi-canal (`working-set`/`sinonimo`) + `make bench-sweep`
  (`bench/t09_fusao.md`). `rrf_k` medido **inerte** (rejeitado); `anchor_weight` `1.0 → 2.0`
  (working-set nDCG@5 87,7 % → 100 %). `DIVERGENCES.md` #105.

### E16-T10 ☑ Perf do caminho de busca e de `prune`
- **Escopo:** `body_share` tokeniza a consulta **uma vez** por `build_hits`; `validity_map` faz
  **um** walk do projeto e valida todas as âncoras; `task_confirmers` é pré-indexado por âncora
  (evita `O(N × T × A²)` em `rank`).
- **Perf:** é o item que **paga** o orçamento do épico (remove quadráticos residuais); saída
  byte-idêntica (goldens) — só o custo muda.
- **Depende de:** E19/T01 (drift), T01 (baseline de latência).
- **Aceite:** micro/A-B na bancada; saída byte-idêntica (goldens).
- **Feito:** `compute_anchor_validity_cached` (walk único; `validity_map` reusa) e
  `body_share_terms` (tokenização única em `build_hits`). **`prune --universe` N=1000: 118,6 ms →
  45,9 ms (−61 %)**; `ask`/`recall` inalterados. `task_confirmers` medido **não-gargalo** no
  corpus da bancada (T=0 sem `outcomes`) — **rejeitado por medição** (padrão E15-T12).

### E16-T11 ☐ Stemming PT (condicional)
- **Objetivo:** decidir, com número, se um stemmer PT conservador paga.
- **Escopo:** implementar atrás de flag de config, medir na bancada de qualidade; adotar só se
  ganho ≥ 20 % sobre D172/D173; caso contrário, **rejeitar e registrar**.
- **Perf:** atrás de flag de config; medir **antes** de adotar (R43); se não pagar, reverter.
- **Depende de:** T03/T04/T09.
- **Aceite:** decisão escrita (incorporado ⇒ novo `Dxx`; rejeitado ⇒ registro no épico), com o
  recorte da bancada.

### E16-T12 ☐ Fecho — docs, goldens, matriz e CHANGELOG
- **Escopo:** `docs/04-ask.md`, `docs/07-knowledge.md`, `docs/09-maintenance.md`, `SKILL.md`,
  `llms.txt`, `AGENTS.md` (se preciso), `MODULE.md` de `retrieval`/`lifecycle`, `DIVERGENCES.md`,
  `17_matriz_aceitacao.md`, `CHANGELOG.md` (`[0.5.0]`) e `Cargo.toml` (via `make update-version`).
- **Depende de:** todas.
- **Aceite:** `make check` + `make ci` verdes; grep por termos obsoletos; versão em sincronia.

## Definition of Done

- [ ] `make check` verde em cada tarefa; `make ci` verde ao fechar.
- [ ] **Bytes de `notas/` idênticos** (TOON/hash/`id`) — só o derivado `.idx/` e os resultados
      de busca mudam, com intenção.
- [ ] Ganho de **qualidade** medido em toda tarefa de busca (`bench/qualidade.md` atualizado).
- [ ] `knowledge rank`/`map` excluem `forgotten`/`superseded` (D176); idade no `rank` (D175);
      `contradicts` no ranking/`prune` (D177); fusão recalibrada (D179). `drift`/`feedback`
      (D174) e retenção (D178) são entregues por **E19/T01/T02**.
- [ ] Stemming com decisão escrita (incorporado/rejeitado) e recorte da bancada.
- [ ] Cada decisão citada tem teste/golden que a trava; linha em `DIVERGENCES.md` quando for borda.
- [ ] Nenhum `src/` > 300 linhas; zero `unwrap/expect/panic/unsafe`.

## Não-objetivos

- `criterion` como gate (E13-T09/R43).
- Dep nova sem ganho ≥ 20 % medido (R43); `HashMap`/`HashSet`/`rayon` (determinismo).
- Mudar o `normalize` do schema ou qualquer byte de `notas/` (D06/D95).
- Migração de corpus (D14); busca fuzzy/typo-tolerant de propósito geral.
- Demolição automática sem aceite (D112) ou fora da proteção de ciclo (D45).

## Riscos

| Risco | Mitigação |
|---|---|
| D172 muda resultado de busca e dedup | `notas/` intacto; golden de retrieval/`write` regenerado com intenção; `DIVERGENCES.md`; A/B |
| Fold colide conceitos (`café`≡`cafe`) | decisão explícita (D172); se indesejado, indexar ambos os tokens — medir |
| D173 agressivo demais (perde recall) | corte por `df/N` calibrado; default conservador; config por projeto |
| D175/D176 mudam ranking/consumidores | goldens de `knowledge rank`/`map`; teste de regressão por consumidor |
| D177 rebaixa/aposenta demais | piso de confiança; grace period; `prune` só propõe (D112) |
| D179 recalibra defaults | medição explícita; defaults antigos acessíveis por config |
| stemming PT over-stemming | condicional e medido; rejeitar sem hesitar |
| qualidade da bancada não representar uso real | três fontes (corpus/sintético/manual) + baseline versionado |
