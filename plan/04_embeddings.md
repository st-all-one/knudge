# Embeddings — provedor de vetores e escolha de modelo

> Ajuste de escopo: o cálculo dos vetores associados a cada nota (base do cluster e da proximidade semântica) passa a ser feito por um **provedor de embedding conectado via `config.toml`** (global/local), e não por um modelo fixo embutido.
>
> Complementa `00_panorama.md` §3 (Configuração) e §6 (Retrieval), e as decisões `D42`/`D79`.

---

## 1. O que o provedor de embedding alimenta

Os vetores **não** são a recuperação principal (que é BM25 + estrutura). Eles servem a três casos, todos **derivados** do corpus:

1. **Dedup semântico** no `write` — pega duplicatas que o BM25 não pega (mesma ideia, palavras diferentes).
2. **Descoberta de links não-declarados** — notas do mesmo tema em módulos diferentes, sem aresta explícita.
3. **Clustering semântico (fase 2)** — subtemas dentro de um cluster estrutural; alimenta o `compact` e a navegação por `clusters`.

Consequência: o embedding é **opcional e reconstruível**. `notas/` + `eventos/` reconstroem `.idx/embeddings.jsonl` do zero. Trocar de modelo = re-embedar tudo, sem perda de verdade.

### 1.1 Assíncrono, lazy e com gap tolerado

O embedding **nunca bloqueia** `write`, `recall` ou o rebuild estrutural. Ele é consumido por uma **fila de digestão**:

- No `write`, a nota é commitada imediatamente (nota + evento); o id entra numa **fila pendente** (derivada, em `.idx/`).
- Um **worker** processa a fila em lote — numa invocação ociosa do CLI, por timer ou via
  `kd maintenance index --drain`.
- `recall` **nunca espera**: usa BM25 + estrutura sempre; usa embeddings só para as notas já digeridas.

**Notas “apagadas” (dark).** Notas recém-criadas ficam invisíveis à camada vetorial até serem digeridas. Numa rajada de 10–20 notas, elas ficam dark por uma janela curta — **gap tolerado e aceito pelo projeto**. Continuam acháveis por BM25/estrutura/`get`; só não participam de dedup semântico e cluster até serem embeddadas.

**Consequências de contrato:**

- **Dedup no write é lexical.** O score do `recall` pré-`write` (limiares 0.75/0.92) é BM25 enquanto não há vetor. O dedup **semântico é eventual**: quando a fila drena, uma reconciliação (estilo `doctor`/`compact`) propõe merge/supersede dos quase-duplicados.
- **Estado por nota é derivado**, nunca no frontmatter: `embedded | pending | stale`, em `.idx/`. Trocar de modelo marca **tudo** como `pending`; nesse meio-tempo o retrieval cai para BM25.
- **Visibilidade:** `prime()` reporta `embeddings_pending: N`; `audit()`/`doctor` sinalizam backlog grande.
- **Backpressure:** `max_pending` limita a fila; acima disso, força catch-up em lote — **nunca descarta nota**.

**Modos:** `lazy` (default; digere ocioso/em lote), `eager` (digere logo após o write, ainda async), `manual` (só via `kd maintenance index --drain`).

```toml
[embeddings]
mode        = "lazy"     # lazy | eager | manual
async       = true       # nunca bloqueia write/read
batch       = 32
max_pending = 1000       # backpressure; acima, força catch-up
```

---

## 2. Avaliação dos dois modelos

Ambos são da mesma família e têm **a mesma forma**:

| | `msmarco-MiniLM-L12-v3` | `msmarco-MiniLM-L12-cos-v5` |
|---|---|---|
| Arquitetura | MiniLM-L12 (12 camadas) | MiniLM-L12 (12 camadas) |
| Dimensões | **384** | **384** |
| Parâmetros | ~33M | ~33M |
| Treino | MS MARCO (passage ranking), receita v3 | MS MARCO, receita **v5** |
| Métrica nativa | **dot-product** (exige normalizar p/ cosseno) | **cosseno** (`-cos`) |
| Geração | mais antiga | mais recente |
| Uso típico | recuperação assimétrica query→passage | idem, com cosseno nativo |

### Veredicto: **`msmarco-MiniLM-L12-cos-v5`**

Para este projeto, `cos-v5` é a escolha correta entre os dois, por quatro razões:

1. **Métrica nativa = cosseno.** O motor do knudge faz busca **brute-force por similaridade de cosseno** (normaliza os vetores e multiplica). O `cos-v5` é treinado exatamente para isso; o `v3` é dot-product e exigiria normalização artificial — funciona, mas não é o objetivo do treino.
2. **Receita de treino mais nova (v5).** Tende a superar v3 em retrieval; mesma arquitetura e mesmo custo, então não há trade-off de tamanho/latência.
3. **Zero normalização ambígua.** Com `cos-v5`, o pipeline é "embeda → cosseno"; com `v3`, seria "embeda → normaliza → cosseno", introduzindo uma etapa a mais para esquecer.
4. **Mesma dimensão (384)** → o índice, a quantização e o orçamento de memória não mudam entre os dois.

