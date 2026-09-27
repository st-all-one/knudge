# Reranking + ANN — plano de execução detalhado (E19/T06 e E19/T08)

> **Status:** proposta/plano de execução. As duas últimas tarefas de E19 continuam **bloqueadas**
> por um **segundo modelo local** (cross-encoder) e por **escala de corpus** — exatamente os dois
> itens que o escopo "sem 2º modelo" excluiu.
>
> **Fecha:** [E19-T06](../implementation/23_modelo_conhecimento_rico.md) (R2 — reranking +
> expansão de consulta + fusão calibrada) e [E19-T08](../implementation/23_modelo_conhecimento_rico.md)
> (R3 — Matryoshka + ANN).
>
> **Base:** [`../proposals/modelo_conhecimento_rico.md`](modelo_conhecimento_rico.md) §6 (Vetores
> modernos) e §8 (R2/R3); [E19](../implementation/23_modelo_conhecimento_rico.md);
> [`../04_embeddings.md`](../04_embeddings.md).
>
> **Políticas:** R15/R43 (deps/hot path), R33 (degradação graciosa), D79/D101/D102 (provedor HTTP
> local), D202 (porta unificada `8889`), D92 (determinismo), E13-T09 (a bancada **observa**, não é
> gate), D14 (sem retrocompatibilidade), D15/D84 (`.idx/` derivado e descartável).

---

## 0. Estado atual (o que já existe e onde encaixar)

| Peça | Onde | Observação |
|---|---|---|
| Porta de embedding | `crates/knudge-core/src/ports/embedder.rs` (`trait Embedder`) | Modelo a copiar para a porta `Reranker`. |
| Adaptador HTTP | `crates/knudge-core/src/adapters/http.rs` (`HttpEmbedder`, `DEFAULT_ENDPOINT = http://127.0.0.1:8889/v1/embeddings`) | Mesmo formato OpenAI-compatible; `build_body`/`parse` reutilizáveis. |
| Fake de teste | `crates/knudge-core/src/ports/fakes/embedder.rs` (`FakeEmbedder`) | Base do `FakeReranker`. |
| Identidade do provedor | `crates/knudge-core/src/embeddings/meta.rs` (`EmbeddingMeta`, `fingerprint`) | Invalida o índice quando muda (D79). |
| Índice vetorial | `crates/knudge-core/src/embeddings/index.rs` (`EmbeddingIndex`, `save`/`load`/`parse`) | `.idx/embeddings.jsonl`; cabeçalho `meta`. |
| Busca vetorial | `crates/knudge-core/src/embeddings/semantic.rs` (`rank_query` — **brute-force**) | Ponto de plugue da ANN (T08). |
| Álgebra de vetores | `crates/knudge-core/src/embeddings/vector.rs` (`cosine`/`dot`/`normalize`) | Base da quantização (T08). |
| Canal semântico | `crates/knudge-core/src/retrieval/pipeline.rs` (`semantic_channel`) | Filtra por `allowed` (D102). |
| Montagem da query | `crates/knudge-cli/src/commands/ask/query.rs` (`attach_semantic`, `semantic_ids`) | Ponto de plugue do reranking (T06). |
| Fusão | `crates/knudge-core/src/retrieval/rrf.rs` (`fuse`) + `weights.rs` | RRF atual; alvo da "fusão calibrada" (T06). |
| Construção do embedder | `crates/knudge-cli/src/commands/embedder.rs` (`build`, `meta`) | Ponto de plugue do `Reranker` na CLI. |
| Chaves de config | `crates/knudge-core/src/config/schema/keys.rs` (`recall.*`) e `keys_embeddings.rs` (`embeddings.*`) | Onde entram as chaves novas. |
| Bancada de qualidade | `bench/src/quality.rs` + `bench/qualidade.md` | Hoje **saturada** em nDCG@5 (100 %). |

---

## 1. Pré-requisitos comuns (destravam as duas tarefas)

1. **Segundo modelo local (T06).** Um *cross-encoder* de reranking servido pelo `llama.cpp` em
   modo reranking. Exemplos de GGUF compatíveis: `bge-reranker-v2-m3`, `jina-reranker-v2-base`,
   `Qwen3-Reranker-0.6B`. Sobe no mesmo host do embedder (porta `8889` já é o default unificado,
   D202), em endpoint distinto (`/v1/rerank` ou `/rerank`, conforme a build).
