# Bateria A/B — `google/embeddinggemma-2` vs. modelo de referência

> Medição de **qualidade de recuperação semântica** sobre um corpus **real** (`_REF/.knudge`,
> 1241 notas). Objetivo: decidir se `google/embeddinggemma-2` substitui ou não o default atual
> (`ibm-granite/granite-embedding-97m-multilingual-r2`, 384d — D123) e em que dimensão.
> Critério declarado: **melhor qualidade de busca**, ainda que custe mais RAM/processamento.

Data: out/2026. Ferramenta: `bench/ab-embeddinggemma2/` (este diretório).

---

## 1. Cenários medidos

| # | cenário | modelo | dimensão | prefixo de tarefa |
|---|---|---|---:|---|
| A | `granite384` | `ibm-granite/granite-embedding-97m-multilingual-r2` | 384 (nativo) | — (referência) |
| B | `gemma768` | `google/embeddinggemma-2` | 768 (nativo) | não |
| C | `gemma384` | `google/embeddinggemma-2` | 384 (MRL) | não |
| D | `gemma768p` | `google/embeddinggemma-2` | 768 | `title: none` / SearchQuery |
| E | `gemma384p` | `google/embeddinggemma-2` | 384 (MRL) | `title: none` / SearchQuery |
| F | `gemma768pt` | `google/embeddinggemma-2` | 768 | `title: {stmt}` / SearchQuery |
| G | `gemma384pt` | `google/embeddinggemma-2` | 384 (MRL) | `title: {stmt}` / SearchQuery |

Servidores `llama.cpp` (build 11574) locais via OpenAI-compatible `/v1/embeddings` — o mesmo
caminho de produção (D101). Granite `--pooling mean` (384d); gemma-2 `--pooling mean` (768d,
com projeção+normalização do próprio modelo). Truncagem MRL 768→384 feita no cliente
(primeiras 384 dimensões + renormalização L2), pois o servidor llama.cpp ignora `dimensions`.

## 2. Método

Corpus: `_REF/.knudge/notas` — 1241 notas (285 fact, 493 task, 169 decision, 169 error, …).
Todas as consultas são derivadas das próprias notas, em 5 famílias:

| família | consulta | relevante | n |
|---|---|---|---:|
| `self` | `statement` da nota | a própria nota | 1241 |
| `kw` | ~10 palavras-chave do `statement` (sem stopwords) | a própria nota | 1241 |
| `body` | 1ª frase do corpo/evidência | a própria nota | 934 |
| `anchor` | `statement` | notas que compartilham ≥1 âncora (arquivo) | 800 |
| `tag` | `statement` | notas que compartilham ≥1 tag | 287 |

Métricas: **Recall@k**, **MRR** e **nDCG@10** sobre o **ranking semântico puro** (cosseno),
isolando o modelo da fusão BM25. Diferenças pareadas (gemma − granite) com **IC95% por
bootstrap** (2000 reamostragens), pareadas por consulta.

> As famílias `anchor`/`tag` medem relevância por **metadado** (arquivo/tag), que é
> essencialmente lexical e ruidoso — servem de controle negativo e **não** entram na decisão.

## 3. Resultados

### R@1 (%) e MRR

| cenário | self R@1 | kw R@1 | body R@1 | MRR(self) | nDCG@10(self) |
|---|---:|---:|---:|---:|---:|
| A granite 384 | 85.3 | 75.3 | 63.6 | 0.8983 | 0.9143 |
| B gemma 768 | 92.2 | 86.1 | 68.0 | 0.9558 | 0.9662 |
| C gemma 384 (MRL) | 91.9 | 83.9 | 67.7 | 0.9536 | 0.9643 |
| D gemma 768 + prefix | 94.6 | 90.7 | 66.1 | 0.9692 | 0.9765 |
| E gemma 384 + prefix | 94.0 | 90.6 | 66.2 | 0.9659 | 0.9740 |
| **F gemma 768 + prefix title** | **96.1** | **92.9** | 69.8 | **0.9796** | **0.9848** |
| **G gemma 384 + prefix title** | 95.6 | 92.8 | **70.6** | 0.9762 | 0.9821 |

Recall@k completo (self): A 85.3/93.5/94.4/96.7 · B 92.2/98.9/99.5/99.7 ·
F 96.1/99.8/99.8/100.0 · G 95.6/99.8/99.8/99.9.

### Diferença pareada vs. A (granite), IC95% bootstrap

| cenário | família | ΔR@1 (pp) [IC95%] | ΔMRR [IC95%] | W/T/L |
|---|---|---:|---:|---|
| F gemma768pt | self | **+10.80** [+8.78,+12.81] | +0.0812 [+0.0675,+0.0964] | 165/1057/19 |
| F gemma768pt | kw | **+17.57** [+15.23,+20.06] | +0.1404 [+0.1224,+0.1597] | 272/939/30 |
| F gemma768pt | body | **+6.21** [+3.85,+8.57] | +0.0630 [+0.0448,+0.0808] | 253/572/109 |
| G gemma384pt | self | +10.23 [+8.22,+12.17] | +0.0779 [+0.0644,+0.0926] | 160/1061/20 |
| G gemma384pt | kw | +17.41 [+14.99,+19.90] | +0.1391 [+0.1218,+0.1570] | 275/933/33 |
| G gemma384pt | body | +6.96 [+4.60,+9.42] | +0.0652 [+0.0473,+0.0852] | 251/575/108 |
| B gemma768 | self | +6.85 [+5.08,+8.62] | +0.0575 [+0.0451,+0.0698] | 141/1068/32 |
| C gemma384 | self | +6.53 [+4.83,+8.22] | +0.0553 [+0.0435,+0.0687] | 141/1065/35 |