> `v3` só se justifica se um benchmark específico do corpus mostrar vantagem — o que é improvável dado o mesmo custo.

### L6 vs L12 (mesma família `cos-v5`)

Os dois `cos-v5` têm a **mesma dimensão (384)** e o **mesmo índice**, então a diferença é só profundidade × qualidade × velocidade:

| | `msmarco-MiniLM-L6-cos-v5` | `msmarco-MiniLM-L12-cos-v5` |
|---|---|---|
| Camadas | 6 | 12 |
| Parâmetros | ~22,7M | ~33,4M |
| Tamanho (fp32) | ~90 MB | ~130 MB |
| Inferência | **~2× mais rápida** | mais lenta |
| Qualidade (retrieval) | boa | **melhor** (diferença modesta) |
| Dimensão / índice | 384 | 384 |
| Armazenamento | **idêntico** | **idêntico** |

**Veredicto: `L12` como default; `L6` como opção rápida.**

1. **Qualidade por pouco custo.** O corpus do knudge é de milhares de notas curtas, não milhões; o custo 2× da inferência é irrelevante no rebuild em lote e aceitável no write. A camada extra ajuda exatamente onde o knudge precisa: **dedup semântico** e **clusters** com nuance.
2. **Índice idêntico.** Como ambos têm 384 dims, trocar L6↔L12 **não muda o formato** de `.idx/embeddings.jsonl` — só exige re-embed (modelo é versão de índice).
3. **`L6` entra como perfil rápido.** Em máquina modesta, CLI interativo ou `provider = http` com GPU compartilhada, `L6` é o *drop-in* de metade do custo. É a escolha de *edge*.

**Ressalva:** a diferença de qualidade é **modesta** (poucos pontos em retrieval) e pode desaparecer em notas curtas e vocabulário estável. A decisão real deve sair do **A/B no corpus** (§2.5), não do leaderboard — por isso o modelo é configurável.

Sugestão de perfis (açúcar sobre `model`):

```toml
[embeddings]
profile = "quality"   # quality = L12-cos-v5 | fast = L6-cos-v5 | custom (usa `model`)
```

### Ressalva importante: idioma

Os modelos MS MARCO são **treinados em inglês**. Se as notas do knudge forem predominantemente em **português** (como o próprio brainstorm), ambos vão **piorar** em paráfrases e sinônimos em PT.

Por isso o modelo é **configurável**: se o corpus for PT-BR, considerar um multilíngue (`intfloat/multilingual-e5-small`, `BAAI/bge-m3`, `paraphrase-multilingual-MiniLM-L12-v2`). O provedor é plugável justamente para permitir essa troca sem tocar no núcleo.

### Ressalva: assimetria

MS MARCO é **assimétrico** (query curta → passage longa), ótimo para `recall(q)`. Para **nota↔nota** (dedup, cluster) o ideal é um modelo **simétrico**. Na prática, notas atômicas e curtas reduzem a assimetria; ainda assim, medir dedup/cluster no corpus real antes de fixar.

### Validação recomendada

Antes de fixar o default, rodar um **A/B rápido no próprio corpus**: 20–30 pares conhecidos (duplicatas e relacionados) e comparar o ranking de `cos-v5` vs `v3` vs um multilíngue. O resultado entra no `DIVERGENCES`/golden.

---

## 3. Configuração

O provedor vive no `config.toml` (global como template, projeto com precedência — D61).

```toml
[embeddings]
enabled      = true
provider     = "http"                                     # http | lightweight | none
model        = "sentence-transformers/msmarco-MiniLM-L12-cos-v5"
revision     = "main"                                     # pinar commit/tag p/ reprodutibilidade
dimensions   = 384
similarity   = "cosine"                                   # cosine | dot
normalize    = true
device       = "auto"                                     # auto | cpu | cuda | metal
batch        = 32
mode         = "lazy"                                     # lazy | eager | manual
async        = true                                       # nunca bloqueia write/read
max_pending  = 1000                                       # backpressure; acima, força catch-up
cache        = true                                       # cache por body_hash em .idx/
cache_max_bytes = 33554432                                # teto com eviction LRU (32 MiB)
cache_ttl_days  = 30
flush_ms     = 2000                                       # flush coalescido (debounce) do índice
endpoint     = "http://127.0.0.1:8080/v1/embeddings"      # OpenAI-compatible (llama-server/TEI/Ollama)
timeout_ms   = 30000
retries      = 2
api_key_env  = "KNUDGE_EMBEDDING_API_KEY"
```

### Provedores

