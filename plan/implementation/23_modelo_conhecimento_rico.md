# E19 — Modelo de conhecimento rico

> **Épico de evolução (pós-E15).** Consolida a exploração aprovada em
> [`../proposals/modelo_conhecimento_rico.md`](../proposals/modelo_conhecimento_rico.md).
>
> **Versão alvo:** **0.5.0** (escopo único — R1–R7 na mesma release; ver o
> [plano-mestre](24_plano_mestre_0.5.0.md) §5). O bump de `schema_version` de R5 (T09) é
> ortogonal à versão do crate. Supersede o corte Q4 (0.5.x/0.6.0): a Trilha D vira a fase final
> de 0.5.0.
>
> **Decisões candidatas (provisório):** ~~D189~~ ~~D190~~ ~~D191~~ ~~D192~~ ~~D193~~
> (**registradas** — confiança Beta, retenção FSRS, data contract soft, autoridade PageRank/PPR,
> comunidades), D194 (reranking — mesmo servidor llama.cpp), D195 (MinHash/LSH), D196
> (Matryoshka/ANN), D197 (claims SPO + ontologia), D198 (TMS/defeasible — após E16/T07), D199
> (drift KL/JS), D200 (flow metrics), D201 (superfície enxuta — `forget` **permanece verbo**).
> Cada onda adotada **registra** sua
> decisão no `plan/03_decisoes-fechadas.md`; o número definitivo sai na implementação.
>
> **Políticas:** R15/R43 (deps/hot path), R33 (degradação graciosa), D04/D05/D13/D95 (contrato de
> bytes), D14 (sem retrocompatibilidade), D87 (confiança derivada), D92 (determinismo), E13-T09
> (`criterion` não é gate).

## Objetivo

Enriquecer o modelo de conhecimento em todas as frentes — `write`, `task`, `ask`, subcomandos,
vetores e ciclo de vida — com **gestão de dados moderna**, **representação de conhecimento** e
**matemática pura**, preservando os pilares (notas = verdade, índice derivado, enums fechados,
determinismo). Toda onda é **medida** (E16/T01) e reversível.

## Pré-requisitos

- E01–E15 ✅; **E16/T01** (bancada de qualidade) é pré-requisito de toda onda que afeta ranking.
- E16 (busca/depreciação) e E19 se sobrepõem em §5/R1–R4; **E19 generaliza** e reusa as decisões
  de E16 (D172–D179).
- **Prioridade 1 (R1):** Beta (R1) + retenção/FSRS (R2) + obrigatoriedades (R3) — maior razão
  ganho/risco, sem tocar bytes.

## Revisão do plano (achados)

1. **Confiança subalimentada.** `drift`/`feedback` mortos e `outcomes` sem posterior. R1 (Beta)
   dá incerteza **principiada**, reaproveita os ensaios existentes e **absorve E16/D174**
   (drift/feedback entram como insumos) — evita editar `confidence.rs` duas vezes.
2. **Depreciação por prazo.** Shelf-life plano → curva de esquecimento + revisão espaçada (R2),
   **substituindo E16/D178**.
3. **Contrato por tipo inexistente.** Nenhum tipo exige *slot*; R3 cria o **data contract** e o
   torna verificável por `doctor`/validators.
4. **Grafo subaproveitado.** Sem centralidade/propagação/comunidade → R4 (PPR/comunidades) e
   GraphRAG.
5. **Busca sem reranking.** R2 (reranking/expansão) é o maior salto de `ask`; depende de E16/T01.
6. **Dedup lexical.** R3 (MinHash/LSH) para escala e resolução de entidades.
7. **Ontologia mínima.** R5 (SPO + `same_as`/`broader`/`narrower`) é o único que toca
   `schema_version`; fica por último por risco de contrato.
8. **Tarefa sem tempo.** R7 (flow metrics/caminho crítico) usa o log de eventos já existente.
9. **Superfície larga.** R7 consolida verbos (≤10) e um vocabulário por conceito.

## Performance e orçamento (herança de E15)

> E19 é o épico de maior risco de regressão (ranking/grafo/vetores). Baseline em
> [`bench/RELATORIO.md`](../../bench/RELATORIO.md); a bancada é observação, não gate.

- **Caminhos quentes:** `ask` (T04 PPR, T06 reranking), `write`/dedup (T07), `Note::parse`
  (T09). `prune`/`doctor`/`knowledge map` são raros (custo aceito, como em E15-T13).
