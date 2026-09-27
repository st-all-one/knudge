# Modelo de conhecimento rico — exploração e roteiro

> **Status:** exploração (não implementada). Documento de visão + roteiro; cada onda adotada vira
> um `Dxx` e tarefas de épico próprias. Complementa
> [`qualidade_busca_depreciacao.md`](qualidade_busca_depreciacao.md) (E16).
>
> **Objetivo:** enriquecer a **abordagem sobre o conhecimento** do knudge — no `write`, no `task`,
> no `ask`, nos subcomandos e nos vetores — emprestando o que há de mais sólido em **gestão de
> dados**, **representação de conhecimento** e **matemática pura**, **sem** quebrar os pilares
> (notas = verdade, índice derivado, enums fechados, contrato de bytes).
>
> **Versão alvo:** **0.5.0** (tudo — R1–R7; o bump de `schema_version` de R5 entrou na mesma
> versão). **Absorve E16/D174** (confiança) e **substitui E16/D178** (retenção) — ver
> [`revisao_integrada.md`](revisao_integrada.md).

---

## 0. Como ler

- **§1–2** mapeiam o modelo atual e separam o que é **rico** do que é **fino**.
- **§3–7** propõem as técnicas modernas por tema, cada uma ancorada em **módulo** e **decisão**.
- **§8** prioriza em ondas (impacto × esforço × risco de contrato).
- **§9** riscos/não-objetivos; **§10** pontos em aberto.

---

## 1. O modelo atual (mapa)

| Camada | O que existe | Módulo |
|---|---|---|
| Nota | `statement` (≤120) + `body` (Markdown), 25 chaves canônicas, opcionais omitidos | `schema/frontmatter.rs`, `schema/keys.rs` |
| Espécies | 10 armazenáveis + `epic` derivado (`scope=epic`) | `schema/types.rs` |
| Arestas | 8 tipos fechados (`references`, `depends_on`, `contradicts`, `supports`, `extends`, `replaces`, `rejects`, `results_in`) | `schema/edge.rs` |
| ID/hash | `id = hash(type + U+001F + normalize(statement))`; `body_hash` | `schema/id.rs`, `schema/hash.rs` |
| Ciclo de vida | `classification` (shelf-life), `status`, `outcomes`, `revision`, `superseded_by` | `lifecycle/`, `write/update` |
| Escrita | idempotência por conteúdo + dedup lexical Dice (0.75/0.92) | `write/dedup`, `write/draft` |
| Tarefa | `epic ⊃ {issue ⊃ task | task}`, papel derivado, `impact` por `depends_on` | `task/` |
| Busca | filtros → BM25 (IDF por campo) → âncoras → RRF (+ vetorial) | `retrieval/` |
| Confiança | `similarity·drift·age + 0.2·(confirmation+feedback) + task_confirmation` | `lifecycle/confidence.rs` |
| Eventos | log append-only (`eventos/`), reconstrução `as_of` | `store/events`, `retrieval/temporal.rs` |
| Vetores | cosseno brute-force, `.idx/embeddings.jsonl` | `embeddings/` |
| MCP | 3 gatilhos + `knudge_status`, hints-ponteiro | `knudge-mcp/src/triggers.rs` |

**Pilares inegociáveis:** notas = verdade; índice/derivados reconstruíveis (D15/D84); enums
fechados (D04/D13); contrato de bytes TOON (D04/D05/D95); determinismo (D92); zero-LLM no núcleo.

---

## 2. Diagnóstico: rico × fino

**Já rico (preservar):** IDs endereçados por conteúdo (estilo *content-addressed*), enums fechados,
arestas explícitas, *event sourcing* + reconstrução `as_of`, confiança **derivada** (não
armazenada), fusão híbrida RRF, dedup em duas fases, portas/adaptadores.

**Fino (onde há ganho):**