| `provider` | Como funciona | Quando usar |
|---|---|---|
| `http` | Cliente HTTP/1.1 **bloqueante** para um servidor OpenAI-compatible. O modelo roda **fora** do binário: o usuário sobe `llama-server -m msmarco-MiniLM-L12-cos-v5.Q5_K_M.gguf --embeddings` (ou TEI/Ollama/vLLM) e o knudge só aponta a URL | **Default**; offline e local, sem dependência de runtime de IA no binário (R16/R43) |
| `lightweight` | Embedder determinístico por hash (SHA-256 → vetor normalizado), **sem pesos** | Testes/CI e offline puro, sem download (D89) |
| `none` | Sem vetores; só BM25 + estrutura | Corpus pequeno; quando `learn()`/dedup semântico ainda não existem |

**Subindo o servidor local (exemplo com o GGUF do repositório):**

```sh
# llama.cpp; expõe /v1/embeddings (OpenAI-compatible)
llama-server -m msmarco-MiniLM-L12-cos-v5.Q5_K_M.gguf --embeddings --port 8080
```

O `kd` consome `http://127.0.0.1:8080/v1/embeddings` por padrão; aponte
`embeddings.endpoint` para outra porta/host se necessário. Enquanto o servidor não estiver de pé,
as notas ficam `pending` (gap tolerado) e o retrieval usa BM25.

> **Por que não inferência `local` in-process (ONNX/candle/llama.cpp)?** Ela exigiria uma
dependência pesada (build C++/CUDA, download de pesos) e um runtime de IA dentro do binário,
contra R16/R43. O modelo já roda otimizado num servidor dedicado; o knudge só consome HTTP —
D101.

### Regras

- **Derivado, sempre.** `.idx/embeddings.jsonl` é reconstruível; nunca é fonte da verdade.
- **Cache por `body_hash`.** Inferência é pulada quando o vetor já existe; falha de cache degrada para pass-through (D83).
- **Modelo é versão de índice.** Guardar `model`, `revision` e `dimensions` em `.idx/meta`; se a config mudar, **re-embedar tudo** (o índice antigo é invalidado, não mesclado).
- **`enabled=false` não bloqueia nada.** `recall` cai para BM25 puro; `write` cai para dedup lexical.
- **Nenhum embedding no frontmatter.** O vetor viaja no índice derivado, nunca na nota (mantém a nota leve e o TOON barato).
- **Flush coalescido.** O índice marca *dirty* e grava no máximo a cada `flush_ms` (default 2000), com flush forçado na saída (D85).

---

## 4. Custo e armazenamento

- 384 dims × `f32` = **1536 bytes/nota**; base64 ≈ 2 KB/nota em JSONL.
- 10k notas ≈ **15 MB** (`f32`) ou ~4 MB (`int8` quantizado).
- Brute-force cosseno em memória: sub-milissegundo até ~10k notas; ~50 ms em 100k. Suficiente; sem banco vetorial.

---

## 5. Impacto nas decisões

| Decisão | Antes | Agora |
|---|---|---|
| **D42** | Embeddings só depois de `learn()`/dedup semântico | Embeddings **configuráveis desde já** via `config.toml`; default `enabled=true` com `cos-v5`, mas o sistema funciona com `none` |
| **D79** (nova) | — | **Modelo default = `msmarco-MiniLM-L12-cos-v5`**, provedor plugável; validar idioma e A/B no corpus |
| **D101** (nova) | `provider = local\|http\|lightweight\|none` | **`provider = http`** (default) consumindo um **servidor local** OpenAI-compatible (`llama-server` com o GGUF); inferência in-process recusada (R16/R43); `lightweight`/`none` para CI/BM25 |
| **D65** | Módulos por escopo temático | Ganha o escopo **`embeddings`** (provedor + cliente), separado do `retrieval` |
| **D68** | MCP + CLI; FFI/WASM no planejamento | O provedor `http` (D101) mantém o binário leve; WASM/FFI deixam de ser necessários para inferência |
| **D83** | — | Cache por `body_hash` + estado `pending`/reconcile; falha nunca descarta nota |
| **D85** | — | Flush coalescido do índice (debounce + flush na saída) |
| **D89** | — | `provider = "lightweight"` para testes/CI/offline |
| **D90** | — | `kd eval --ab` (Recall@k/nDCG@k/MRR) decide o modelo sobre o corpus |

---

## 6. Em uma frase

**`msmarco-MiniLM-L12-cos-v5` como default** (qualidade, mesmo índice do L6, embedding em lote), com **`msmarco-MiniLM-L6-cos-v5` como perfil rápido** para latência/edge — ambos treinados para cosseno, na mesma dimensão e custo de armazenamento. O usuário sobe um **servidor local** (`llama-server` com o GGUF) e o knudge consome via **`provider = http`** (OpenAI-compatible), **assíncrono e lazy** (nunca bloqueia; fila de digestão com gap tolerado), **cache por `body_hash`** e **flush coalescido**, sempre derivado e re-embebido quando o modelo muda; `lightweight` cobre testes/CI e `none` cai para BM25; a escolha do modelo se decide por **`kd eval --ab`** sobre o corpus; com a ressalva de que, se o corpus for PT-BR, um modelo multilíngue deve ser avaliado.