- **Orçamento (tolerância):** micro de `score`/`tokenize`/`Note::parse` ≤ **+20 %**; e2e
  `--no-idle` `ask` ≤ **+25 %** por onda (com Recall@k/nDCG ≥ baseline), acumulado ≤ **+40 %**
  no épico. Reranking (T06) é o item mais caro: **top-K curto** (ex.: 20–50), **opcional** e
  degradável (sem modelo ⇒ RRF) — nunca no caminho de `write`/`task`.
- **Protocolo:** A/B de **qualidade + latência** por onda (E16/T01); `make bench` com e sem
  `--no-idle`; recorte `bench/<tarefa>.md`; goldens/proptest para determinismo.
- **Padrões Rust (skill):** iteração de potência (T04) com **tolerância/nº fixo** (determinístico
  e limitado); MinHash/LSH (T07) com seed fixa e assinaturas cacheadas no `.idx/` (D84); ANN
  (T08) **só** quando o corpus doer — brute-force segue default; `Cow`/`entry`/capacidade.
- **Dependência:** nenhuma dep nova sem A/B ≥ 20 % (R43); `unicode-normalization` existente;
  `HashMap`/`HashSet`/`rayon` proibidos (D92).
- **Contrato:** R5 (T09) bumpa `schema_version` ⇒ medir o custo de parse e manter as chaves
  canônicas ordenadas (D04/D95); rebuild byte-idêntico.

## Sequência de execução

```
R1 (base):    T01 (Beta) → T02 (retenção/FSRS) → T03 (obrigatoriedades)
R4 (grafo):   T04 (PageRank/PPR) → T05 (comunidades/GraphRAG)
R2 (busca):   T06 (reranking/expansão/fusão)
R3 (escala):  T07 (MinHash/LSH) → T08 (Matryoshka/ANN)
R5 (ontolog.):T09 (SPO + ontologia + proveniência)
R6 (razão):   T10 (TMS/defeasible + drift)
R7 (tarefa):  T11 (flow metrics/caminho crítico) → T12 (superfície enxuta)
Fecho:        T13 (docs/goldens/matriz/CHANGELOG)
```

## Tarefas

### E19-T01 ☑ R1 — confiança bayesiana (Beta-Bernoulli) — **absorve E16/D174**
- **Escopo:** `lifecycle/confidence.rs` — posterior `Beta(α+Σs, β+Σf)` (parcial = 0.5); média
  posterior para `stars` e **limite inferior** do intervalo de credibilidade como confiança
  conservadora. **Absorve E16/D174**: persistir a validade de âncoras (derivado `.idx/`,
  off-path, D84) e alimentar `drift`; derivar `feedback` dos `outcomes` negativos. Puro,
  determinístico, sem dep.
- **Perf:** O(1) por nota; persistir `drift` num único walk (reusar `Corpus`), cacheado.
- **Aceite:** proptest (monotonicidade, limites `[0,1]`, empate só com mesma evidência); A/B em
  `knowledge rank`/`ask`; bytes de `notas/` intactos.
- **Feito (D189):** `lifecycle/beta.rs` (`posterior_mean`/`lower_bound` Wilson, puro, sem dep);
  `Meta.failures` derivado de `failure`/`abandoned`/`partial` (`filter.rs`); `INDEX_FORMAT`
  → `retrieval-v3`; `confidence_score` = `base + lower_bound(s,f) + 0,2·feedback + task`;
  `stars` usa a média posterior. **`feedback` de `outcomes` negativos absorvido** (as falhas).
- **Pendente:** persistência de `drift` (validade de âncoras off-path, `.idx/`, D84) → **T01b**.

### E19-T01b ☑ R1/D174 — persistir `drift` de âncoras (absorção de E16/T05)
- **Escopo:** derivado `.idx/drift.jsonl` (como `usage.jsonl`, purgável por D84); computar num
  **único walk** do projeto (off-path: `rebuild`/`doctor`/`maintenance proposals`); carregar em
  `RecallQuery`/`RankQuery` e alimentar `ConfidenceInput.drift` (`pipeline.rs`/`rank.rs`).
- **Perf:** walk único reusado; arquivo pequeno; ausente ⇒ `drift = 0` (degradação graciosa).
- **Aceite:** teste de que âncora quebrada reduz a confiança do `rank`; arquivo ausente é no-op;
  A/B na bancada.