2. **Escala de corpus (T08).** A ANN só compensa acima de ~10⁴ notas (o brute-force atual é
   aceitável abaixo disso). É preciso um corpus grande o suficiente para o A/B **mostrar** o
   ganho — a fixture sintética da bancada pode ser escalada para isso.
3. **Bancada com folga (T06).** Hoje `bench/qualidade.md` marca **100 % de nDCG@5** em todas as
   famílias: **não há headroom** para medir ganho de reranking. É preciso adicionar famílias
   **difíceis** (distratores confundíveis, consultas longas/ambíguas, paráfrases distantes) antes
   de qualquer A/B. Sem isso, "não move o ponteiro" é indistinguível de "bancada fácil demais".

> **Regra de ouro (R43):** se o A/B não mostrar ganho ≥ 20 % em qualidade (ou −20 % em latência,
> no caso da ANN) com o **mesmo output**, a mudança é **revertida** e vira `Dxx` negativa.

---

## 2. E19/T06 — reranking + expansão de consulta + fusão calibrada

### 2.1 Objetivo

Elevar o topo do ranking do `ask` reordenando os **top-K** candidatos da fusão com um
cross-encoder, expandir a consulta (sinônimos/HyDE) para melhorar recall semântico e trocar o RRF
por uma **fusão calibrada** por canal — tudo **opcional** e **degradável** para RRF.

**Não-objetivos:** tocar `write`/`task`; mudar o contrato de `notas/`; treinar modelo; inferência
in-process (D101).

### 2.2 Pré-requisito: modelo cross-encoder

- Subir o `llama.cpp` em modo reranking:
  `llama-server -m bge-reranker-v2-m3.gguf --reranking --port 8889` (ou build equivalente).
- O `kd` **não** baixa nem sobe o reranker (mesma política do embedder, D101); apenas o invoca.
- **Reconciliação (D182):** `kd drain service --status`/`--reconcile` deve passar a comparar
  também `embeddings.rerank_endpoint`/`embeddings.rerank_model` com o worker — mesma mecânica de
  `endpoint: ok|fora|divergente` já existente.

### 2.3 Porta nova `Reranker` (core puro)

Novo arquivo `crates/knudge-core/src/ports/reranker.rs`, espelhando `Embedder`:

```rust
/// Identidade do reranker (modelo/revisão). Mudança invalida cache, não o índice.
pub struct RerankMeta { pub model: String, pub revision: String }

pub trait Reranker: Send + Sync {
    fn meta(&self) -> &RerankMeta;
    /// Reordena/scores documentos para uma consulta. Preserva a ordem de entrada.
    /// # Errors: Timeout/Internal em falha do provedor — o chamador degrada para RRF (R33).
    fn score(&self, query: &str, docs: &[String]) -> Result<Vec<f32>>;
}
```

- Registrar em `ports/mod.rs` e reexportar.
- **Fake** `FakeReranker` em `ports/fakes/reranker.rs` (scores determinísticos por
  `hash(query|doc)`), usado nos testes do core.
- Adaptador real em `adapters/http.rs` (ou novo `adapters/rerank.rs`): reaproveita `Endpoint`,
  `build_body` e o parser de resposta; novo `build_rerank_body(model, query, docs)`.

### 2.4 Fluxo no `ask`

1. Em `commands/ask/query.rs::recall_query`, depois da fusão RRF, tomar os **top-K** ids
   (`recall.rerank_top_k`, default **30**, faixa 20–50).
2. Montar os textos dos candidatos (statement + snippet; **nunca** o corpo inteiro — custo).
3. Chamar `Reranker::score(query, docs)` e reordenar.
4. Combinar o score do reranker com a fusão (ver §2.5) e reordenar de forma **estável**
   (`score desc, id asc`).
5. **Degradação (R33):** sem reranker/modelo/endpoint, ou timeout → manter a ordem RRF e anexar
   `warnings[]`; `strict` promove o aviso a erro (D94).

Ponto de plugue: `commands/embedder.rs::build` ganha `build_reranker(session) -> Result<Option<Box<dyn Reranker>>>`.

