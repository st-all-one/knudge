# Qualidade da busca e depreciação de conhecimento — plano de implementação

> **Status:** proposta (não implementada). Épico executável em
> [`../implementation/20_qualidade_busca_depreciacao.md`](../implementation/20_qualidade_busca_depreciacao.md).
>
> **Objetivo:** elevar a **qualidade da busca** (`ask`/`knowledge`) em PT-BR e tornar a
> **depreciação de conhecimento** dirigida por **evidência** — sem tocar o contrato de bytes de
> `notas/` (o índice é derivado, D15/D27/D84). Nenhuma chave TOON muda; o contrato `--json` só
> ganha campos aditivos.
>
> **Versão alvo:** **0.5.0** (muda *resultado* de busca, não bytes de `notas/`; ver §Riscos).

Temas: (1) tokenização PT; (2) avaliação de qualidade; (3) confiança derivada; (4) depreciação
por evidência; (5) fusão de canais; (6) consistência entre consumidores.

---

## 0. Decisões propostas (numeração provisória)

| Id | Decisão |
|---|---|
| **D172** | **Normalização de acentos na tokenização de retrieval.** O tokenizador passa a dobrar acentos (NFD + remoção de marcas combinantes) antes de aplicar a regra ASCII; o `normalize` do schema (que alimenta `id`/`body_hash`) **não muda**. Revisa **D36**. |
| **D173** | **Termos de alta frequência.** `STOPWORDS` ganha as formas dobradas do PT (`ja`, `so`, `ate`, `apos`, `entao`, `tambem`, `nao`, `sao`, …) e o canal lexical descarta termos com `df/N` acima de um limiar configurável. Revisa **D122**. |
| ~~**D174**~~ | **Absorvida por E19/D189** — a confiança derivada (`drift`/`feedback`) entra como insumo do posterior Beta. Ver [`revisao_integrada.md`](revisao_integrada.md) §3 (C1). |
| **D175** | **Idade no ranking sem query.** `knowledge rank` passa a penalizar idade de forma **aditiva** (hoje `similarity=0` anula o `age_factor`). Revisa **D107/D146**. |
| **D176** | **Consistência de status.** Todo consumidor de corpus (`knowledge rank`/`map`, `rewind`, `learn`…) exclui `forgotten`/`superseded` por padrão — como `ask` (D43), `tags` (D43), `dedup` (D43) e `body_check` já fazem. Revisa **D43/D143**. |
| **D177** | **`contradicts` no ranking e na depreciação.** Aresta declarada rebaixa/avisa o lado perdedor no `recall`/`rank` e entra no plano de `prune` como candidato (com a confiança derivada como desempate). Revisa **D45/D158**. |
| ~~**D178**~~ | **Substituída por E19/D190** — a retenção vira curva de esquecimento (FSRS). Ver [`revisao_integrada.md`](revisao_integrada.md) §3 (C2). |
| **D179** | **Fusão recalibrada.** `rrf_k` e pesos dos canais (`lexical`/`anchor`/`semantic`) são escolhidos por **medição de qualidade** na bancada, não por heurística herdada. Revisa **D81/D123/D151**. |

> **Stemming PT (A2) fica condicional.** Só vira decisão (novo `Dxx`) se a bancada de qualidade
> (T01) provar ganho ≥ 20 % sobre D172/D173; caso contrário, é registrado como **rejeição
> medida** (como T12 fez com as dependências).

> **Avaliação (F1)** não é decisão de produto: implementa o intento de **D145** (a avaliação vive
> na bancada externa `bench/`, não em `kd`).

---

## 1. Achados (evidência no código)

### 1.1 Tokenização ASCII — o maior gargalo em PT-BR (→ D172/D173)

`retrieval/token.rs`: `is_word` aceita só `[A-Za-z0-9_]`; qualquer byte `≥ 0x80` é separador.
`content_terms` ainda descarta tokens de 1 caractere. Efeito no corpus PT-BR:

| Entrada | Indexado como | Consequência |
|---|---|---|
| `café` | `caf` | `café` ≠ `cafe` (não casam) |
| `são`, `não` | `s` + `o` → **descartados** | palavra **invisível** ao BM25 |
| `configuração` | `configura` | ≠ `configuracao` |
| `método` | `todo` | **colide com o inglês `todo`** |
| `código` | `digo` | fragmento ambíguo |

Palavras funcionais comuns do PT (`não`, `são`, `é`, `já`, `só`, `após`, `então`, `até`,
`também`) simplesmente não existem para o índice lexical. É o item de maior impacto.

