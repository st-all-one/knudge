# knudge-core — núcleo puro

Coração do knudge: **modelo, contrato de bytes, persistência, busca, escrita, ciclo de vida e
tarefas**. É puro por construção (D65) — não toca terminal, `argv`, relógio/RNG global nem o
sistema de arquivos diretamente. Todo acesso ao mundo externo atravessa uma **porta**
([`ports`](#portas-fakes-e-adaptadores)). As implementações reais (`adapters`) vivem no mesmo
crate, mas o domínio **nunca** as importa.

> Dependências externas mínimas: `thiserror` (erros), `indexmap` (ordem determinística),
> `unicode-normalization` (NFC) e `sha2` (hash). Nada de `tokio`, `reqwest`, DB ou ORM.

O crate é **publicável** (`publish = true`) e a fachada [`knudge`](#fachada-de-incorporação)
oferece a mesma montagem de adaptadores + projeto + config que a CLI/MCP fazem. O diretório de
conhecimento é configurável por `Project` (default `.knudge`, aceita aninhado `.a/b`).

## Invariantes

- `#![forbid(unsafe_code)]` (R01); `#![warn(missing_docs)]`.
- Sem `Rc`/`Weak`/`RefCell`/`Cell`, `HashMap`/`HashSet`, `LinkedList` (use `BTreeMap`/`IndexMap`).
- Sem `unwrap`/`expect`/`panic` no código de produção (D92); `Result<T>` = `Result<T, Error>`.
- Sem `SystemTime::now`/`Instant::now`/`env::var`/`process::exit` — só via portas.
- Aritmética com `checked_*`/`saturating_*`; `overflow-checks` ligado em dev e release.
- Indexação/slicing (`[]`) negada: `.get()`, `.first()`, iteradores.
- Determinismo é requisito: toda iteração sobre coleções tem ordem estável.
- `notas/` é a **verdade**; `eventos/` é auditoria; `.idx/` é derivado e descartável (D20/D21/D84).

## Portas, fakes e adaptadores

O domínio depende só das *traits*; o binário monta os adaptadores reais; os testes usam os fakes.

| Porta (`ports`) | Contrato | Fake (`ports::fakes`) | Adaptador (`adapters`) |
|---|---|---|---|
| `Clock` | tempo em ms UTC | `FixedClock` | `SystemClock` |
| `Rng` | aleatoriedade (só jitter) | `SeqRng` | `ThreadRng` |
| `Env` | variáveis/caminhos do ambiente | `FakeEnv` | `StdEnv` |
| `Fs` | I/O atômico, sem seguir symlink | `MemFs`, `FaultyFs` | `StdFs` |
| `Git` | comandos `git` (sem shell) | `FakeGit` | `StdGit` |
| `HookRunner` | hooks externos com timeout | `NoopHookRunner` | `ProcessHookRunner` |
| `Logger` | log estruturado com redação | `RecordingLogger` | `TracingLogger` (no CLI) |
| `Embedder` | vetores de embedding | `FakeEmbedder` | `HttpEmbedder` |

`FaultyFs` injeta falhas de escrita para testar atomicidade/rebuild (E03). `MemFs` é o FS
determinístico padrão dos testes do core.

## Mapa de módulos

| Módulo | Papel em uma linha | Entradas principais |
|---|---|---|
| `error` | Taxonomia de erro e mapa código→exit (R30–R35). | `Error`, `ErrorKind`, `Result`, `lock_or_recover` |
| `time` | `Timestamp` UTC em milissegundos (D07). | `Timestamp` |
| `logging` | Redação de segredos no layer de log (R22/D159). | `Redactor` |
| `schema` | Contrato de bytes do frontmatter: chaves, enums, IDs, hash. | `Frontmatter`, `NoteType`, `EdgeKind`, `Claim`, `Provenance` |
| `toon` | Parser/emissor do frontmatter TOON. | `parse`, `emit`, `split_frontmatter` |
| `jsonl` | JSONL tolerante + codec JSON canônico. | `lines`, `json::{encode, decode}` |
| `store` | Persistência crash-safe de notas/eventos/lock/rebuild. | `Store`, `Note`, `EventLog`, `LockGuard` |
| `corpus` | Leitura **única** do store (notas + índice + grafo). | `Corpus`, `load_notes`, `load_fresh` |
| `config` | Config em dois níveis + codec TOML próprio. | `Config`, `ConfigValue`, `Table` |
| `git` | Worktree, `info/exclude`, `AGENTS.md`, skill, `sync`. | `Project`, `onboard`, `sync`, `protocol_block` |
| `graph` | Projeção de arestas: integridade, ciclos, rank, comunidades, ontologia, TMS. | `Graph`, `louvain`, `pagerank`, `equivalence_classes` |
| `retrieval` | BM25 + âncoras + filtros + views + fusão RRF. | `recall`, `get`, `Index`, `RecallQuery` |
| `write` | Escrita idempotente, dedup, update/supersede, outcomes, lote. | `write`, `Draft`, `Patch`, `WriteOutcome` |
| `handoff` | `rewind`: manifest/escopo/working set + orçamento + `context_id`. | `rewind`, `RewindMode`, `budget` |
| `maintenance` | `diff`, `learn`, `compact` (só propõem). | `diff`, `learn`, `propose_compact` |
| `task` | Hierarquia `epic ⊃ {issue ⊃ task \| task}` e fluxo. | `submit`, `TaskSpec`, `context_of`, `critical_path` |
| `health` | Validators, `doctor [--fix]`, `audit`, evidência, leitura tolerante. | `doctor`, `doctor_fix`, `accept`, `audit` |
| `lifecycle` | Shelf-life, retenção, confiança, decay, purga, clusters. | `demotion_candidates`, `posterior_mean`, `DriftStore` |
| `embeddings` | Provedor plugável, cache, fila lazy, ranking/sugestão. | `drain`, `rank_query`, `suggest`, `EmbeddingIndex` |
| `knowledge` | Promoção de conhecimento a regras no `AGENTS.md`. | `recommend`, `render_block` |
| `ports` | Traits determinísticas + fakes. | `Clock`…`Embedder`, `fakes::*` |
| `adapters` | Implementações reais (`std`) — **fora** do domínio. | `StdFs`, `StdGit`, `HttpEmbedder`… |
| `knudge` | **Fachada de incorporação** (adaptadores reais + projeto + config). | `Knudge`, `KnudgeBuilder` |

## Fachada de incorporação

`knudge::Knudge` é a porta de entrada para usar o núcleo como biblioteca: monta os adaptadores
`std`, resolve o `Project` e carrega a config efetiva, expondo `store`, `events`, `index`,
`corpus`, `graph`, `notes`, `write_context` e `sweep_residues`. O layout do diretório de
conhecimento é configurável:

```rust
let kd = Knudge::builder().knowledge_dir(".a/b").open()?;
```

Tudo abaixo da fachada continua puro (portas) — quem precisa injectar outro `Fs`/`Git` usa
`Project`, `Store`, `Index` etc. diretamente. Os padrões de Git (`info/exclude`,
`.gitattributes`, `sync`, `AGENTS.md`) e a varredura de âncoras derivam do `Project::layout`, de
modo que trocar `.knudge` por `.a/b` é consistente.

## Detalhe por escopo

### Fundação — `error`, `time`, `logging`

- **`error`**: `enum Error` `#[non_exhaustive]` (`Send + Sync + 'static`), sem `Box<dyn Error>` na
  API. `ErrorKind` é o contrato de máquina (nunca traduzido) e `exit_code()` mapeia
  `not_found=3, invalid_input=2, conflict=4, io=5, timeout=6, config=7, schema=8,
  unsafe_blocked=9, internal=70` (101 é panic, 0 é sucesso). I/O sempre com `Error::io(path, src)`
  (R34). `retryable()` = só `Timeout`. `lock_or_recover` resolve poison de mutex.
- **`time`**: `Timestamp(i64)` ms desde a época, UTC; `to_rfc3339`. Nunca usa o relógio global.
- **`logging`**: `Redactor` substitui segredos por `[REDACTED:<tipo>]` (D159); corpo de nota e
  tokens nunca vão para o log (R22).

### `schema` — contrato de bytes

Fixa ordem de chaves (D04/D13), enums fechados, normalização/hash (D06/D95) e IDs por conteúdo
(D01–D03). A serialização concreta vive em `toon`.

| Arquivo | Papel |
|---|---|
| `mod.rs` | Reexports; `SCHEMA_VERSION = 2`. |
| `keys.rs` | `CANONICAL_KEYS` (**31**, ordem canônica), `REQUIRED_KEYS` (5; `type` derivado em `epic`). |
| `types.rs` | Enums fechados: `NoteType` (**10** armazenáveis + `Epic` **derivado** de `scope=epic`, D149), `Scope` (`epic`/`issue`/`task`), `Classification` (`foundational`/`tactical`/`observational`), `Status` (`active`/`in_progress`/`blocked`/`closed`/`superseded`/`forgotten`). |
| `parse.rs` | `FromStr` dos enums fechados com "did-you-mean" (D212). |
| `edge.rs` | `EdgeKind` (**12**) + `EDGE_KEYS`; `is_ontology`/`is_symmetric`/`inverse`/`is_supersession`. |
| `frontmatter.rs` | `Frontmatter` canônico e `validate()` (chaves, tipos, claims, proveniência). |
| `body.rs` | `normalize` (NFC + trim + colapso) e `body_hash`. |
| `hash.rs` | Hash curto endereçável (D95). |
| `id.rs` | `note_id` = `<prefixo>_<base36(8)>` sobre `type + U+001F + normalize(statement)`; prefixo é **histórico** (D01–D03). |
| `text.rs` | Contagem de `statement` em escalares Unicode (D08). |
| `value.rs` | Valor do subconjunto TOON (D74/D75). |
| `outcomes.rs` | `outcome_stats` — sucessos/falhas para a confiança (D189/D190). |
| `slots/` | `missing_slots`/`expected_slots` — obrigatoriedades **soft** por espécie (D191). |
| `claims.rs` | `Claim { subject, relation, object }` SPO, ≤120 escalares (D207). |
| `provenance.rs` | `Provenance { entity, activity, agent }` PROV-lite (D207). |
| `suggest.rs` | `hint`/`closest`/`distance` — "did-you-mean" (Levenshtein) para enums fechados (D212). |

### `toon` — frontmatter

| Arquivo | Papel |
|---|---|
| `mod.rs` | `parse`/`emit`/`split_frontmatter`/`detect_version`; `FENCE = "---"`. |
| `lex.rs` | Lexer de linhas do subconjunto TOON. |
| `parse.rs` | Parser de blocos. |
| `emit.rs` | Emissor (ordem canônica, opcionais omitidos). |
| `flow.rs` | Escalares e coleções *flow* (`[a, b]`, `{a: 1}`). |

Gramática completa em [`TOON.md`](../../wiki/specs/TOON.md). Mudou o formato? Atualize `TOON.md`,
goldens e `SCHEMA_VERSION`/rebuild (D15).

### `jsonl` — JSON Lines

`lines` itera linhas; dedup on-read (D26) e tolerância a linha malformada (skip + warning).
`json/` é um codec JSON mínimo, compacto e **determinístico** (chaves ordenadas), sem dependência
externa — usado no log de eventos e nos derivados `.idx/`.

### `store` — persistência crash-safe

| Arquivo | Papel |
|---|---|
| `mod.rs` | `Store` sobre uma `Fs`: `read`/`read_optional`/`write`/`update`/`remove`/`list_ids`, layout `notas/<tipo>/<id>.md` com fallback ao plano legado (D150). |
| `note.rs` | `Note` = frontmatter TOON + corpo; `parse`/`render`. |
| `commit.rs` | Ordem multi-arquivo: **nota primeiro, evento depois** (D21). |
| `events/` | `EventLog` append-only, rotação por tamanho, leitura tolerante e checkpoint. |
| `lock.rs` | Lock advisory por arquivo-alvo (D23–D25) com liberação RAII. |
| `rebuild.rs` | `Staging` — rebuild double-buffer do índice derivado (D27). |
| `purge.rs` | `purge_derived` em toda remoção (D84). |
| `detach.rs` | Limpeza referencial ao remover nota (D46/D84). |
| `sweep.rs` | Varredura de resíduos na inicialização (`*.tmp`/`*.stale`, nunca `*.lock`; D160). |

### `corpus` — leitura única

`Corpus { notes, index, graph }` derivados do **mesmo** vetor, sem clonar (E15-T02). `load_notes`
para quem só quer frontmatters (paralela por `std::thread::scope`, ordem preservada — E15-T12);
`load_fresh` reusa o índice de `.idx/` quando fresco (E15-T11). Ordem byte-idêntica à de duas
leituras separadas.

### `config` — dois níveis

Global (template do usuário) e projeto (efetivo, tem precedência — D61–D64). `Config::effective`
mescla e **remove segredos do projeto** (D91). Writer TOML mantém ordem de inserção/quoting
estáveis (diff mínimo, D63). Schema em `schema/keys.rs` + `schema/keys_embeddings.rs`.

### `git` — worktree e onboarding

Resolve a raiz no **worktree principal** (D29/D91), escreve `.git/info/exclude` (D30),
`merge=union` para o log (D31), blocos idempotentes em `AGENTS.md` (D60) e `sync` de
`notas/`+`eventos/` (D32). `skill.rs` instala a skill do projeto (`.agents/skill/kd/SKILL.md`,
D162). `onboard` cria/atualiza `.knudge/` de forma idempotente. Nada de shell (R12).

### `graph` — projeção de arestas

O grafo é **projeção** das notas (D49); a extração textual é sugestão revisável e **nunca** entra
no grafo (D49/D50).

| Arquivo | Papel |
|---|---|
| `link.rs` | `insert_note`/`link` — nós e arestas explícitas. |
| `integrity.rs` | `Issue`/`IssueKind` — referências quebradas, órfãos. |
| `cycles.rs` | `strongly_connected`/`cyclic_components`. |
| `extract.rs` | Sugestões conservadoras de aresta (D49/D50). |
| `suggestions.rs` | Store derivado de sugestões (`.idx/suggestions.jsonl`). |
| `rank.rs` | `pagerank`/`personalized_pagerank` (PPR determinístico, D192). |
| `communities.rs` | `louvain` (comunidades, D193). |
| `ontology.rs` | `equivalence_classes`, `broader_ancestors`, `narrower_descendants`, `has_hierarchy_cycle`, `claim_conflicts` (D207). |
| `tms.rs` | `retracted`, `defeated_dependents`, `defeated_by_replacement` (D208). |

### `retrieval` — busca

Cascata do mais barato ao mais caro: filtros → BM25 no resíduo → âncoras → fusão RRF
(D39/D41/D81). Canal ausente/falho degrada com `warnings` (a menos que `strict`).

| Arquivo | Papel |
|---|---|
| `mod.rs` | `recall`/`get`; `RecallQuery`/`RecallHit`/`HitChannels`; consts (`DEFAULT_RRF_K=60`, `DEFAULT_LIMIT=5`, `DEFAULT_MAX_TERM_RATIO=0.9`). |
| `index/` | `Index` derivado, persistência canônica e carga tolerante; `INDEX_FORMAT = "retrieval-v4"`. |
| `postings.rs` | Índice invertido em memória (peneira do BM25, nunca persistido). |
| `bm25.rs` | BM25 com IDF por campo e boost por confirmação. |
| `token.rs` | Tokenização com fold de diacríticos (D172). |
| `stem.rs` | Stemmer PT conservador (D206). |
| `anchor.rs` | Canal de âncoras + `GlobPattern` (DP). |
| `filter.rs` | Filtros determinísticos **antes** da estatística. |
| `views.rs` | Views `ready`/`blocked`. |
| `rrf.rs` | Fusão Reciprocal Rank Fusion. |
| `weights.rs` | Pesos calibrados da fusão (`anchor_weight=2.0`, D179). |
| `rank.rs` | Ranking por confiança, sem query textual (D107). |
| `tags.rs` | Vocabulário de tags. |
| `temporal.rs` | Consulta `as_of` (D155). |
| `snippet.rs` | Snippet/match do corpo (D161/D172). |
| `contradiction.rs` | Lado perdedor de contradições (D177). |
| `why.rs` | Motivo (`why`) de um hit (D39). |
| `pipeline.rs` | Construção de candidatos e hits. |
| `format.rs` | Formatação no contrato de pipe. |

### `write` — protocolo idempotente

Idempotente por conteúdo (D01). Duas fases: `recall` ranqueia; decisão calibrada `<0.75` cria,
`0.75–0.92` merge, `≥0.92` rejeita (D26). Estrito na forma, tolerante na operação (D05/D16/D17).

| Arquivo | Papel |
|---|---|
| `mod.rs` | `write`, `WriteContext`, `WriteAction`/`WriteOutcome`, `thresholds_from_config`. |
| `draft.rs` | `Draft` — campos tipados que viram frontmatter válido. |
| `dedup/` | Duas fases; `sieve` exata (pequeno/esparso) e `lsh` MinHash/LSH (denso grande, D204); `merges` propõe fusões. |
| `merge.rs` | Fusão de campos semânticos (inclui claims/proveniência, D207). |
| `update/` | `update` versionado + `history`; `patch.rs` campos mutáveis (`--update --params`, D147). |
| `lifecycle.rs` | `link`, `forget`, `restore` (soft, D49/D52). |
| `status.rs` | Transições de `status` permitidas. |
| `outcome.rs` | `outcomes[]` para **qualquer** nota (D103). |
| `batch.rs` | Lote a partir de JSONL de rascunhos (D110). |

### `handoff` — `rewind`

`prime` é o protocolo estático (D57); `rewind` é o estado dinâmico. Três modos — manifest
(~30 tokens), escopo (container/domínio) e working set (arquivos) — com orçamento sem tokenizer
(D40/D82) e `context_id` endereçável (D88).

| Arquivo | Papel |
|---|---|
| `mod.rs` | `rewind`, `RewindMode`, `RewindRequest`/`RewindInput`/`RewindOutput`. |
| `manifest.rs` | Manifest e ranking por trust-tier. |
| `next.rs` | Linhas dinâmicas `next:`/`fresh:` (D106). |
| `scope.rs` | Auto-context-scope e auto-flip (D41). |
| `budget.rs` | Orçamento de tokens sem tokenizer. |
| `context.rs` | `context_id` endereçável e handoff 1:1. |

### `maintenance` — passado e consolidação

Nada escreve sem aceite (D33/D47). `diff` lê a auditoria de eventos; `learn` propõe
notas/links/merges (write-gap, quase-duplicata, lacuna de grafo, tarefa→conhecimento — D111);
`compact` propõe fusões (e `apply_compact` as materializa após aceite).

### `task` — hierarquia como view derivada

`epic` é grupo (`scope=epic`, sem `type`, D149); `issue`/`task` são `type=task`. O pai vive no
**marcador do corpo** (D93) e vira aresta `results_in`. `kd write` rejeita `task`/`epic` — tudo de
tarefa passa aqui.

| Arquivo | Papel |
|---|---|
| `mod.rs` | `submit`, `parent_of`, `is_task`, reexports. |
| `spec.rs` | `TaskSpec`, `WORK_KINDS`, `validate_kind`. |
| `hierarchy.rs` | `child`/`children`/`validate_parent`/`validate_blocks`. |
| `context.rs` | `context_of` — pai/bloqueadores/filhos (D125). |
| `progress.rs` | `progress_of`/`epic_of` — rollup (D127). |
| `lifecycle.rs` | `review` (fechamento) e `outcome` (D53/D138). |
| `impact.rs` | `impact`/`impacts`/`is_actionable` (D106/D109). |
| `role.rs` / `mode.rs` | Papel (D115/D134) e modo derivados do grafo (D116/D136). |
| `plan.rs` / `template.rs` | Plano por LLM (`--prompt`/`--from`, D105); templates `.knudge/templates.toml` (D97). |
| `batch/` | Lote por `--params`/`--batch` (D141). |
| `program.rs` | Programas externos (Épico-raiz ancorado a `plan/*.md`, D119). |
| `flow.rs` | `task_flows`/`throughput`/`critical_path` PERT/CPM (D205). |
| `membership.rs` | Marcador de filiação (D52/D93). |

### `health` — não apodrecer em silêncio

Nota ruim nunca derruba um comando (D16–D18); nada é apagado sem decisão (D52/D86).

| Arquivo | Papel |
|---|---|
| `validator/` | Catálogo de validators e resolução de `checks` (`kind = check\|gate`, D54/D156). |
| `audit.rs` | Relatório de integridade/saúde do corpus. |
| `doctor/` | `doctor [--fix]`; checks, `body_check` (D162/D191), `semantic` (integridade + claims/ontologia) e `fix` reversível (D19). |
| `evidence.rs` | Fechamento por evidência e `outcomes[]` (D48/D55). |
| `gate.rs` | Portão de evidência de proposta (D156). |
| `tolerant.rs` | Leitura tolerante (`SkippedNote`, `TolerantRead`). |
| `anchors/` | Âncoras com `content_hash` + verify-on-hit (D86); store derivado. |

### `lifecycle` — retenção e consolidação

Nada de conhecimento válido é demolido por ciclo (D45); nada é removido antes da janela de
retenção.

| Arquivo | Papel |
|---|---|
| `shelf_life.rs` / `retention.rs` | Shelf-life por classificação (D44) e curva de esquecimento/revisão espaçada (D190). |
| `usage.rs` | Uso (citações) por nota — derivado `.idx/usage.jsonl` (D154). |
| `confidence.rs` / `beta.rs` | Confiança derivada (D87) com posterior Beta-Bernoulli (D189). |
| `decay.rs` | Decay de âncoras no rebuild (D43). |
| `drift.rs` | Drift de âncoras persistido (`.idx/drift.jsonl`, D203). |
| `term_drift.rs` | Drift de termos KL/JS (D208). |
| `plan.rs` | `demotion_candidates` — shelf-life × decay × proteção × contradição × TMS × drift. |
| `retire.rs` / `supersession.rs` | Purga de inativos com histórico (D48); supersessão com proteção de ciclos (D45). |
| `clusters.rs` / `semantic.rs` | Clusters fase 1 (estruturais, D47) e fase 2 (semânticos, off-path, D42). |
| `communities.rs` | GraphRAG sobre arestas + âncoras (D193). |

### `embeddings` — derivado e opcional

Vetores nunca bloqueiam (D42/D79). Nota recém-criada fica *dark* até a fila derivada drená-la
(D80), com cache por `body_hash` (D83), purga em remoção (D84), flush coalescido (D85) e provedor
determinístico para testes (D89).

| Arquivo | Papel |
|---|---|
| `meta.rs` | Identidade do provedor/índice (`EmbeddingMeta`, `Similarity`). |
| `vector.rs` | Álgebra de vetores (`cosine`, `dot`, `normalize`, `similarity`). |
| `index.rs` | `EmbeddingIndex` (`.idx/embeddings.jsonl`), invalidação por modelo. |
| `cache/` | Cache `(body_hash, modelo)` com teto/LRU; persistência JSONL (D148/D153). |
| `pipeline.rs` | `drain` — worker de reconcile da fila. |
| `vectors.rs` | Resolução de vetores do dreno: cache, lote e isolamento. |
| `state.rs` | `EmbeddingState`/`EmbeddingMode`/`classify`/`is_backlogged`. |
| `semantic.rs` | Vizinhos, duplicatas e links no índice vetorial. |
| `suggest.rs` | Sugestões semânticas de aresta/contradição (D158). |
| `flush.rs` | Flush coalescido. |
| `lightweight.rs` | Embedder por hash para testes/CI/offline. |

### `knowledge` — governança

`recommend` sugere regras a partir do corpus; `render_block`/`apply_block` mantêm o bloco
`knudge:rules` do `AGENTS.md` (D157). Desligado por default (`rules.enabled`).

## Fluxos

- **Escrita** (`write`): `Draft` → valida forma → `recall` de candidatos → decisão calibrada
  (cria/merge/rejeita) → `merge`/`update` → commit nota→evento → purge/flush derivados.
- **Busca** (`ask`): `Corpus::load`/`load_fresh` → `retrieval::recall` (filtros → BM25 → âncoras
  → RRF, + canal PPR) → `snippet`/`why` → `format`.
- **Rewind** (`handoff`): `Corpus` → `detect_scope` → `rewind` (manifest/escopo/working set) →
  `budget` → `context_id` → `next_tasks`.
- **Manutenção**: `EventLog` + `Corpus` → `diff`/`learn`/`propose_compact` →
  `health::gate::accept` → `apply_compact`/`write`.
- **Embeddings**: `drain` consome a fila → `Embedder` → cache → `EmbeddingIndex` → `semantic`/
  `suggest`.

## Contratos de bytes (resumo)

- Ordem canônica de **31 chaves** (`CANONICAL_KEYS`); opcionais **omitidos**, nunca `null`.
- `type` é enum fechado de **10** armazenáveis (`epic` é derivado de `scope=epic`, D149); `EdgeKind` de **12**; `scope`/`classification`/`status` fechados.
- `normalize` = NFC + trim + colapso; `body_hash` e `id` derivam dele (D95).
- `SCHEMA_VERSION = 2` (claims/proveniência, D207); `INDEX_FORMAT = "retrieval-v4"` (D206).
- Chave desconhecida: rejeita no write, warning no read. Tipo desconhecido: rejeita a nota.

## Testes

Unidade em `src/<mod>/tests.rs` com `#[cfg(test)] mod tests;`; integração em
`crates/knudge-core/tests/`. Sem `unwrap`/`expect`/`panic` (use `?`, `matches!`, `.ok()`).
Testes do core usam **fakes** (`FixedClock`, `SeqRng`, `MemFs`, `FakeGit`, `RecordingLogger`) —
nada de tempo/FS reais. Proptest para funções puras; golden para contrato de bytes.

## Onde mexer

| Quero… | Vá para |
|---|---|
| schema/tipos/IDs/hash | `schema/` |
| parser/emissor TOON | `toon/` |
| notas, eventos, lock, rebuild | `store/` |
| leitura única do corpus | `corpus/` |
| config TOML em dois níveis | `config/` |
| worktree, exclude, `AGENTS.md`, `sync` | `git/` |
| arestas, integridade, ciclos | `graph/` |
| BM25, âncoras, filtros, views, RRF | `retrieval/` |
| escrita, dedup, update/supersede, forget | `write/` |
| rewind, manifest, orçamento, `context_id` | `handoff/` |
| diff, learn, compact | `maintenance/` |
| tarefas plan/epic/issue/task | `task/` |
| validação, audit, doctor, validators | `health/` |
| shelf-life, decay, purga, confiança, clusters | `lifecycle/` |
| erros e exit codes | `error.rs` |
| tempo determinístico | `time.rs` |
| redação de segredos | `logging.rs` |
| portas e fakes | `ports/` |
| acesso real a SO/rede | `adapters/` |
| embeddings (provedor, cache, fila, eval) | `embeddings/` |

## Referências

- Contrato de bytes: [`TOON.md`](../../wiki/specs/TOON.md).
- Decisões fechadas: [`plan/03_decisoes-fechadas.md`](../../plan/03_decisoes-fechadas.md).
- Políticas de engenharia: [`plan/implementation/14_revisao_tecnica.md`](../../plan/implementation/14_revisao_tecnica.md).
- Bordas: [`DIVERGENCES.md`](../../wiki/specs/DIVERGENCES.md).