### 2.5 Fusão calibrada

- Config `recall.fusion = rrf | calibrated` (default **`rrf`**, byte-idêntico).
- `calibrated`: normaliza o score de **cada canal** (min-max ou z-score) e combina por
  combinação convexa com os pesos de `weights.rs`; o score do reranker entra como canal extra.
- Preservar o desempate `(score desc, id asc)` (D81) e a **soma das parcelas** documentada em
  `channels` (D151). O `--json` deve continuar expondo as parcelas por canal.
- A/B obrigatório contra o RRF; se não Pareto-dominar, mantém `rrf` como default.

### 2.6 Expansão de consulta

- **Zero-LLM (default):** sinônimos derivados do próprio corpus — `tags`, `claims` (D207) e
  ontologia (`same_as`/`broader`/`narrower`, D207) + radical do `stem` (D206). Determinístico.
- **HyDE (opcional):** só se houver LLM local (mesmo servidor); gera um texto hipotético,
  embute e usa como vetor de consulta. **Desligado por default** (`recall.hyde = false`).
- Saída aditiva no `--json`: `query_expansion: {applied, terms[]}`.

### 2.7 Contrato e chaves de config

Novas chaves (canônicas, em ordem):

| Chave | Default | Papel |
|---|---|---|
| `recall.rerank` | `false` | Liga o reranking (opt-in). |
| `recall.rerank_top_k` | `30` | Quantos candidatos reranquear (20–50). |
| `recall.rerank_weight` | `1.0` | Peso do canal de rerank na fusão. |
| `recall.fusion` | `rrf` | `rrf` \| `calibrated`. |
| `recall.hyde` | `false` | Expansão HyDE (exige LLM). |
| `embeddings.rerank_endpoint` | derivado de `embeddings.endpoint` | URL do reranker. |
| `embeddings.rerank_model` | `bge-reranker-v2-m3` | Identidade do reranker. |
| `embeddings.rerank_timeout_ms` | `5000` | Timeout (→ degradação). |

- **Sem bump de `schema_version`** (D207 já foi o de R5); nenhum byte de `notas/` muda.
- `--json` **aditivo**: `rerank: {applied, model, top_k, scores[]}` e `channels.rerank`.

### 2.8 Arquivos a tocar

- Novos: `ports/reranker.rs`, `ports/fakes/reranker.rs`, `adapters/rerank.rs` (ou bloco em
  `http.rs`), `retrieval/fusion.rs` (calibrada) e testes.
- Editados: `ports/mod.rs`, `adapters/mod.rs`, `embeddings/meta.rs` (se o fingerprint precisar do
  reranker — provavelmente **não**, é cache, não índice), `retrieval/{rrf,weights,pipeline}.rs`,
  `commands/ask/{mod,query}.rs`, `commands/embedder.rs`, `config/schema/{keys,keys_embeddings}.rs`,
  `commands/drain/service.rs` + `scripts/knudge-idle.sh` (reconciliação).

### 2.9 Testes

- **Proptest:** normalização da fusão calibrada (invariâncias, limites `[0,1]`, monotonicidade);
  estabilidade do desempate.
- **Unit:** `FakeReranker` reordena; ausência de modelo ⇒ ordem RRF intacta + warning;
  timeout ⇒ idem.
- **Integração (CLI):** `kd ask --json` com reranker fixture ⇒ `data.rerank.applied = true`;
  sem fixture ⇒ `false` e saída idêntica ao baseline.
- **Golden:** o `--json` do `ask` sem reranker **não muda** (byte-idêntico).

### 2.10 Bancada e o problema de headroom

- **Antes** de codar: estender `bench/src/quality.rs` com famílias difíceis (distratores,
  consultas longas, paráfrases). Sem folga, o A/B não mede nada.
- Protocolo: `make bench-quality` (nDCG/MRR/Recall) **e** `make bench` (`ask` e2e, `--no-idle`).
- Orçamento: `ask` e2e ≤ **+25 %** por onda, acumulado ≤ **+40 %**; reranking só no top-K (20–50),
  nunca no caminho de `write`/`task`.
- Recorte: `bench/t06_rerank.md` versionado.

### 2.11 Aceite