**Correção proposta (D172).** Dobrar acentos **no tokenizador de retrieval**, sem tocar o
`normalize` do schema:

```rust
pub fn tokenize(input: &str) -> Vec<Cow<'_, str>> {
    if input.is_ascii() { return tokenize_ascii(input); } // fast-path atual, zero-custo
    let folded: String = input.nfd().filter(|c| !is_combining_mark(*c)).collect();
    tokenize_ascii(&folded).into_iter().map(|t| Cow::Owned(t.into_owned())).collect()
}
```

`unicode-normalization` já é dependência (`schema::body`), então **não entra dep nova**. O
fast-path ASCII preserva o custo atual para identificadores/termos em inglês. `café→cafe`,
`são→sao`, `configuração→configuracao`, `método→metodo`.

**Borda:** o índice `.idx/` é derivado, então `notas/` não muda — mas o **dedup muda**
(`dice` passa a casar `café`/`cafe`), exigindo golden de `write` e medição. `id`/`body_hash`
seguem sensíveis a acento (contrato do schema preservado).

**Follow-up (D173).** Com o fold, formas como `ja`/`so`/`ate`/`apos`/`entao`/`tambem` viram
termos de conteúdo e recriam os votos espúrios que D122 corrigiu. Duas alavancas:
1. `STOPWORDS` ganha as formas **dobradas** das palavras funcionais PT;
2. **corte dinâmico** por `df/N` em `content_terms` (independente de idioma) — o IDF já atenua
   (`idf_from` ≈ `0.5/N` para `df=N`), mas o **RRF usa ranks**, então o corte no canal lexical é
   o que evita o rank-1 espúrio.

### 1.2 Confiança derivada com termos mortos (→ D174/D175)

`lifecycle/confidence.rs`:
```
base   = similarity · drift_factor(drift) · age_factor(age)
conf   = base + 0.2·(confirmation + feedback) + task_confirmation
```
Mas `drift` e `feedback` **nunca são preenchidos em produção** (só o `Default`; `grep` confirma
zero atribuições fora de testes). Logo `drift_factor ≡ 1.0` e `feedback ≡ 0.0`: D87 promete
"evidência + feedback + idade + drift de âncoras", mas **drift e feedback são código morto**.

- `compute_anchor_validity` existe (`decay.rs`) e é chamada **só** no `prune`
  (`maintenance/extra.rs`), **não é persistida** → o `recall` não tem `drift` barato.
- `confirmation` só soma `success + partial·0.5`; `failure`/contradição **não** entram em
  `feedback`.

**Correção (D174):** persistir a validade de âncoras como **derivado** (`.idx/`, off-path,
purgável por D84) e alimentar `drift` em `recall`/`rank`; derivar `feedback` dos `outcomes`
negativos. Sem isso, a confiança é só `similarity·age + 0.2·confirmation`.

**Correção (D175):** em `rank.rs`, `similarity = 0` → `base = 0` → o `age_factor` é anulado.
Então `knowledge rank` ("as mais confiáveis") **não penaliza idade**. Passar a idade a somar de
forma **aditiva** (ex.: `base = age_factor(age)` quando não há similaridade).

### 1.3 Consistência entre consumidores (→ D176)

`CorpusScope::select` monta `Filter` com `statuses` **vazio**, e `rank()`/
`structural_clusters_filtered()` não filtram status. Já `ask` (`resolve_statuses`), `tag_counts`
(D43), `propose_merges`/`live_ids` (D43) e `body_check` **excluem** `Forgotten | Superseded`.
Resultado: **`kd knowledge rank`/`map` ressuscitam conhecimento apagado/superseded** no ranking
e nos clusters. É o achado mais acionável e barato de corrigir.

### 1.4 `contradicts` inerte (→ D177)

Existe `EdgeKind::Contradicts` e `knowledge suggest` (D158) propõe contradições, mas uma aresta
**declarada** não rebaixa a nota no ranking nem entra em `demotion_candidates` (que só olha
shelf-life + decay). Conhecimento contraditório coexiste e ambos rankeiam.

### 1.5 Shelf-life puramente temporal (→ D178)

`shelf_life.rs`: `foundational=∞`, `tactical=365d`, `observational=30d`, `renew_on_use=false`.
Uma `observational` com vários `success` expira em 30 dias do mesmo jeito; uma `tactical`
recém-confirmada expira em 365. O decay de âncoras só alcança notas **ancoradas**
(`fraction()==1.0` quando `total==0`) → notas sem âncora só morrem por tempo.

### 1.6 Fusão comprimida (→ D179)

