# Arquitetura do knudge

> Registro da separação **núcleo puro + portas + adaptadores** (D65) e dos escopos temáticos
> (D66/D68). Complementa `plan/00_panorama.md` §2 e o grafo de dependências de
> `plan/implementation/README.md`.

## 1. Princípio

O domínio **não conhece** terminal, `argv`, relógio global, RNG global nem sistema de arquivos.
Todo acesso ao mundo externo atravessa uma **porta** (`knudge_core::ports`). Isso permite:

- testar o domínio só com **fakes** (`ports::fakes`), reprodutível byte a byte;
- trocar implementação (real ↔ fake ↔ remota) sem tocar no domínio;
- confinar `unsafe`, I/O impuro e dependências de SO a poucos módulos.

## 2. Camadas e crates

```
knudge-mcp ──┐
             ├── knudge-core (núcleo puro + portas + adaptadores std)
knudge-cli ──┘
```

| Crate | Papel | Pode conter |
|---|---|---|
| `knudge-core` | Modelo, schema, retrieval, ciclo de vida e **portas** | Lógica pura; `adapters` (std) isolado |
| `knudge-cli` | Binário `kd`; monta adaptadores e escreve a saída | `clap`, `tracing`, I/O de terminal |
| `knudge-mcp` | Servidor MCP reativo: motor de gatilhos + transporte JSON-RPC stdio (E12/E14) | Protocolo MCP; nada de domínio |

`knudge-core` é a **única biblioteca** (interna). `knudge-cli` e `knudge-mcp` são **pacotes de
binário** (só `[[bin]]`, sem `[lib]`): o que se distribui são os executáveis `kd` e `knudge-mcp`.

**Regra:** os adaptadores **não** são dependência do domínio. `knudge-core::adapters` existe para
conveniência, mas só `cli`/`mcp` o importam.

## 3. Portas (`knudge-core::ports`)

| Porta | Fornece | Fake |
|---|---|---|
| `Clock` | instante UTC (`Timestamp`) | `FixedClock` |
| `Rng` | aleatoriedade (só jitter) | `SeqRng` |
| `Env` | variáveis e argumentos | `FakeEnv` |
| `Fs` | leitura/escrita atômica, append, create-exclusive, rename, `fsync`, mtime | `MemFs`, `FaultyFs` |
| `Git` | worktree, status e execução de `git` sem shell | `FakeGit` |
| `HookRunner` | hooks externos (sem shell); `adapters::ProcessHookRunner` com timeout + kill de grupo | `NoopHookRunner` |
| `Logger` | log estruturado (stderr) | `RecordingLogger` |

## 4. Escopos temáticos (`knudge-core`)

| Módulo | Responsabilidade | Épico |
|---|---|---|
| `error` | `Error`/`ErrorKind`, mapa código→exit, poison | E01 |
| `time` | `Timestamp` UTC com milissegundos (D07) | E01/E02 |
| `logging` | redação de segredos (R22) | E01 |
| `ports` | traits + fakes | E01 |
| `adapters` | implementações `std` | E01+ |
| `config` | config em dois níveis, schema, codec TOML | E04 |
| `schema` | schema canônico, tipos, IDs | E02 |
| `toon` | parser/emissor TOON | E02 |
| `jsonl` | leitura/escrita JSONL + codec JSON canônico | E03 |
| `store` | notas (`notas/`), eventos, lock, rebuild, purge, sweep | E03 |
| `git` | worktree principal, `info/exclude`, `AGENTS.md`, `sync` | E04 |
| `graph` | arestas explícitas, integridade, ciclos, sugestões | E05 |
| `retrieval` | BM25, âncoras, filtros, views `ready`/`blocked` e RRF | E06 |
| `write` | protocolo de escrita, dedup, update/supersede, ciclo de vida | E07 |
| `handoff` | `rewind` (manifest/escopo/working set), orçamento e `context_id` | E08 |
| `maintenance` | `diff`, `learn` e `compact` (propostas) | E08 |
| `task` | hierarquia `epic ⊃ { issue ⊃ task | task }` como view derivada | E08 |
| `health` | validators, evidência, `audit`, `doctor`, leitura tolerante e âncoras por hash | E09 |
| `lifecycle` | shelf-life, decay de âncoras, purga com retenção, confiança derivada e clusters | E09/E10 |
| `embeddings` | provedor HTTP plugável, cache por `body_hash`, fila lazy e avaliação A/B | E11 |

