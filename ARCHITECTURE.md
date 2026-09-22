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
| `knudge-mcp` | Servidor MCP reativo (E12-T03) | Protocolo MCP; nada de domínio |

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
| `HookRunner` | hooks externos (sem shell) | `NoopHookRunner` |
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
| `task` | hierarquia `plan ⊃ epic ⊃ issue ⊃ task` como view derivada | E08 |
| `health` | validators, evidência, `audit`, `doctor`, leitura tolerante e âncoras por hash | E09 |
| `lifecycle` | confiança derivada (E09); decay e clusters | E09/E10 |
| `embeddings` | provedor plugável e fila lazy | E11 |

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
| Ordem | Bloco de 8 arestas logo após `superseded_by`, antes de `revision` (27 chaves — D98). |
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
| RRF | `1/(k+rank+1)`, `k=60`; desempate `(score desc, id asc)`; canal ausente só não soma (D81). |
| Filtros | `type`/`classification`/`status`/`tags`/`anchors` antes do BM25; `container` via `depends_on` transitivo (D41). |
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
| Tarefas | `kd write` rejeita `task`/`container` (D93); `scope` só vale para eles. |

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
| Tarefas | `plan`/`epic` = `container`, `issue`/`task` = `task`; pai por marcador no corpo + aresta `results_in`; `blocks` 1-based; profundidade máx. 4 (D52/D53/D93). |

## 11. Validação, saúde e leitura tolerante (E09)

| Conceito | Regra |
|---|---|
| Catálogo de validators | `.knudge/validators.toml` (subset TOML — D99): `cmd` + `scope` + `severity` + `timeout`; `globals` no topo. |
| Resolução de `checks` | `explícitos ∪ globais ∪ por_âncora(anchors)` (D54); explícito ausente vira `missing[]`. |
| Fechamento por evidência | `close_task` grava `evidence` + `outcomes[]` e **infere** `outcome` pela severidade (D48/D55); sem evidência, não fecha. |
| Confirmação | **Derivada** de `outcomes` (`success + partial*0.5`) — nunca armazenada (D48). |
| `audit` | Leitura pura: integridade, ciclos, âncoras quebradas, duplicatas, arestas sugeridas faltantes e locks stale (D46). |
| `doctor --fix` | 10 checks; corrige `body_hash`, âncoras quebradas, locks stale e índice divergente; **idempotente** (D19/D84). |
| Leitura tolerante | Chave desconhecida → warning; `type` desconhecido/nota malformada → **skip + orientação**, sem derrubar o comando (D16–D18). |
| Âncoras | `path` na nota, `content_hash` em `.idx/anchors.jsonl`; `cited` invalida, `context` não; stale **sinaliza**, não apaga (D86). |
| Confiança derivada | `sim × drift × idade + feedback`, pisos, sempre `[0,1]`, calculada no `recall` (D87). |

## 12. Fluxo de uma operação

```
kd <verbo>
  → knudge-cli: parsing (clap) + envelope (--json)
  → monta adaptadores (SystemClock, StdFs, StdEnv, TracingLogger, …)
  → chama o domínio (knudge-core) que só usa ports
  → saída: stdout = dados | stderr = logs
  → exit code = ErrorKind::exit_code() (101 reservado a panic)
```

## 13. Invariantes de engenharia

- `#![forbid(unsafe_code)]` em `core`/`cli`/`mcp` (R01).
- Sem `Rc`/`RefCell` no core; estado compartilhado via `Arc<Mutex<_>>` (R03).
- Sem `unwrap`/`expect`/`panic` em `src/` (D92); poison com `into_inner` (R32).
- Arquivos de produção ≤ 300 linhas (D92).
- `clippy -D warnings` lendo `clippy.toml` (R44); perfis e supply chain (R40–R43).

## 14. Referências

- Visão: `plan/00_panorama.md`
- Decisões: `plan/03_decisoes-fechadas.md` (D01–D98)
- Contrato de bytes: `TOON.md`
- Políticas de engenharia: `plan/implementation/14_revisao_tecnica.md` (R01–R44)
- Superfície CLI: `plan/implementation/16_cli_surface.md`