1. **Proposição livre, sem estrutura.** `statement` é texto; a semântica mora no `type`. Não há
   *sujeito–predicado–objeto* (SPO), qualificadores nem papéis. → não se pode inferir, comparar
   nem contradizer com precisão.
2. **Proveniência pobre.** `source` (string única) + `evidence` (mapa livre). Sem autor, método,
   confiança de origem nem *lineage* por afirmação.
3. **Temporalidade simples.** Só `created_at`; `expires_at`/`not_before` foram removidos (D135).
   `as_of` reconstrói **estado**, mas o **conteúdo é o atual** (borda documentada) — não é
   bitemporal de verdade.
4. **Incerteza ad-hoc.** `outcomes` são tentativas de Bernoulli, mas a confiança usa
   `success + partial·0.5` + pesos fixos — sem posterior nem intervalo.
5. **Dedup lexical O(N²)-ish.** Dice sobre conjuntos de termos; sem *blocking* nem LSH.
6. **Clusters por eixo, não por estrutura.** Agrupa por `anchor`/`type`/`classification`/`scope`
   — não por **comunidade** do grafo.
7. **Grafo subaproveitado.** `depends_on`/`impact`/ciclos; **sem** centralidade, PageRank,
   propagação de relevância nem detecção de comunidade.
8. **Contradição inerte.** `contradicts` existe (D158 sugere), mas não rebaixa nem deprecia
   (E16/T07).
9. **Retenção por prazo fixo.** Shelf-life = `created + TTL`; sem curva de esquecimento nem
   revisão espaçada.
10. **Busca sem reranking.** Retorna a fusão direto; sem *cross-encoder* nem expansão de consulta.
11. **Vetores brute-force.** Cosseno sobre todos; sem ANN/quantização/Matryoshka.
12. **Tarefa sem tempo.** D135 tirou o tempo; sem *flow metrics* (cycle/lead time) nem caminho
    crítico — apesar do log de eventos ter os instantes.
13. **Superfície larga.** 14 verbos; `knowledge`/`maintenance`/`self` com subcomandos.
14. **Obrigatoriedades mínimas.** Só `id/statement/created_at/body_hash/schema_version`; nenhum
    tipo exige *slot* próprio.

---

## 3. Gestão de dados moderna (o que emprestar)

| Técnica | O que dá | Onde | Custo |
|---|---|---|---|
| **Bitemporal** (valid-time + transaction-time) | "o que se acreditava em T" com precisão | `schema` + `retrieval/temporal` | alto (contrato) |
| **Merkle-DAG de revisões** | diff/merge de 3 vias, histórico ramificado | `write/update` | médio |
| **CRDT (G-Set / LWW-Register)** | sync sem conflito em `outcomes`/`tags`/`anchors` | `sync`, `write` | médio |
| **Proveniência W3C PROV-lite** | `entity/activity/agent` por afirmação | `schema` | médio |
| **Content-addressed + CAS** | já existe (ID por hash); falta *store* de blobs | `store` | baixo |
| **Data contract / schema registry** | `schema_version` + validação por tipo | `schema` | baixo |
| **Lineage** | grafo de dependências do dado | `graph`, `events` | baixo |
| **Tabela imutável + views** | já é (notas + derivados) | — | — |

**Adotar primeiro (baixo custo, alto valor):** data contract por tipo (§7), CRDT leve para
`outcomes` (§4), lineage via `events` (§7).

---

## 4. Representação de conhecimento

1. **Claims tipados (SPO + qualificadores).** Cada nota pode declarar `(sujeito, relação,
   objeto)` além do `statement` — sem substituir a afirmação. Habilita inferência, contradição
   precisa e dedup semântica. **Onda R5.**
2. **Ontologia leve (SKOS/OWL-lite).** Acrescentar `broader`/`narrower`/`related`/`same_as` ao
   vocabulário de arestas (D51). `same_as` resolve **identidade** (entity resolution) e
   `broader/narrower` dão **hierarquia** de conceitos (hoje só `extends`). **Onda R4.**