- **Feito (D203):** `lifecycle/drift.rs` (`DriftEntry`/`DriftIndex`/`DriftStore`,
  `entries_from_validity`); `prune` persiste o drift do **mesmo** walk de `validity_map` (E16/T10);
  `ask`/`knowledge rank` carregam o índice; `confidence_score` desconta o score inteiro por
  `drift_factor`. Testes: `lifecycle::tests::drift`, `retrieval::tests::rank::drift_reduces_*`.
  A/B na bancada: fixture sem arquivo (drift ausente) ⇒ `ask`/`rank` inalterados.

### E19-T02 ☑ R2 — retenção por curva de esquecimento + revisão espaçada — **substitui E16/D178**
- **Escopo:** `lifecycle/shelf_life.rs` — retenção `R(t)=exp(-t/S)` com estabilidade `S` que
  cresce a cada `outcome` de sucesso (FSRS-like); `prune` propõe revisão quando `R < limiar`.
  Substitui `age_factor`/TTL **e E16/D178** por uma função única e configurável.
- **Perf:** O(N) em `prune`/`freshness` (raro); sem custo por comando.
- **Depende de:** T01.
- **Aceite:** proptest (retenção decrescente no tempo, crescente em revisões); A/B em
  `prune`/`freshness`; `prune` só propõe (D112).
- **Feito (D190):** `lifecycle/retention.rs` (`retention`/`stability_days`/`retention_for`, puro);
  `ShelfLife.effective_ttl_days` = `base + base·growth%·reviews/100` (inteiro, saturante);
  `origin` = `max(created, último ensaio, último uso se `renew_on_use`)`; config
  `retention.growth_percent` (default 50); `schema::outcome_stats` centraliza os `outcomes`;
  `prune --json` ganha `retention` (aditivo). `age_factor` do `confidence` segue separado.

### E19-T03 ☑ R3 — obrigatoriedades por tipo (data contract) — **soft, via corpo**
- **Escopo:** slots mínimos por espécie (`decision`→alternativas, `error`→causa/correção,
  `risk`→probabilidade/impacto, `def`→termo, `snippet`→linguagem, `fact`→lastro…) validados como
  **seções de corpo** e reportados por `doctor` (D162/D156) — **sem chave nova** (R1 não bumpa
  `schema_version`). **Soft** por padrão (aviso); promoção a **hard** (validators, D156) só
  depois, por config.
- **Perf:** checagem de corpo O(body), só em `write`/`doctor`.
- **Aceite:** teste por tipo (ausência ⇒ warning/`doctor`, e `invalid_input` só sob `strict`);
  `doctor` lista o que falta; `--dry-run` explica; nenhuma chave nova.
- **Feito (D191):** `schema/slots/` (`Slot`, `expected_slots`, `missing_slots`; fold D172,
  PT/EN, limite de palavra); `write` avisa (`warnings[]`) e sob `strict` dá `invalid_input`;
  `--dry-run` expõe `missing_slots`; check `body` do `doctor` (D162) conta slots ausentes (status
  `degraded`, `healthy` intacto). `fact` sem lastro fica com D162; `snippet`/`question` exigem
  âncora/`depends_on`.

### E19-T04 ☑ R4 — PageRank / Personalized PageRank
- **Escopo:** `graph/` — autoridade por PageRank (arestas `references`/`supports`/`extends`/
  `replaces`) e **PPR** semeado pelo *working set* para a busca; iteração de potência com
  tolerância fixa (determinístico). **Soma** ao canal de âncoras (canal novo; âncoras mantidas) —
  recalibrar a fusão **depois** (E16/T09).
- **Perf:** PPR com nº fixo de iterações/tolerância; grafo já em memória; A/B `ask`.
- **Depende de:** E16/T01.
- **Aceite:** proptest (convergência, simetria sob permutação); A/B em `ask`/`knowledge rank`.
- **Feito (D192):** `graph/rank.rs` (`pagerank`/`personalized_pagerank`/`power_iteration`, puro;
  damping 0,85, ≤32 iterações, L1 1e-8, *dangling* redistribuído pela personalização); canal
  `ppr` na fusão RRF (`recall.ppr_weight`, default **0,0** = desligado); `channels.ppr` no
  `--json` do `ask`. `knowledge rank` não usa a fusão, então o A/B é em `ask`.