`rrf.rs`: `score = Σ peso/(k+rank+1)` com `k=60`. Num corpus de 10²–10³ docs, rank 1 vs rank 10
diferem ~13 %; a fusão fica quase uniforme — por isso o default `semantic_weight = 30` (D123)
precisa ser tão alto para o vetor dominar (o próprio código admite). `k=60` foi desenhado para
web (10⁵–10⁶ docs). Alternativa a medir: `k` pequeno e/ou **normalização de score por canal**.

### 1.7 Sem avaliação de qualidade (→ T01/F1)

D145 removeu `maintenance eval` (era stub) e moveu a avaliação para `bench/` — mas a bancada mede
**latência**, não qualidade (nenhum `nDCG`/`MRR`/`Recall@k` em `bench/src`). Sem isso, nenhuma
proposta de D172–D179 pode ser validada pelo guardrail "se não move o ponteiro, reverte".

### 1.8 Performance (secundário)

- `body_share` (`bm25.rs`) re-tokeniza a consulta **por hit** em `build_hits`.
- `validity_map` (`maintenance/extra.rs`) chama `compute_anchor_validity` **por nota**, e cada
  chamada faz um **walk completo** do projeto → `prune` é `O(notas × tamanho_do_projeto)`.
- `rank()` chama `from_tasks_with` para **todo** doc → `O(N × T × A²)`.

---

## 2. Plano de medição (T01)

A bancada `bench/` ganha um modo **`quality`** (zero-dep, fora do workspace, como o resto):
um conjunto rotulado `consulta → ids esperados` e as métricas **Recall@k**, **MRR** e **nDCG@k**.
Fontes do conjunto:

1. **Derivado do corpus** (determinístico): para cada nota, o `statement` é a consulta e o
   próprio `id` é o esperado (sanity check de recall); variações com remoção de stopwords e
   acentos medem o efeito de D172/D173.
2. **Sintético controlado**: pares sinônimo↔nota (mede o canal vetorial) e paráfrase↔nota (mede
   o lexical + fold).
3. **Amostra manual** (opcional): ~20 consultas reais do projeto, rotuladas à mão.

O modo é **observação**, nunca gate (`make check` inalterado), coerente com E13-T09.

---

## 3. Riscos e mitigação

| Risco | Mitigação |
|---|---|
| D172 muda resultado de busca e dedup | `notas/` intacto; golden de retrieval/`write` regenerado com intenção; `DIVERGENCES.md` ganha linha; A/B na bancada de qualidade |
| Fold de acentos colide conceitos (`café`≡`cafe`) | decisão explícita em D172; se indesejado, indexar **ambos** (token cru + dobrado) — medir antes |
| D173 agressivo demais (perde recall) | corte por `df/N` calibrado na bancada; default conservador; config por projeto |
| D174 persistir validade de âncoras adiciona derivado | 100 % reconstruível e purgável (D84); off-path; warning se ilegível |
| D175/D176 mudam ranking/consumidores | goldens de `knowledge rank`/`map`; testes de regressão por consumidor |
| D177/D178 rebaixam/aposentam demais | piso de confiança; grace period; `prune` só **propõe** (D112) |
| D179 recalibra defaults | medição explícita; defaults antigos continuam acessíveis por config |
| stemming PT over-stemming | **condicional** e medido; rejeitar sem hesitar |

---

## 4. Não-objetivos

- `criterion` como gate (E13-T09/R43).
- Dep nova sem ganho ≥ 20 % medido (R43); `HashMap`/`HashSet`/`rayon` (determinismo).
- Mudar o `normalize` do schema ou qualquer byte de `notas/` (contrato D06/D95).
- Migração de corpus (D14).
- Busca fuzzy/typo-tolerant de propósito geral (fora de escopo; o fold cobre acentos).

---

## 5. Performance (herança de E15)

> O orçamento completo e as notas `**Perf:**` por tarefa estão no épico
> [`../implementation/20_qualidade_busca_depreciacao.md`](../implementation/20_qualidade_busca_depreciacao.md)
> §"Performance e orçamento".

- **Orçamento:** micro de `tokenize`/`content_terms`/`score` ≤ +20 %; e2e `--no-idle`
  `ask`/`rewind` ≤ +15 % por tarefa (≤ +25 % acumulado), **com** ganho de Recall@k/MRR/nDCG
  medido em T01.
- **Regra:** mudança de busca que regride sem ganho de qualidade é **revertida**; o
  `Corpus`/`Postings` já carregados são **reusados** (sem novo walk do disco);
  `unicode-normalization` é dep existente (D172).