Todos os IC95% das famílias-chave (`self`/`kw`/`body`) **excluem zero** → ganho real, não ruído.
Em `anchor`/`tag` o R@1 de gemma é ligeiramente menor (metadado lexical), mas o MRR melhora.

## 4. Análise

1. **Gemma-2 ganha folgado do granite** no canal semântico: +6 a +11 pp de R@1 (self) e
   +8 a +17 pp (kw). É a maior diferença medida em toda a bateria.
2. **Prefixo de tarefa importa muito** no gemma-2 (como o model card prevê): sem prefixo,
   self 92.2 %; com `SearchQuery`/`Document`, 96.1 % (+3.9 pp). É o segundo maior efeito.
3. **Formato do documento com título é o melhor**: `title: {statement} | text: {body}` supera
   `title: none | text: {statement}\n{body}` em todas as famílias (+1.5 pp self, +2.2 pp kw,
   **+4.4 pp body**). Faz sentido: o `statement` é o título natural da nota.
4. **MRL 384d ≈ 768d**: truncar para 384 custa só ~0.4–0.5 pp de R@1 (dentro do ruído) e **não
   custa nada** em `body`. Metade do índice, qualidade praticamente idêntica.
5. **Fusão híbrida (referência):** rodando o `recall` real (BM25+âncoras+RRF+semântico) só com o
   granite, o R@1 sobe de 85.3 → 86.1 (self) e 75.3 → 77.2 (kw) — confirma que o canal semântico
   é o que domina (peso 30, D124), mas a fusão completa por cenário é cara (~0,4 s/consulta em
   1241 notas) e ficou fora do escopo desta rodada.

## 5. Recomendação

**Adotar `google/embeddinggemma-2` com prefixos de tarefa** é a escolha de melhor qualidade,
coerente com o critério declarado.

- **Máxima qualidade (768d, `title`):** cenário **F** — self R@1 **96.1 %**, kw **92.9 %**,
  MRR 0.9796. Índice 2× o atual, sem truncagem no cliente.
- **Equivalente prático (384d MRL, `title`):** cenário **G** — self **95.6 %**, kw **92.8 %**,
  `body` **70.6 %** (o melhor de todos). Índice do mesmo tamanho do atual. Perde ~0.5 pp no self.
- Cenários **B/C** (sem prefixo) já superam o granite, mas deixam ~4–7 pp na mesa — **não**
  recomendados como configuração final.

Licença Apache-2.0 e pesos públicos, sem gating — sem impedimento de supply-chain (R43).
Custo: 296 MiB (Q8_0) × 110 MB do granite, e ~4× a latência de embedding (CPU).

## 6. Lacunas de código para tirar o ganho em produção

O ganho real exige mais que `config.toml`:

1. **Prefixos de tarefa assimétricos (bloqueante).** Hoje `embedding_text` (`pipeline.rs:117`) e
   a consulta (`commands/ask/query.rs`) são simétricos e sem prefixo. É preciso uma camada
   *model-aware*: documento = `title: {statement} | text: {body}` e consulta =
   `task: search result | query: {q}`. Mexe em contrato de bytes/cache (body_hash) → exige `Dxx`
   + goldens.
2. **Truncagem MRL no cliente (só para 384d).** `HttpEmbedder` não trunca/renormaliza e o
   servidor ignora `dimensions`. Para adotar 768d nativo, este item é dispensável.
3. **Validação de dimensão (bug pré-existente, corrigir antes de qualquer A/B de produção).**
   `parse_vectors` não confere o tamanho do vetor contra `meta.dimensions`, e
   `vector::cosine/dot` truncam em silêncio via `zip` — configurar 384 com modelo 768 produz
   scores plausíveis e errados.
4. **Worker (se adotado):** `scripts/knudge-idle.sh` pina `MODEL_ID`/`MODEL_FILE`/`MODEL_SHA256`
   (D183) e a reconciliação (D182) acusaria divergência.

## 7. Reprodução

```sh
# 1. servidores (llama.cpp build com suporte a embeddinggemma2)
llama serve -m granite-97m-r2-Q8_0.gguf          --embeddings --pooling mean --port 8890
llama serve -m embeddinggemma-2-Q8_0.gguf        --embeddings --pooling mean --port 8891

# 2. harness (neste diretório; fora do workspace)
cargo build --release
./target/release/ab-embeddinggemma2 \
    --scenario gemma768pt --root <PROJETO> \
    --endpoint http://127.0.0.1:8891/v1/embeddings \
    --model google/embeddinggemma-2 --dims 768 --truncate 0 --prefix gemmatitle \
    --out results/gemma768pt.json --dump results/gemma768pt.tsv

# 3. análise (IC95% bootstrap)
python3 analyze.py
```

Modelos: `ggml-org/embeddinggemma-2-GGUF` (`embeddinggemma-2-Q8_0.gguf`) e
`mykor/granite-embedding-97m-multilingual-r2-GGUF` (pinado pelo worker).

## 8. Limitações

- **Ground truth automático**: know-item (`self`/`kw`/`body`) + relevância por metadado
  (`anchor`/`tag`). Não há rótulo humano; as famílias `anchor`/`tag` são ruidosas e não decidem.
- A consulta `self` contém o texto que aparece no documento — inflada por natureza; `kw` e `body`
  são mais discriminativas e por isso sustentam a decisão.
- Mede o **canal semântico isolado**, não o `recall` final. A fusão completa por cenário não foi
  executada por custo (~0,4 s/consulta); a rodada com granite indica que o semântico domina.
- Incerteza residual de batching do servidor (~0,2 pp), muito abaixo dos efeitos medidos.