## 5. Persistência (E03)

`notas/` é a verdade; `eventos/` é auditoria; `.idx/` é derivado e reconstruível.

| Peça | Regra |
|---|---|
| Escrita de nota | tmp + rename no mesmo diretório (D20); `fsync` em batch (D22) |
| Ordem de commit | **nota antes do evento** (D21); crash deixa nota sem evento |
| Lock | advisory por arquivo-alvo, stale 30 s, reclaim por rename (D23–D25) |
| Evento | `{id, op, note_id?, at, actor?, data?}`; `id` derivado do conteúdo |
| Dedup | on-read por `id` (D26/D28), idempotente sob `merge=union` (D31) |
| Rotação | `eventos/events.jsonl` ativo → `events-NNNN.jsonl` acima de `max_bytes` (R13) |
| Checkpoint | `.idx/events.checkpoint` guarda o último evento processado |
| Remoção | canônico primeiro, depois purga do derivado (D84) |
| Rebuild | double-buffer `.idx.new/` + rename atômico (D27) |
| Resíduos | `*.tmp`/`*.lock` velhos removidos no start, com `warn` (R10) |

`revision` é **contador de versões** (default 1; cada `update` incrementa) — sinal de
volatilidade, não CAS (D48).

## 6. Configuração e worktree (E04)

| Nível | Caminho | Papel |
|---|---|---|
| Global | `$XDG_CONFIG_HOME/local/knudge/config.toml` (ou `~/.config/…`) | template/default curado |
| Projeto | `<raiz>/.knudge/config.toml` | efetivo, **precedência** (D61) |

- **Resolução:** `<raiz>` = worktree principal (`git rev-parse --git-common-dir`); worktrees
  ligados compartilham `.knudge/`; submódulo resolve no próprio top-level (D29).
- **Instanciação:** `onboard` clona o global **literalmente**; não sobrescreve projeto sem
  `--force` (D62); sem global, usa defaults.
- **Segredos:** vivem só no global (D91); o projeto é sanitizado em `Config::effective`.
- **Codec:** subset TOML próprio; leitura preserva ordem (diff mínimo), escrita canônica (D97).
- **Persistência (D34):** `persist_in_project=true` versiona `notas/`/`eventos/` e exclui o
  derivado via `.git/info/exclude` (nunca `.gitignore` — D30); `false` exclui `.knudge/` inteiro.
- **`sync` (D32):** `git -C <raiz> add .knudge/notas .knudge/eventos` + commit com mensagem
  gerada do último evento; `merge=union` em `events*.jsonl` (D31).
- **`AGENTS.md` (D60):** bloco entre `<!-- knudge:start -->`/`<!-- knudge:end -->` com version
  marker; reexecutar não duplica nem sobrescreve o conteúdo do usuário.

## 7. Grafo e arestas (E05)

| Conceito | Regra |
|---|---|
| Fonte | Arestas explícitas no **frontmatter** (chave = `EdgeKind`, valor = lista de ids); a nota é a verdade (D49/D98). |
| Vocabulário | Fechado: `references, depends_on, contradicts, supports, extends, replaces, rejects, results_in` (D51). |
| Ordem | Bloco de 8 arestas logo após `superseded_by`, antes de `revision` (**25 chaves** — D98/D135/D142). |
| `link()` | Adiciona sem duplicar; rejeita id inválido e auto-aresta. |
| `expand` | BFS determinística só no **explícito**, com corte por `depth` e filtro por tipo. |
| Supersessão | `replaces` (novo → antigo) e `superseded_by` (antigo → novo); bidirecionalidade cobrada pela integridade (D46). |
| Ciclos | SCC (Kosaraju iterativo) sobre `replaces`/`depends_on`; membros **não demovem** (D45). |
| Sugestões | Extração conservadora (ids, wikilinks, verbos) vai para `.idx/suggestions.jsonl` — derivado, purgável; **nunca** vira aresta (D49/D50/D84). |

## 8. Retrieval: BM25, âncoras e RRF (E06)

