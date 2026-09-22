# knudge-core — núcleo puro

Escopo temático do **núcleo**, sem terminal, `argv`, relógio global ou RNG global (D65). Todo
acesso ao mundo externo atravessa uma **porta** (`ports`). As implementações reais (`adapters`)
existem no mesmo crate, mas o domínio **nunca** as importa — só as portas.

## Módulos

| Módulo | Papel |
|---|---|
| `error` | `Error`/`ErrorKind`, mapa código→exit e helper de poison (R30–R35). |
| `time` | `Timestamp` UTC com milissegundos (D07). |
| `logging` | `Redactor` — redação de segredos no layer de log (R22). |
| `ports` | Traits `Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger` + fakes. |
| `adapters` | Implementações reais (`std`) das portas — **não** usadas pelo domínio. |
| `schema` | Schema canônico, tipos, IDs (E02). |
| `toon` | Parser/emissor TOON próprio (E02). |
| `jsonl` | Leitura/escrita JSONL com streaming (E03). |
| `git` | Regras de worktree/persistência (E04). |
| `retrieval` | BM25, âncoras, RRF (E06). |
| `lifecycle` | Decay, confiança derivada, clusters (E10). |
| `embeddings` | Contrato do provedor plugável (E11). |

## Invariantes

- `#![forbid(unsafe_code)]` (R01).
- Sem `Rc`/`RefCell`; estado compartilhado via `Arc<Mutex<_>>` (R03).
- Sem `unwrap`/`expect`/`panic` no código de produção (D92).