### E19-T05 ☑ R4 — comunidades + GraphRAG
- **Escopo:** `lifecycle/clusters.rs` — comunidades (Louvain/Leiden) sobre o grafo (arestas +
  âncoras + vizinhança vetorial), variante determinística; resumos locais/globais para
  `knowledge map` materializado (D150).
- **Perf:** só em `knowledge map` (raro); variante determinística.
- **Depende de:** T04.
- **Aceite:** teste de comunidade estável; A/B em `knowledge map`; doc.
- **Feito (D193):** algoritmo em `graph/communities.rs` (Louvain *local moving* + agregação,
  determinístico); montagem + resumo em `lifecycle/communities.rs` (arestas + âncoras);
  `knowledge map --communities` (JSON aditivo + `--write` com `## Comunidades`). A vizinhança
  vetorial ficou de fora (entra com E19/T08/ANN).

### E19-T06 ☐ R2 — reranking + expansão de consulta + fusão calibrada
- **Escopo:** recuperar top-K e **reranquear** com cross-encoder local no **mesmo servidor
  llama.cpp** do embeddings (opcional; degrada para RRF); expansão (HyDE/sinônimos, zero-LLM por
  padrão); fusão por normalização de score (E16/T09). A reconciliação de endpoint (E17/D182)
  cobre **também** o reranker.
- **Perf:** reranqueia **top-K** (20–50), opcional, degrada para RRF; A/B `ask`.
- **Depende de:** E16/T01, T04.
- **Aceite:** A/B com ganho em nDCG/MRR; `--json` aditivo (`rerank`/`channels`); sem modelo ⇒ RRF.

### E19-T07 ✅ R3/D204 — MinHash/LSH + resolução de entidades
- **Escopo:** `write/dedup` — assinaturas MinHash + *blocking* LSH (fallback lexical para corpus
  pequeno); casamento por identidade (`same_as` em R5).
- **Perf:** MinHash/LSH com assinaturas cacheadas; A/B `write`/`compact`.
- **Depende de:** T01.
- **Aceite:** proptest (MinHash aproxima Jaccard; recall ≥ Dice no corpus); A/B em `write`/`compact`.
- **Feito (D204):** `write/dedup/lsh.rs` (MinHash 64 + banding 16×4, determinístico, sem dep) e
  `write/dedup/sieve.rs` (peneira exata extraída). Acima de `MIN_LSH_CORPUS` (256) **e** denso
  (termo em ≥ `DENSE_MIN_DF` docs), `propose_merges` usa LSH; senão a peneira exata (byte-idêntica).
  Dice exato + limiar preservados (recall ≥). **A/B:** denso N=1000 1,42 s → 46 ms (−97 %);
  `compact`/`doctor` N=1000 −94 %; esparso inalterado. `same_as` fica para R5/T09.

### E19-T08 ☐ R3 — Matryoshka + ANN
- **Escopo:** `embeddings/` — truncar dimensões (MRL) e quantizar (int8/binário); ANN
  (HNSW/IVF-PQ) só quando o corpus doer; brute-force segue default.
- **Perf:** brute-force default; ANN só com corpus grande; int8 reduz memória.
- **Depende de:** T06.
- **Aceite:** A/B (latência × recall); determinismo (seed fixa); `.idx/` reconstruível.

### E19-T09 ☐ R5 — claims SPO + ontologia + proveniência
- **Escopo:** `schema/` — claims `(sujeito, relação, objeto)` + arestas `same_as`/`broader`/
  `narrower`/`related` (SKOS-lite) + proveniência (`entity/activity/agent`, PROV-lite); **bump de
  `schema_version`** e rebuild.
- **Perf:** medir `Note::parse`; chaves canônicas ordenadas; rebuild byte-idêntico.
- **Depende de:** T03, T07.
- **Aceite:** goldens TOON/rebuild; inferência e contradição precisas; `DIVERGENCES.md`.

### E19-T10 ☐ R6 — TMS/defeasible + drift (KL/JS)
- **Escopo:** rastrear suposições e invalidar dependentes (TMS/ATMS); derrota de crença
  (`replaces` + `contradicts`); detecção de drift por KL/JS sobre termos/embeddings no tempo.