| Conceito | Regra |
|---|---|
| Índice | Derivado em `.idx/retrieval.jsonl` (uma linha JSON por nota), reconstruível byte a byte; ausente → reconstrói (D15/D27). |
| Tokenização | ASCII explícita `[a-z0-9_]`; `café` → `caf` (D36). |
| BM25 | `k1=1.5`, `b=0.75`, **IDF por campo** (`statement` domina, peso 3) e **peso por tipo** (D35/D37). |
| Boost | `score * (1 + 0.1 * (success + partial*0.5))` a partir de `outcomes` (D38). |
| Âncoras | Canal determinístico por `path`/`id` com globs `?`/`*`/`**` (D81/D86). |
| RRF | `Σ peso_canal/(k+rank+1)`, `k=60`, `semantic_weight=30` (D81/D124); desempate `(score desc, id asc)`; canal ausente só não soma. |
| Filtros | `type`/`classification`/`status`/`tags`/`anchors` antes do BM25; `scope` via `depends_on` transitivo (D41/D149). |
| Views | `ready`/`blocked` computadas do `depends_on` transitivo; ciclo de dependência = `blocked` (D53). |
| Contrato | `recall` em pipe `id\|statement\|score\|why`; `why` fechado (`file_match|anchor_match|tracker_match|stars|recent|universal`); corpo só via `get` (D39). |
| Degradação | Canal falho → resultado parcial + `warnings`; `strict` (D94) promove a erro; teto de índice avisa (E06-T07). |

## 9. Escrita e protocolo (E07)

| Conceito | Regra |
|---|---|
| Idempotência | `id` endereçado por `type + statement` (D01); mesmo `body_hash` → `unchanged`; mesmo id com corpo diferente → conflito (use `update`). |
| Duas fases | `propose` (recall + score) → decisão → `write`; `< create_below` cria, `[create_below, merge_below)` merge, `≥ merge_below` rejeita (D26). |
| Score lexical | Dice sobre o conjunto de termos (calibrado para `0.75`/`0.92`); limiares vêm de `[dedup]` (D80). |
| Merge | Funde no candidato (tags/âncoras unidas, corpo acrescido, confiança máx) e incrementa `revision`. |
| `update` | Mesma chave de conteúdo → edita no lugar (`revision++`); chave nova → **supersede** (novo id + `replaces`/`superseded_by` — D01/D48). |
| Ciclo de vida | `forget`/`restore` são soft (`status`), nunca apagam; transições protegidas (`superseded` só via supersede). |
| Estrito | Chave/tipo desconhecidos rejeitados; opcionais vazios **omitidos** (D05/D16/D17). |
| Reconciliação | `propose_merges` só **propõe** quase-duplicados; nada é fundido sem aprovação (D47/D80). |
| Tarefas | `kd write` rejeita `task`/`epic` (D93/D149); `scope` só vale para eles. |

## 10. Handoff, manutenção e tarefas (E08)

| Conceito | Regra |
|---|---|
| `prime` vs `rewind` | `prime` é **protocolo estático** byte-idêntico; `rewind` é o **estado dinâmico** (D57). |
| Três modos | manifest (~30 tokens: contadores + recentes + dirty), escopo (container/domínio) e working set (âncoras) (D57). |
| Ranking | `star*100 + foundational*50 + tactical*20 + observational*10`, desempate `created_ms desc, id asc`. |
| Orçamento | `ceil(chars/4)`, default 4000; trunca o último item e ignora sobra < 100 tokens (D40/D82). |
| Auto-scope/flip | deriva o container dos arquivos tocados; vira manifest quando `>100 notas` ou `>5 containers` (D41). |
| `context_id` | id derivado do texto, guardado em `.idx/contexts/`; `--resume` devolve **bytes idênticos** (D88). |
| `diff` | lê a auditoria de eventos por intervalo/escopo — nunca o git global (D33). |
| `learn` | propõe `create_note`/`merge`/`supersede`/`link` de eventos + âncoras; nunca escreve (D33/D47). |
| `compact` | propõe `concat`/`keep_latest`/`merge_outcomes`; só aplica sob aceite (D47). |
| Tarefas | `epic` = grupo (`scope=epic`, sem `type`), `issue`/`task` = `task`; pai por marcador no corpo + aresta `results_in`; `blocks` 1-based; `epic ⊃ {issue ⊃ task \| task}` (D52/D53/D93/D134/D149). |

## 11. Validação, saúde e leitura tolerante (E09)

