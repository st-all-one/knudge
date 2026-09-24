# knudge-core — núcleo puro

Escopo temático do **núcleo**, sem terminal, `argv`, relógio global ou RNG global (D65). Todo
acesso ao mundo externo atravessa uma **porta** (`ports`). As implementações reais (`adapters`)
existem no mesmo crate, mas o domínio **nunca** as importa — só as portas.

## Módulos

| Módulo | Papel |
|---|---|
| `error` | `Error`/`ErrorKind`, mapa código→exit e helper de poison (R30–R35). |
| `time` | `Timestamp` UTC com milissegundos (D07). |
| `logging` | `Redactor` — redação de segredos no layer de log (R22), com rótulo tipado `[REDACTED:<tipo>]` (D159). |
| `ports` | Traits `Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger` + fakes. |
| `adapters` | Implementações reais (`std`) das portas — **não** usadas pelo domínio. |
| `schema` | Schema canônico, tipos, IDs (E02). |
| `toon` | Parser/emissor TOON próprio (E02). |
| `jsonl` | Leitura/escrita JSONL e codec JSON canônico (E03). |
| `store` | Notas, eventos, lock, rebuild, purge e sweep (D160: só `*.tmp`/`*.stale`, nunca `*.lock`); layout por tipo (`notas/<tipo>/<id>.md`) com leitura tolerante ao layout plano legado e `read_optional` (D150) (E03). |
| `config` | Config em dois níveis, schema e codec TOML próprio (E04). |
| `git` | Worktree principal, `info/exclude`, `AGENTS.md`, `sync` (E04). |
| `graph` | Arestas explícitas, integridade, ciclos, sugestões e **item de trabalho** (`scope`/`is_work_item`, D120) (E05). |
| `retrieval` | Índice derivado, BM25, âncoras, filtros, views, RRF, pipeline de hits, ranking por confiança (`rank`, D107), vocabulário de tags (`tag_counts`, D107), `why` semântico (D121), stopwords/fragmentos (`content_terms`, D122), parcelas de canal por hit (`HitChannels`, D151) e **consulta temporal** (`temporal`, D155). |
| `write` | Escrita idempotente, dedup em duas fases, update/supersede, ciclo de vida, `outcomes[]` para qualquer nota (D103) e lote JSONL (`batch`, D110). |
| `handoff` | `rewind` (manifest com `next:`/`fresh:` — D106, escopo/files), orçamento sem tokenizer e `context_id` (E08). |
| `maintenance` | `diff`, `learn` (write-gap, quase-duplicata, lacuna de grafo e tarefa→conhecimento — X2/D111) e `compact` como propostas (E08). |
| `task` | Hierarquia `epic ⊃ { issue ⊃ task \| task }`, **contexto** (`context_of`: pai/bloqueadores/filhos, D125), **progresso por épico** (`progress_of`/`epic_of`, D127), ciclo de vida, **espécie** (`--kind`, D113), **papel/modo** derivados (D115/D116/D136), **impacto** (`impact`/`is_actionable`, D106/D109/D120), **tags/âncoras** (D104), **templates de plano** (`--prompt`/`--from`, D105), **lote** (`--params`/`--batch`, D141) e **programas externos** (D119). |
| `health` | Validators (com `kind = check|gate` — D156), evidência, portão de propostas (`gate`, D156), `audit`, `doctor --fix` (inclui `program-anchor` — D119), leitura tolerante e âncoras por hash (E09). |
| `lifecycle` | Shelf-life (com **renovação por uso** — `usage`/`renew_on_use`, D154), frescor (`freshness`, D106), confirmação por tarefa (`from_tasks`, D108), decay de âncoras, purga com retenção, confiança derivada e **clusters** em duas fases (`structural_clusters`/`container_of`/`semantic_clusters`, D128). |
| `embeddings` | Provedor HTTP plugável, cache por `(body_hash, model)` (versionável em `.knudge/emb_cache.jsonl` — D148/D153), fila lazy, purga/flush, ranking de query (`rank_query`, D102) e **sugestões semânticas** (`suggest`, D158) (E11). |
| `knowledge` | Promoção de conhecimento a regras governadas no `AGENTS.md` (`recommend`/`render_block`/`apply_block`, D157). |

## Invariantes

- `#![forbid(unsafe_code)]` (R01).
- Sem `Rc`/`RefCell`; estado compartilhado via `Arc<Mutex<_>>` (R03).
- Sem `unwrap`/`expect`/`panic` no código de produção (D92).