- **Perf:** TMS/drift só em `prune`/`doctor` (off-path).
- **Depende de:** E16/T07, T09.
- **Aceite:** teste de retratação (dependentes caem); A/B de drift em `prune`/`doctor`.

### E19-T11 ✅ R7/D205 — flow metrics + caminho crítico
- **Escopo:** derivar do log de eventos `cycle time`, `lead time`, `throughput` e o **caminho
  crítico** do DAG `depends_on` (PERT/CPM); expor em `task`/`rewind`.
- **Perf:** derivar do log em **um** pass; cachear em `rewind`/`task`.
- **Depende de:** E16/T01.
- **Aceite:** proptest (caminho crítico = maior caminho ponderado); `--json` aditivo.
- **Feito (D205):** `task/flow.rs` (`TaskFlow`/`throughput`/`critical_path`, puros); `kd task flow`
  (`resumo|`/`throughput|`/`critico|` + `--json`) e `rewind --json` (`data.flow`). Sem verdade nova.

### E19-T12 ☐ R7 — superfície enxuta
- **Escopo:** consolidar verbos (≤10), um vocabulário por conceito (`knowledge`→`ask`/`map`,
  `self` enxuto — E18); **`forget` permanece verbo** (é a aplicação do `prune`, D112);
  `--help`/matriz/`prime` atualizados. Depende de E18 (worker/`drain service`).
- **Perf:** enxugar verbos não muda o custo dos verbos mantidos.
- **Depende de:** E18.
- **Aceite:** contagem de verbos; testes de remoção (exit 2); docs em sincronia.

### E19-T13 ☐ Fecho — docs, goldens, matriz e CHANGELOG
- **Escopo:** `docs/01-conceitos.md`, `docs/04-ask.md`, `docs/05-write.md`, `docs/06-task.md`,
  `docs/07-knowledge.md`, `SKILL.md`, `llms.txt`, `16_cli_surface.md`, `17_matriz_aceitacao.md`,
  `DIVERGENCES.md`, `CHANGELOG.md` (`[0.6.0]`), `Cargo.toml`; incorpora os **pontos em aberto**.
- **Aceite:** `make check` + `make ci` verdes; versão em sincronia.

## Definition of Done

- [ ] `make check` verde em cada tarefa; `make ci` verde ao fechar.
- [ ] Toda onda com **A/B de qualidade** (E16/T01); bytes de `notas/` intactos fora de R5.
- [ ] Confiança bayesiana (R1, absorve E16/D174) e retenção por uso (R2, substitui E16/D178)
      substituem as heurísticas; obrigatoriedades por tipo (R3) **soft/via corpo** verificadas
      por `doctor`.
- [ ] Grafo usado para autoridade/propagação/comunidade (R4); busca com reranking (R2).
- [ ] Dedup escalável (R3); ontologia/claims com `schema_version` (R5); TMS/drift (R6);
      flow metrics (R7); superfície ≤10 verbos.
- [ ] Nenhum `src/` > 300 linhas; zero `unwrap/expect/panic/unsafe`; stdout = dados (R20).

## Não-objetivos

- `criterion` como gate (E13-T09); dep nova sem ganho ≥20% (R43); LLM obrigatório; servidor
  obrigatório (R16); migração de corpus (D14); perder o contrato de bytes.

## Riscos

| Risco | Mitigação |
|---|---|
| R5 quebra o contrato TOON | `schema_version` + rebuild; onda dedicada e última |
| matemática "exótica" (FCA/espectral) sem ganho | **experimental**; só com A/B |
| Beta/FSRS mal calibrados | parâmetros em config; defaults conservadores; A/B |
| reranking exige modelo | local/opcional; degrada para RRF (R33) |
| superfície mexida quebra usuários | D14; `--help`/matriz/CHANGELOG no mesmo commit |
| determinismo (ANN/LSH) | seed fixa; ordem canônica; fallback exato |

## Pontos em aberto (aguardando indicação do usuário)

> O usuário indicou que ainda tem pontos a acrescentar **antes** do início do código. Cada ponto
> vira uma tarefa `E19-Txx` (ou nota) aqui.

- _(a preencher)_
- **Candidato (análise de riscos):** `kd rewind --digest` — ponteiros das notas omitidas pelo
  orçamento (nunca sumário semântico); aditivo e default byte-idêntico. Ver
  [`../proposals/riscos_memoria_duravel.md`](../proposals/riscos_memoria_duravel.md) §2.3.