3. **Truth Maintenance System (TMS/ATMS).** Rastrear de quais **suposições** cada crença
   depende; retratar uma suposição invalida os dependentes. Casa com `contradicts` e com a
   depreciação (E16/T07). **Onda R6.**
4. **Raciocínio derrotável (defeasible).** Conhecimento novo **derrota** o antigo sem apagar
   (já há `replaces`; falta a semântica de derrota). **Onda R6.**
5. **Resolução de entidades.** Dedup atual é lexical; moderno é *blocking* + similaridade
   (MinHash/LSH + embeddings). **Onda R3.**

---

## 5. Matemática pura aplicada

### R1 — Confiança bayesiana (Beta-Bernoulli) ★
`outcomes` são **ensaios de Bernoulli**. Modelar a confiabilidade como posterior
`Beta(α + Σs, β + Σf)` (parcial = 0.5). Usar a **média posterior** para o `stars` e o **limite
inferior do intervalo de credibilidade** (ex.: 95%) como confiança conservadora — uma nota com
1 sucesso não empata com uma com 20. Substitui a heurística de `confidence.rs`; puro,
determinístico, **sem dep**. Revisa D87. **Absorve E16/D174** (`drift`/`feedback` como insumos).
> **Impacto:** alto · **Esforço:** baixo · **Contrato:** nenhum byte muda (derivado).

### R2 — Retenção por curva de esquecimento + revisão espaçada (FSRS-like) ★
Trocar o TTL plano de `shelf_life` por **retenção** `R(t) = exp(-t/S)`, com **estabilidade `S`**
crescendo a cada `outcome` de sucesso (modelo de *spaced repetition* — FSRS/Ebbinghaus). O
`prune` passa a propor revisão quando `R < limiar`, não quando o prazo venceu. Substitui
`age_factor`/shelf-life por uma função única e **calibrável**. Revisa D44/D154. **Substitui
E16/D178.**
> **Impacto:** alto (depreciação dirigida por uso) · **Esforço:** médio · **Contrato:** derivado.

### R3 — PageRank / Personalized PageRank ★
Autoridade a partir do grafo (`references`/`supports`/`extends`/`replaces`); **PPR** semeado pelo
*working set* (âncoras) propaga relevância pela vizinhança — generaliza o canal de âncoras e o
`knowledge rank`. Iteração de potência com tolerância fixa ⇒ **determinístico**. **Soma** ao canal
de âncoras (não o substitui). Revisa D81/D107.
> **Impacto:** alto · **Esforço:** médio · **Contrato:** derivado (ranking).

### R4 — Comunidades (Louvain/Leiden)
Substituir o agrupamento por eixo (`clusters.rs`) por **detecção de comunidade** no grafo de
notas (arestas + âncoras + vizinhança vetorial). Variante determinística (ordem canônica + seed
fixa). Alimenta `knowledge map` e o GraphRAG (§6). Revisa D47/E10-T06.
> **Impacto:** médio-alto · **Esforço:** médio.

### R5 — MinHash + LSH para dedup
Trocar o Dice exato por **assinaturas MinHash** e *blocking* LSH: O(N) na prática, robusto a
paráfrase e a ordem; casa com o `dedup` em duas fases (D26/D80). Mantém o fallback lexical para
corpora pequenos.
> **Impacto:** médio (escala) · **Esforço:** médio.

### R6 — Detecção de drift (KL/JS)
Comparar a distribuição de termos/embeddings de um tópico **hoje** × **no passado** (do log de
eventos) por **divergência KL/JS**; drift alto ⇒ conhecimento provavelmente obsoleto ⇒ candidato
a `prune`/revisão. Casa com R2 e com o decay.
> **Impacto:** médio · **Esforço:** médio.