| Conceito | Regra |
|---|---|
| Catálogo de validators | `.knudge/validators.toml` (subset TOML — D99): `cmd` + `scope` + `severity` + `timeout`; `globals` no topo. |
| Resolução de `checks` | `explícitos ∪ globais ∪ por_âncora(anchors)` (D54); explícito ausente vira `missing[]`. |
| Fechamento por evidência | `close_task` grava `evidence` + `outcomes[]` e **infere** `outcome` pela severidade (D48/D55); sem evidência, não fecha. |
| Confirmação | **Derivada** de `outcomes` (`success + partial*0.5`) — nunca armazenada (D48). |
| `audit` | Leitura pura: integridade, ciclos, âncoras quebradas, duplicatas, arestas sugeridas faltantes e locks stale (D46). |
| `doctor --fix` | 11 checks (inclui o tamanho do índice vetorial/cache — E11); corrige `body_hash`, âncoras quebradas, locks stale e índice divergente; **idempotente** (D19/D84). |
| Leitura tolerante | Chave desconhecida → warning; `type` desconhecido/nota malformada → **skip + orientação**, sem derrubar o comando (D16–D18). |
| Âncoras | `path` na nota, `content_hash` em `.idx/anchors.jsonl`; `cited` invalida, `context` não; stale **sinaliza**, não apaga (D86). |
| Confiança derivada | `sim × drift × idade + feedback`, pisos, sempre `[0,1]`, calculada no `recall` (D87). |

## 12. Ciclo de vida, decay e clusters (E10)

| Conceito | Regra |
|---|---|
| Shelf-life | `foundational` nunca expira (`0` dias); `tactical` (365) e `observational` (30) com prazos por config (D44). A expiração é **sempre derivada** de `created_at` + prazo (D135). |
| Decay de âncoras | Valida literais (existe) e globs (casa); demove após grace se a fração válida < threshold (D43). Varredura do projeto limitada e off-path. |
| Demolição | Sempre **soft** (`forget`); membros de ciclo de supersessão/dependência são **protegidos** (D45). |
| Purga | `retired_at` derivado de eventos (`forget`/`supersede`); conteúdo só sai após a janela de retenção e a remoção purga o derivado (D48/D84). |
| Clusters fase 1 | Agrupamento determinístico por `anchor`/`type`/`classification`/scope — sem estatística nem embeddings (D47). |
| Clusters fase 2 | Semântico **dentro** de um cluster estrutural, acima do volume mínimo e off-path; similaridade injetada (E11). |

## 13. Embeddings (E11)

| Conceito | Regra |
|---|---|
| Provedor | `http` (default; servidor local OpenAI-compatible — `llama-server`/TEI/Ollama), `lightweight` (hash, testes/CI) ou `none` (BM25). Sem inferência in-process (D101/R16/R43). |
| Porta | Trait `Embedder` (`ports`); o domínio nunca fala HTTP. `adapters::http::HttpEmbedder` é cliente HTTP/1.1 bloqueante sobre `std::net` (timeout + retry idempotente). |
| Índice | `.idx/embeddings.jsonl` com cabeçalho `meta` (provider/model/revision/dimensões/similaridade); mudança de modelo **invalida** e força re-embed (D79). Default `granite-embedding-97m-multilingual-r2` (D123). |
| Cache | `.idx/emb_cache.jsonl` por `body_hash`, com teto e eviction LRU; falha degrada para *pass-through* (D83/R14). |
| Fila | Estado `indexed\|pending\|stale` **derivado** do `body_hash`; falha do provedor marca `pending`, nunca descarta (D80/D83). `max_pending` é backpressure; acima, catch-up. |
| Flush | *Dirty flag* + debounce `flush_ms`, com flush forçado na saída (D85). |
| Purga | Toda remoção passa por `purge_derived`, que apaga registros com `id` do índice vetorial (D84). |
| Avaliação | `Recall@k`/`nDCG@k`/`MRR` puras, com ranqueador injetado — alicerce do `kd maintenance eval --ab` (D90). |

## 14. CLI, MCP e hooks (E12)