- [ ] A/B com ganho em nDCG@5/MRR **na bancada difícil** (não na saturada).
- [ ] `ask` sem reranker: **byte-idêntico** ao atual.
- [ ] `--json` aditivo (`rerank`/`channels.rerank`) e degradação com `warnings[]`.
- [ ] Reconciliação do worker cobre o reranker (D182).
- [ ] `Dxx` nova (ex.: `D210`) + linha em `DIVERGENCES.md`; `make check` verde.

---

## 3. E19/T08 — Matryoshka + quantização + ANN

### 3.1 Objetivo

Reduzir memória e latência da busca vetorial sem perder recall: **truncar** dimensões (MRL),
**quantizar** (int8/binário) e, só quando o corpus doer, indexar com **ANN**. O brute-force atual
(`rank_query`) permanece o default até o limiar.

**Não-objetivos:** substituir o embedder; mudar o contrato de `notas/`; ANN por padrão.

### 3.2 Pré-requisito: modelo MRL + escala

- O modelo default `ibm-granite/granite-embedding-97m-multilingual-r2` (384d) suporta **MRL**
  (truncamento de dimensões). Confirmar na versão pinada (`embeddings.revision`).
- ANN só entra acima de `recall.ann_min_vectors` (proposto **10 000**); abaixo disso o brute-force
  é mais simples, exato e determinístico.

### 3.3 Matryoshka (MRL)

- Chave `embeddings.matryoshka_dim` (default **0** = desligado). Quando `> 0` e `< dimensions`:
  1. truncar o vetor para `matryoshka_dim`;
  2. **re-normalizar** (L2) — o truncamento quebra a norma unitária;
  3. persistir a dimensão usada na identidade (`EmbeddingMeta.dimensions` passa a ser a
     truncada) para invalidar o índice corretamente (D79).
- Efeito: `.idx/embeddings.jsonl` menor e `rank_query` mais rápido (O(d) por vetor).
- **A/B:** latência × recall em 384/256/128; adotar só se recall cair dentro do orçamento.

### 3.4 Quantização

- `embeddings.quantization = none | int8 | binary` (default **`none`**).
- **int8:** guardar `scale` por vetor + `i8`; score por produto interno aproximado.
- **binary:** sinal por dimensão (`1`/`0`) + distância de Hamming para pré-filtro; refino opcional
  com os vetores completos (re-ranking curto).
- Formato no `.idx/`: nova versão do codec em `embeddings/index.rs` (`serialize`/`parse`) —
  **derivado e reconstruível** (D15/D84), então mudar o formato exige apenas rebuild.
- `--json` do `drain --status` ganha diagnóstico aditivo (dimensão efetiva, quantização, bytes).

### 3.5 ANN

- Ponto de plugue: `embeddings/semantic.rs::rank_query`. Criar
  `embeddings/ann.rs` com a estrutura (HNSW ou IVF-PQ) atrás de uma trait
  `VectorIndex { fn search(&self, query, k) -> Vec<(String, f32)> }`, com duas implementações:
  `BruteForce` (default) e `Ann`.
- **Decisão de dependência (R43):** implementação **in-house determinística** é o caminho
  preferido (o projeto evita deps). Se adotar crate (`hnsw_rs`, `usearch`, …): justificar com A/B
  ≥ 20 % e `default-features = false`.
- **Determinismo (D92):** construção single-thread com **seed fixa**, ordem canônica de inserção
  e desempate `(score desc, id asc)`. Sem `rayon`/paralelismo (proibido).
- **Fallback exato:** abaixo do limiar ou se o índice ANN estiver ausente/corrompido, usar
  brute-force (R33) — nunca falhar a busca.
- Construção/cache no `.idx/` (como o resto dos derivados), reconstruível.

### 3.6 Contrato e chaves de config

| Chave | Default | Papel |
|---|---|---|
| `embeddings.matryoshka_dim` | `0` | Dimensão truncada (0 = usa `dimensions`). |
| `embeddings.quantization` | `none` | `none` \| `int8` \| `binary`. |
| `recall.ann` | `false` | Liga a ANN (opt-in). |
| `recall.ann_min_vectors` | `10000` | Limiar de escala para a ANN. |
| `recall.ann_ef_search` | `64` | Parâmetro de busca da ANN (se HNSW). |

