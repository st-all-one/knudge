# Matemática do knudge

Derivação dos modelos quantitativos usados na busca, no grafo, no ciclo de vida, no dedup e no
fluxo. A maioria é da 0.5.0 (E16–E19). O objetivo é documentar **de onde vêm** as fórmulas e as
constantes, não só os valores.

- Código: `retrieval/`, `graph/`, `lifecycle/`, `write/dedup/`, `task/flow.rs`
- Decisões: D35–D38, D81, D123, D173, D179, D189, D190, D192, D193, D204, D205, D208
- Bordas numéricas: [`DIVERGENCES.md`](DIVERGENCES.md) (#101, #103, #105, #107, #109, #111)

## Convenções

- Tudo é `f64` determinístico; **nunca** há RNG. Ordem de iteração sempre canônica
  (`BTreeMap`/`BTreeSet`) — a aritmética de ponto flutuante é a mesma em qualquer execução.
- Comparações usam `total_cmp` (ordem total IEEE-754), não `partial_cmp`.
- Onde o domínio é limitado, há `clamp` explícito (`clamp01`); `NaN` é tratado como `0`.
- Os valores são derivados; nada é persistido (D87).

---

## 1. BM25 (D35–D38, D173)

Score lexical de um documento `d` para os termos `q` da consulta, somado por **campo** `f`
(`statement`/`body`/`tags`):

```
score(d, q) = type_weight(type_d) · (1 + 0.1·confirmação_d)
            · Σ_f  w_f · Σ_{t∈q} IDF_f(t) · tf_f(t,d)·(k1+1)
                                           ─────────────────────
                                             tf_f(t,d) + k1·norm_f(d)
```

- `k1 = 1.5`, `b = 0.75`.
- **Normalização por comprimento:** `norm_f(d) = 1 − b + b·len_f(d)/avg_len_f`.
- **Peso de campo** `w_f`: `statement = 3.0`, `tags = 2.0`, `body = 1.0` (o `statement` domina — D37).
- **IDF de Robertson-Sparck Jones** (com suavização), por campo:

```
IDF_f(t) = ln(1 + (N − df_f(t) + 0.5) / (df_f(t) + 0.5))
```

  (`ln_1p` no código). O `IDF` **por campo** faz `statement` e `body` terem relevâncias distintas.
- **Peso por tipo** `type_weight` (D37): `decision 1.20`, `error 1.15`, `fact 1.10`,
  `def/risk 1.05`, `task/question 1.00`, `snippet 0.95`, `link/meta 0.90`, `epic 0.70`.
- **Boost por confirmação** (D38): `1 + CONFIRMATION_STEP·confirmação`, `CONFIRMATION_STEP = 0.1`.
  A confirmação é derivada de `outcomes` (`success + 0.5·partial`) e das tarefas (D108).
- **Corte de alta frequência** (D173): um termo `t` é descartado quando
  `df(t)/N ≥ recall.max_term_ratio` (default `0.9`), **apenas** se `N ≥ MIN_CUTOFF_CORPUS (64)`.
  Justificativa: o RRF usa *ranks*; um termo quase universal produziria um rank-1 espúrio mesmo
  com IDF pequeno. Abaixo de 64 notas `df/N` é alto para quase todo termo, então o corte é no-op.
- `df(t)` usa o **maior** `df` entre os campos (`max`), conservador.

**Idempotência do scoring:** o conjunto `terms` é deduplicado e preserva a ordem de entrada; o
score é soma, então a ordem não afeta o resultado (mas afeta o desempate, que é `(score desc, id asc)`).

---

## 2. Reciprocal Rank Fusion (D81/D123/D179)

Fusão dos canais `c` (lexical, âncoras, vetorial, PPR). Para um id `x` com rank `r_c(x)` no canal
`c` (0-based):

```
RRF(x) = Σ_c  peso_c / (k + r_c(x) + 1)
```

- `k = 60` (`recall.rrf_k`), medido **inerte** no corpus rotulado (D179).
- Pesos default (`FusionWeights`): `lexical = 1.0`, `anchor = 2.0`, `semantic = 30.0`,
  `ppr = 0.0` (D123/D179/D192).
- **Por que peso:** sem peso, um canal com muitos votos (lexical) dilui o vetorial. `semantic = 30`
  **Pareto-domina** o neutro no corpus PT-BR; `anchor = 2.0` corrige o empate entre um match exato
  de âncora e um rank-1 lexical ruidoso (D179).
- **Degradação:** a soma só inclui canais presentes; canal ausente **não quebra** (R33).
- **Desempate determinístico:** `(score desc, id asc)`.
- `channels` conta quantos canais contribuíram; `contribs` guarda a parcela de cada um (D151).

---

## 3. Confiança derivada: Beta-Bernoulli + Wilson (D87/D189/D203)

A confiança é calculada em tempo de consulta e nunca armazenada.

### 3.1 Ensaios de Bernoulli

Cada `outcome` vira um ensaio: `success = 1`, `partial = 0.5`, `failure/abandoned = 1 falha`.
Sejam `s` sucessos e `f` falhas (podem ser fracionários).

### 3.2 Posterior Beta

Com prior uniforme `Beta(1, 1)`, o posterior é `Beta(1+s, 1+f)`. A **média** posterior:

```
μ = (1 + s) / (2 + s + f)              (0 se s = f = 0)
```

### 3.3 Limite inferior de Wilson (95 %)

O intervalo de credibilidade de Wilson para uma proporção `p = s/(s+f)` com `n = s+f` e
`z = Z_95 = 1.96` é:

```
p + z²/(2n) ± z·√( p(1−p)/n + z²/(4n²) )
─────────────────────────────────────────
              1 + z²/n
```

O knudge usa o **limite inferior** (o lado `−`), `clamp`ado a `[0,1]`. Por que conservador: uma
nota com **1** sucesso recebe ≈ `0,21` e **não empata** com uma com **20** sucessos (≈ `0,84`) —
o limite inferior cresce com `n`, mas penaliza pouca evidência. É `O(1)` e sem dependência.

> O intervalo é chamado "credibilidade" na doc do código porque vem do posterior, mas a álgebra
> é a de Wilson (frequencista) — as duas leituras coincidem numericamente aqui.

### 3.4 Composição do score

```
base      = similaridade · age_factor(idade)
evidência = lower_bound(s, f)
feedback  = 0.2 · max(feedback_explícito, 0)
tarefa    = task_confirmation                       (D108, já ponderada)
recência  = AGE_WEIGHT · age_factor(idade) · (1 − similaridade)

score = clamp01( (base + evidência + feedback + tarefa + recência) · drift_factor(drift) )
```

- **Fator de idade:** `age_factor(a) = 1 / (1 + a/90)` — meia-vida de 90 dias (`AGE_HALF_LIFE_DAYS`).
- **Recência aditiva** (D175): `AGE_WEIGHT = 0.05`, só quando `similaridade = 0` (o `rank`), onde
  o termo multiplicativo `base` sumiria. É ≤ 0,05 e **só desempata** — a evidência domina.
- **Fator de drift:** `drift_factor(d) = 1 − 0.5·d` (`DRIFT_FLOOR = 0.5`), em `[0.5, 1]`.
  Desconta o score **inteiro** (D203) — uma nota com âncoras quebradas perde confiança mesmo sem
  query.
- **Penalidade de contradição** (D177): `max(confidence − 0.1, 0)` para o lado perdedor.

**Monotonicidade** (proptest): crescente em `similaridade`/`sucessos`/`feedback`, decrescente em
`falhas`/`drift`/`idade`.

---

## 4. Retenção FSRS-like (D190)

O shelf-life deixa de ser prazo fixo. A retenção decai exponencialmente com o tempo desde a
origem; a estabilidade `S` cresce a cada sucesso.

### 4.1 Curva

```
R(t) = limiar^(t/ttl) = exp(−t/S),     S = ttl / ln(1/limiar)
```

- `limiar = DEFAULT_REVIEW_THRESHOLD = 0.5`; `ttl` em dias.
- O prazo é o cruzamento `R = limiar` (por construção, `t = ttl`).
- `ttl ≤ 0` ⇒ `R = 1` (nunca decai — `foundational`).

### 4.2 Crescimento por revisão

Cada `outcome` de **sucesso** estende o prazo (D190):

```
ttl_efetivo = base · (1 + growth% · reviews / 100)
```

com `growth% = 50` (`retention.growth_percent`). Ou seja, `reviews` sucessos somam 50 % da base
cada. A **origem** é resetada: `origin = max(created_at, último ensaio, último uso se renew_on_use)`.

### 4.3 Estabilidade

`stability_days(ttl, limiar) = ttl / ln(1/limiar)` converte o prazo para a constante de decaimento
`S`; com `limiar = 0.5`, `S = ttl / ln 2 ≈ 1.4427·ttl`.

---

## 5. Drift de âncoras (D43/D86/D203)

Sejam `V` âncoras válidas e `B` quebradas:

```
fração_válida = V / (V + B)
drift         = 1 − fração_válida          ∈ [0,1]
drift_factor  = 1 − 0.5·drift              ∈ [0.5,1]
```

`prune` persiste `drift` em `.idx/drift.jsonl`; `ask`/`rank` carregam e aplicam. Arquivo ausente
⇒ `drift = 0`. Demove após *grace* se `fração_válida < threshold` (D43).

---

## 6. Drift de termos: KL e Jensen-Shannon (D208)

Para um **tópico** (âncora), separa as notas pela **mediana** de `created_at` em janela
**antiga** e **nova**; compara as distribuições de termos.

### 6.1 Distribuição

`P(t) = count(t) / Σ_t' count(t')`, sobre `content_terms` (tokenização com fold + stem).

### 6.2 Divergência KL (base 2)

```
KL(P‖Q) = Σ_t  P(t)·log2( P(t) / Q(t) )
```

Termos ausentes em `Q` usam `SMOOTHING = 1e-9` (Laplace) para evitar `log(0)`. Base 2 ⇒ `KL` em
bits; **assimétrica** e ilimitada.

### 6.3 Divergência Jensen-Shannon (base 2)

```
M      = (P + Q)/2
JS(P,Q)= ½·KL(P‖M) + ½·KL(Q‖M)          ∈ [0,1]
```

Simétrica, limitada a 1 bit (base 2). É a métrica usada por `topic_drift`.

### 6.4 Limiar

Propõe revisão quando `JS ≥ DRIFT_THRESHOLD = 0.5` **e** o tópico tem `≥ MIN_DRIFT_NOTES = 4`
notas (evita medir ruído). O `prune` só **propõe** (D112).

---

## 7. PageRank e Personalized PageRank (D192)

Grafo **dirigido** de autoridade (arestas `references`/`supports`/`extends`/`replaces`), sem
duplicatas de alvo por nó.

### 7.1 Iteração de potência

```
r_{k+1}(v) = (1−d)·p(v) + d·Σ_{u→v} r_k(u)/outdeg(u) + d·D·p(v)
```

- `d = 0.85` (`DAMPING`), `r_0(v) = 1/n`.
- `p(v)` é o **vetor de personalização** (distribuição, soma 1): uniforme (`1/n`) para o
  PageRank global, ou `1/|seeds|` nas sementes (PPR semeado pelo working set).
- `D = Σ_{u: outdeg(u)=0} r_k(u)` é a massa **dangling** (nós sem saída), redistribuída por `p` —
  por isso a soma permanece 1.
- **Convergência:** ≤ `MAX_ITERATIONS = 32`, para quando a norma L1
  `Σ_v |r_{k+1}(v) − r_k(v)| < TOLERANCE = 1e-8`.
- **Determinismo:** ids em ordem canônica (`BTreeMap`); sem RNG. Invariante a relabeling (proptest).

### 7.2 Uso

Vira o canal `ppr` da fusão RRF (`recall.ppr_weight`, default **0.0** = desligado). Só compensa em
corpora com muitas arestas de autoridade.

---

## 8. Louvain determinístico (D193)

Grafo **não-dirigido ponderado**; `w_ij = w_ji`, `k_i = Σ_j w_ij` grau ponderado, `2m = Σ_i k_i`.

### 8.1 Modularidade e ganho

A modularidade de uma partição é `Q = (1/2m)·Σ_ij [w_ij − k_i·k_j/(2m)]·δ(c_i,c_j)`. Mover um nó
`i` para a comunidade `c` altera `Q` proporcionalmente a:

```
ΔQ_{i→c}  ∝  w_{i,c} − k_i · Σ_tot(c) / (2m)
```

onde `w_{i,c}` = soma dos pesos de `i` para nós de `c` e `Σ_tot(c)` = grau total de `c`. O
*local moving* escolhe o `c` de maior ganho; **empate mantém a comunidade corrente** (só troca em
ganho estritamente maior — `EPSILON = 1e-9`).

### 8.2 Agregação

Cada comunidade vira um super-nó; arestas internas viram **auto-laço com peso dobrado** (`×2`,
para preservar `2m`). Repete até `MAX_LEVELS = 8` níveis ou até não haver redução; cada nível faz
≤ `MAX_PASSES = 64` passos de local moving.

### 8.3 Determinação

Nós processados em ordem lexicográfica; comunidades ordenadas por tamanho desc e, em empate, pelo
menor membro. Resultado é **função pura** do grafo.

---

## 9. MinHash / LSH (D204)

Usado como *blocking* no dedup quando o corpus é grande e denso (a peneira exata degenera em
O(N²)).

### 9.1 Assinatura MinHash

Para cada termo `t` e cada permutação `i ∈ {0,…,63}`:

```
base     = fnv1a(t)                       (64 bits)
h_i(t)   = splitmix64( base ⊕ (i · φ) ),   φ = 0x9E3779B97F4A7C15
sig_i(S) = min_{t∈S} h_i(t)
```

`splitmix64` é um finalizador de mistura forte; `fnv1a` é estável entre execuções/plataformas.
`SIGNATURE_LEN = 64`.

**Propriedade de MinHash:** `P(sig_i(S₁) = sig_i(S₂)) = J(S₁,S₂)`, onde `J` é o Jaccard
`|S₁∩S₂|/|S₁∪S₂|`.

### 9.2 Banding

A assinatura de 64 é dividida em `BANDS = 16` bandas de `ROWS = 4` linhas. Um par compartilha
**ao menos uma** banda com probabilidade:

```
P(colisão) = 1 − (1 − s^ROWS)^BANDS = 1 − (1 − s⁴)^16
```

Para `s = 0,92` (limiar de merge): `s⁴ ≈ 0,716`, `(1−0,716)^16 ≈ 1,3e-9`, logo `P ≈ 1`. Pares
distantes são praticamente descartados. Como os candidatos ainda passam pelo **Dice exato** e pelo
limiar de merge, as propostas são **idênticas ou superconjunto** (recall ≥) — LSH é aproximado,
mas não perde par real (DIVERGENCES #107).

---

## 10. Caminho crítico: PERT/CPM (D205)

DAG de `depends_on`. Duração `dur(id) = lead_time` (fechada: `close − create`; aberta:
`agora − create`).

```
L(id) = dur(id) + max_{t ∈ deps(id)} L(t)          (ciclo ⇒ 0)
```

O caminho crítico é o `argmax` de `L` sobre todos os ids. **Desempate** (`better`): maior
`total_ms`; empate pelo caminho **mais longo** (`ids.len()` maior); empate pelo **menor** caminho
lexicográfico. Memoizado; ciclos quebram devolvendo vazio. `throughput` agrupa fechamentos em
janelas por `div_euclid(window)`.

---

## 11. Constantes (referência)

| Constante | Valor | Onde |
|---|---|---|
| `K1` / `B` | `1.5` / `0.75` | BM25 |
| `CONFIRMATION_STEP` | `0.1` | boost BM25 |
| pesos de campo | `3.0`/`2.0`/`1.0` | BM25 |
| `DEFAULT_MAX_TERM_RATIO` / `MIN_CUTOFF_CORPUS` | `0.9` / `64` | corte D173 |
| `DEFAULT_RRF_K` | `60` | RRF |
| pesos de fusão | `1.0`/`2.0`/`30.0`/`0.0` | RRF |
| `Z_95` | `1.96` | Wilson |
| `AGE_HALF_LIFE_DAYS` / `AGE_WEIGHT` | `90` / `0.05` | confiança |
| `FEEDBACK_WEIGHT` / `CONTRADICTION_PENALTY` | `0.2` / `0.1` | confiança |
| `DRIFT_FLOOR` | `0.5` | drift |
| `DEFAULT_REVIEW_THRESHOLD` / `DEFAULT_GROWTH_PERCENT` | `0.5` / `50` | retenção |
| `SMOOTHING` / `DRIFT_THRESHOLD` / `MIN_DRIFT_NOTES` | `1e-9` / `0.5` / `4` | KL/JS |
| `DAMPING` / `MAX_ITERATIONS` / `TOLERANCE` | `0.85` / `32` / `1e-8` | PageRank |
| `MAX_LEVELS` / `MAX_PASSES` | `8` / `64` | Louvain |
| `SIGNATURE_LEN` / `BANDS` / `ROWS` / `MIN_LSH_CORPUS` | `64` / `16` / `4` / `256` | MinHash/LSH |

## 12. Determinismo numérico

- Sem RNG: toda "aleatoriedade" (MinHash, PageRank) é **hash determinístico** ou semente fixa.
- Ordem canônica (`BTreeMap`/`BTreeSet`/sort estável) em toda soma acumulada.
- Comparações de empate sempre explícitas (`total_cmp`, `then_with`).
- O resultado é **função pura** da entrada — a mesma consulta/corpus dá o mesmo score.

## Onde vive

| Modelo | Arquivo |
|---|---|
| BM25/IDF | `retrieval/bm25.rs` |
| RRF/pesos | `retrieval/rrf.rs`, `retrieval/weights.rs` |
| Beta/Wilson | `lifecycle/beta.rs` |
| Confiança | `lifecycle/confidence.rs` |
| Retenção | `lifecycle/retention.rs`, `lifecycle/shelf_life.rs` |
| KL/JS | `lifecycle/term_drift.rs` |
| PageRank/PPR | `graph/rank.rs` |
| Louvain | `graph/communities.rs` |
| MinHash/LSH | `write/dedup/lsh.rs` |
| PERT/CPM | `task/flow.rs` |