### R7 — Métricas de fluxo e caminho crítico (PERT/CPM)
Do log de eventos derivar **cycle time** (`close − create`), **lead time**, **throughput** e o
**caminho crítico** do DAG de `depends_on` (com estimativas opcionais). `impact` já é a base.
> **Impacto:** médio (task) · **Esforço:** baixo-médio.

### R8 — Ganho de informação / IDF estendido
Usar **informação mútua** para relacionar termos e **ganho de informação** para ranquear por
"quanto o termo distingue" — refinamento do IDF por campo, já presente.
> **Impacto:** baixo · **Esforço:** baixo.

### R9 — Embedding espectral do grafo (Laplacian eigenmaps)
Embutir o grafo de conhecimento no mesmo espaço dos vetores (Laplaciano normalizado → autovetores)
para *link prediction* e vizinhança estrutural — complementa `learn`/`suggest`.
> **Impacto:** experimental · **Esforço:** alto.

### R10 — Análise formal de conceitos (FCA)
Do reticulado `notas × tags/atributos`, derivar **conceitos formais** (fechos de Galois) para um
mapa conceitual do corpus. Muito "puro"; entrega `knowledge map` alternativo.
> **Impacto:** experimental · **Esforço:** alto.

---

## 6. Vetores modernos

1. **Reranking (cross-encoder).** Recuperar top-50 (lexical+vetorial) e **reranquear** com um
   *cross-encoder* pequeno servido pelo llama.cpp local. É o maior salto de qualidade de `ask`.
   **Onda R2.**
2. **Expansão de consulta (HyDE / multi-query).** Gerar uma resposta hipotética (LLM local) ou
   sinônimos e embutir isso — melhora recall semântico. **Onda R2** (opcional, zero-LLM por
   padrão).
3. **Matryoshka (MRL) + quantização.** Truncar dimensões (384→128) e/ou quantizar (int8/binário)
   para ANN rápido; o modelo granite suporta MRL. **Onda R3.**
4. **ANN (HNSW/IVF-PQ).** Para corpora grandes; brute-force é aceitável até ~10⁴ notas. **Onda R3**
   (só quando doer).
5. **GraphRAG.** Comunidades (R4) + resumos locais/globais para responder perguntas de *visão
   geral* ("quais são os temas do projeto?"). Casa com `knowledge map` materializado (D150).
   **Onda R4.**
6. **Fusão calibrada.** RRF → normalização de score por canal + combinação convexa (E16/T09).
7. **Late interaction (ColBERT-lite).** *MaxSim* por token — recall forte para consultas longas;
   experimental. **Onda R6.**

---

## 7. `write`, `task` e superfície

### Obrigatoriedades por tipo (data contract)
Adicionar **slots mínimos** por espécie (novas chaves canônicas ou seções de corpo validadas):

| Tipo | Obrigatório (proposto) |
|---|---|
| `decision` | `alternatives` (≥1 rejeitada) + corpo com `Por quê:`/`Consequência:` |
| `error` | `cause` + `fix` (seções do corpo) |
| `risk` | probabilidade/impacto (ou `evidence`) |
| `def` | termo canônico (`statement`) + `aliases` |
| `snippet` | linguagem + âncora |
| `question` | `depends_on`/âncora (o que falta) |
| `fact` | ≥1 âncora **ou** ≥1 aresta (lastro) |
| `task` | `checks`/`evidence` no fecho |

Isso realiza "adição de obrigatoriedades" e o incentivo ao corpo (D162) de forma **verificável**
por `doctor`/validators (D156).

### Superfície (redução)
- **`knowledge`** dobra em **`ask`** (rank/tags como modos) ou vira `kd map`? — decidir (hoje
  `ask` já é "toda a pesquisa", D146).