| Conceito | Regra |
|---|---|
| Superfície | 12 verbos (`16_cli_surface.md`): `kd`(=prime), `init`, `prime`, `rewind`, `ask`, `write`, `task`, `maintenance`, `config`, `forget`, `sync`, `self`. |
| Sessão | `knudge-cli::session::Session` resolve o projeto, carrega a config efetiva e monta store/eventos/índice/grafo sob demanda. É a única borda que toca adaptadores reais. |
| Saída | **stdout = dados, stderr = logs** (R20); `--json` emite o envelope `{success, command, data?, error{code,message,retryable}, warnings?}` (D71/R31). EPIPE → exit 0 (D73). |
| `strict` | Config de projeto (`behavior.strict`, D94) promove `warnings[]` a erro; **não** existe flag `--strict`. |
| Hooks | `HookRunner` (porta) + `ProcessHookRunner` (adaptador; timeout + kill do grupo de processos, sem shell — D59/R12). Orquestração em `commands/hooks.rs`: `pre-record` (bloqueia/muta), `post-record`, `pre-prune`, `pre-compact`; `pre-prime` é reservado (`prime` é byte-idêntico — D57). |
| MCP | `knudge-mcp::triggers::HintEngine` — 3 gatilhos (pré-`write`, pré-edição, fim de sessão), hints **ponteiro**, cap 3, dedup por sessão e modo observação (D68). |
| MCP (transporte) | Binário `knudge-mcp`: JSON-RPC 2.0 sobre stdio, **uma linha por mensagem**, `initialize`/`ping`/`tools/list`/`tools/call`; `knudge_pre_write`/`pre_edit`/`session_end`/`status`; `EPIPE`/EOF → exit 0 (E14, D68/D71/D73). |
| Distribuição | `kd self completions <bash|zsh|fish>` e `kd self setup <claude|cursor|codex|pi>` gravam recipes em `.knudge/setup/` (D69). |

## 15. Fluxo de uma operação

```
kd <verbo>
  → knudge-cli: parsing (clap) + envelope (--json)
  → monta adaptadores (SystemClock, StdFs, StdEnv, TracingLogger, …)
  → chama o domínio (knudge-core) que só usa ports
  → saída: stdout = dados | stderr = logs
  → exit code = ErrorKind::exit_code() (101 reservado a panic)
```

## 16. Invariantes de engenharia

- `#![forbid(unsafe_code)]` em `core`/`cli`/`mcp` (R01).
- Sem `Rc`/`RefCell` no core; estado compartilhado via `Arc<Mutex<_>>` (R03).
- Sem `unwrap`/`expect`/`panic` em `src/` (D92); poison com `into_inner` (R32).
- Arquivos de produção ≤ 300 linhas (D92).
- `clippy -D warnings` lendo `clippy.toml` (R44); perfis e supply chain (R40–R43).

## 17. Qualidade e verificação (E13)

| Camada | O que trava | Onde |
|---|---|---|
| Golden | bytes de `prime`/`--json`/erros/`init`/EPIPE | `crates/knudge-cli/tests/golden.rs` + `tests/golden/` |
| Property | TOON, RRF, decay/confiança, ids/hash sob normalização | proptests nos `src/<mod>/tests*` + `proptest-regressions/` |
| Stress | lock sem *lost update*, escritas concorrentes, leitor × rebuild | `crates/knudge-core/tests/stress.rs` |
| Crash-injection | nota-antes-de-evento, tmp+rename, rebuild double-buffer | `FaultyFs` em `store/tests/*` |
| Bordas | catálogo com o teste que trava cada uma | [`DIVERGENCES.md`](DIVERGENCES.md) |
| Aceite | pipe/`--json`/erro/exit/estado por verbo | [`plan/implementation/17_matriz_aceitacao.md`](plan/implementation/17_matriz_aceitacao.md) |
| Dinâmica | `miri` no core puro; fuzz de TOON/JSONL; supply chain (`deny`/`audit`/`machete`/`typos`) | `.github/workflows/ci.yml`, `fuzz/`, `deny.toml` |

O gate local é `make check` (fmt + clippy + test + linhas); `make ci` soma os alvos extras
(que pulam se a ferramenta não estiver instalada).

## 18. Referências

- Visão: `plan/00_panorama.md`
- Decisões: `plan/03_decisoes-fechadas.md` (D01–D101)
- Contrato de bytes: `TOON.md`
- Políticas de engenharia: `plan/implementation/14_revisao_tecnica.md` (R01–R44)
- Superfície CLI: `plan/implementation/16_cli_surface.md`
- Matriz de aceite: `plan/implementation/17_matriz_aceitacao.md`
- Bordas: `DIVERGENCES.md`