- Nenhum byte de `notas/` muda; só `.idx/` (derivado).
- `--json` aditivo no `drain --status` e no `ask` (`data.semantic: {mode, dim, quant}`).

### 3.7 Arquivos a tocar

- Novos: `embeddings/ann.rs`, `embeddings/quantize.rs` e testes.
- Editados: `embeddings/{semantic,index,meta,vector}.rs`, `retrieval/pipeline.rs`,
  `commands/ask/query.rs`, `commands/embedder.rs`, `config/schema/keys{,_embeddings}.rs`,
  `commands/drain/*`, `bench/src/{e2e,quality}.rs`.

### 3.8 Testes

- **Proptest:** truncar+normalizar preserva direção (cosseno do vetor cheio ≈ do truncado acima de
  um piso); quantização int8/binary aproxima o score dentro da tolerância; ANN devolve os mesmos
  vizinhos que o brute-force num corpus pequeno (recall ≥ limiar).
- **Determinismo:** duas construções com a mesma seed e mesma ordem ⇒ mesmos resultados.
- **Degradação:** índice ANN ausente/corrompido ⇒ brute-force + warning.
- **Integração:** `kd ask --json` com `recall.ann=true` em corpus grande mantém recall.

### 3.9 Bancada

- Estender a fixture para N ≥ 10⁴ notas (a bancada já escala; ver `bench/src/fixture.rs`).
- A/B: latência p50/p90 × Recall@k da ANN vs brute-force, em 384/128d e `none/int8/binary`.
- Orçamento: `ask` e2e ≤ **+15 %**; memória do índice medida em bytes.
- Recorte: `bench/t08_ann.md` versionado.

### 3.10 Aceite

- [ ] Brute-force continua o **default** e byte-idêntico.
- [ ] MRL e quantização reduzem memória/latência com recall dentro do orçamento (A/B).
- [ ] ANN determinística (seed fixa) e com fallback exato.
- [ ] `.idx/` reconstruível; nenhum byte de `notas/` muda.
- [ ] `Dxx` nova (ex.: `D211`) + `DIVERGENCES.md`; `make check` verde.

---

## 4. Sequência e dependências

```
T06 (reranking) ──► T08 (Matryoshka/ANN)
   │                     │
   └─ pré-req: modelo    └─ pré-req: modelo MRL + corpus ≥ 10^4
      cross-encoder         (e a ANN idealmente aproveita o reranking curto)
```

- **T06 primeiro:** o reranking é o maior salto de qualidade e não depende da ANN.
- **T08 depois:** a ANN pode usar um **refino** com os vetores completos ou com o reranker (o
  plano de T06 já prevê re-ranking curto), o que melhora o recall da busca aproximada.
- Ambos exigem a **bancada difícil** (§1.3); sem ela, nenhum dos dois deve ser adotado.

## 5. Decisões e bordas esperadas

- **D210** (T06): porta `Reranker`, fusão calibrada opt-in, expansão zero-LLM por default,
  degradação para RRF.
- **D211** (T08): MRL/quantização opt-in, ANN com fallback exato e determinismo por seed.
- **`DIVERGENCES.md`:** (a) reranking não pode alterar a saída sem reranker (byte-idêntico);
  (b) fusão calibrada com desempate determinístico; (c) ANN devolve vizinhos estáveis com seed
  fixa; (d) quantização aproxima o score dentro da tolerância documentada.
- Revisam: D79/D101/D102 (provedor), D202 (porta), D151 (parcelas por canal), D81 (desempate),
  E13-T09 (bancada não é gate).

## 6. Checklist de conclusão (cada tarefa)

- [ ] Pré-requisito satisfeito (modelo/scale) e **bancada difícil** pronta.
- [ ] Defaults byte-idênticos (opt-in); `make check` verde.
- [ ] A/B de **qualidade + latência** em `bench/t06_rerank.md`/`bench/t08_ann.md`.
- [ ] `Dxx` registrada + `DIVERGENCES.md` com o teste que trava a borda.
- [ ] `--json` aditivo documentado; `MODULE.md`/docs atualizados.
- [ ] Se não move o ponteiro: **reverter** e registrar a `Dxx` negativa (R43).