- **`forget`/`restore`** **permanecem verbos** (o `forget` é a aplicação do `prune`, D112).
- **`sync`/`config`/`self`** são meta: manter, mas com poucos subcomandos (E18 consolida `self`).
- **`maintenance`** só revisão (E18); **`drain service`** para o worker (E18).
- Meta: **≤ 10 verbos**, um vocabulário por conceito.

### Inferência e extração
- `write` **propõe** arestas (regex/embeddings) e **bloqueia** duplicatas (MinHash).
- `task` deriva **flow metrics** (R7) e **caminho crítico**.
- `ask` usa **PPR** (R3) + **reranking** (R2).

---

## 8. Roteiro em ondas (impacto × esforço × contrato)

| Onda | Conteúdo | Impacto | Esforço | Toca contrato? |
|---|---|---|---|---|
| **R1 — Base** | R1 (Beta) + R2 (retenção) + obrigatoriedades por tipo (§7) | alto | baixo-médio | não (derivado) / leve (validators) |
| **R2 — Busca** | Reranking + expansão de consulta + fusão calibrada (E16/T09) | alto | médio | não |
| **R3 — Escala** | MinHash/LSH (R5) + Matryoshka/ANN (§6) + resolução de entidades | médio | médio | não |
| **R4 — Grafo** | PageRank/PPR (R3) + comunidades (R4) + GraphRAG | alto | médio | não |
| **R5 — Ontologia** | Claims SPO + `same_as`/`broader`/`narrower` + proveniência | alto | alto | **sim** (schema_version) |
| **R6 — Razão** | TMS/defeasible (R6) + drift KL/JS (R6) + late interaction | médio | alto | parcial |
| **R7 — Tarefa** | Flow metrics + caminho crítico (R7) + superfície enxuta (§7) | médio | baixo-médio | leve |

**Ordem recomendada:** R1 → R4 (maior razão ganho/risco, sem tocar bytes) → R2 → R3 → R5 → R6 → R7.
**Versão:** tudo em **0.5.0**; R5 (T09) bumpou `schema_version` 1→2 na mesma release (aditivo).
As únicas pendências são R2/T06 (reranking) e R3/T08 (Matryoshka/ANN), dependentes de 2º
modelo/escala — ver [`reranking_ann.md`](reranking_ann.md).

---

## 9. Riscos e não-objetivos

| Risco | Mitigação |
|---|---|
| R5 (ontologia/SPO) quebra o contrato TOON | `schema_version` + rebuild (D14/D15); só na onda dedicada |
| Matemática "exótica" (FCA/espectral) sem ganho | marcar **experimental**; adotar só com A/B (E13-T09) |
| Beta/FSRS mal calibrados | parâmetros em config + defaults conservadores; A/B |
| Reranking exige modelo | local via llama.cpp; **opcional**; degrada para RRF (R33) |
| Superfície mexida quebra usuários | D14; `--help`/matriz/CHANGELOG no mesmo commit |
| Determinismo quebrado por ANN/LSH | seeds fixas; ordem canônica; fallback exato |

**Não-objetivos:** `criterion` como gate; dep nova sem ganho ≥20% (R43); LLM obrigatório; servidor
obrigatório (R16); migração de corpus (D14); perder o contrato de bytes.

---

## 10. Pontos em aberto (aguardando indicação do usuário)

- _(a preencher)_

---

## 11. Performance (herança de E15)

> O orçamento completo está no épico
> [`../implementation/23_modelo_conhecimento_rico.md`](../implementation/23_modelo_conhecimento_rico.md)
> §"Performance e orçamento".

- **Orçamento:** micro de `score`/`tokenize`/`Note::parse` ≤ +20 %; e2e `--no-idle` `ask` ≤ +25 %
  por onda (≤ +40 % no épico), **com** Recall@k/nDCG ≥ baseline. Reranking (T06) reranqueia
  **top-K** (20–50), opcional, degradável para RRF.
- **Determinismo:** PPR com tolerância/nº fixo; LSH/ANN com seed fixa; brute-force segue default.
