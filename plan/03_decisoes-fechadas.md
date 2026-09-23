# Decisões fechadas — knudge

> Respostas aos 78 pontos de `02_decisoes.md`. Este é o contrato de projeto: a partir daqui, código e documentação derivam daqui.
>
> Legenda: ✅ decidido · ⚠️ com ponto de atenção registrado.

---

## A. Identidade e IDs

| # | Decisão final |
|---|---|
| **D01** ✅ | **ID endereçado por conteúdo** — hash curto de `type + statement`. Torna o `write` idempotente sob retry. Se a chave mudar, cria novo id + `superseded_by` (híbrido: identidade pelo conteúdo, linhagem explícita). |
| **D02** ✅ | **Prefixo acompanha o `type`** e é **histórico**: reclassificar o tipo **não** reescreve o id. |
| **D03** ✅ | **Formato fixo** — `<prefixo>_<base36(8)>`; colisão detectada no write. |

## B. Schema e serialização (contrato de bytes)

| # | Decisão final |
|---|---|
| **D04** ✅ | **Ordem canônica de declaração do schema**; round-trip preserva; nota nova insere na ordem canônica. |
| **D05** ⚠️ | **Não existe nota parcial/opcional.** Ou a nota é válida e existe, ou não existe. Campos opcionais são **omitidos**, nunca `null`/vazio. Sem `draft`, sem nota incompleta. |
| **D06** ✅ | `body_hash` = hash de **`statement` + corpo**, após **NFC + trim + colapso de whitespace**. Congelado por teste. |
| **D07** ✅ | Timestamp **UTC, com milissegundos, sufixo `Z`**. |
| **D08** ✅ | `statement ≤ 120` conta **escalares Unicode**. |
| **D09** ✅ | **Inteiros** normalizados na decodificação; nunca emitir `.0`. |
| **D10** ✅ | **Raw UTF-8**; escapar só `"`, `\` e controles. |
| **D11** ✅ | Lista vazia → **arquivo zero bytes**. |
| **D12** ✅ | **Normalizar** o newline final no append. |
| **D13** ✅ | **Ordem canônica sempre**, inclusive em nota nova. |

## C. Versionamento e migração

| # | Decisão final |
|---|---|
| **D14** ✅ | **Sem retrocompatibilidade, sem aliases.** Greenfield: começa já no padrão atual; migração é em massa, não retroativa. |
| **D15** ✅ | **On-read com defaults** (sem aliases, por D14); rebuild só quando o formato do índice muda. |
| **D16** ✅ | Chave desconhecida no read: **tolerar com warning**. |
| **D17** ⚠️ | Tipo desconhecido no read: **rejeitar**. (Rejeição é por-nota, com erro explícito — não pode derrubar a leitura inteira; ver D18.) |
| **D18** ✅ | Nota/linha malformada: **skip com warning + orientação de correção** (exemplo/placeholder de como consertar). |
| **D19** ✅ | `doctor --fix`: **corrige** o reversível (hash, âncoras quebradas, locks stale, duplicatas). |

## D. Escrita, atomicidade e crash

| # | Decisão final |
|---|---|
| **D20** ✅ | Escrita atômica: **tmp + rename no mesmo diretório**. |
| **D21** ✅ | Commit multi-arquivo: **nota primeiro, evento depois**; container é derivado. |
| **D22** ✅ | `fsync` em **batch** (write em page cache; fsync no rebuild/sync). |

## E. Concorrência

| # | Decisão final |
|---|---|
| **D23** ✅ | **Lock advisory por arquivo-alvo.** |
| **D24** ✅ | Reclaim de stale: **rename sidecar + inode/mtime**; nunca apagar lock alheio. |
| **D25** ✅ | **Ordem de aquisição de locks documentada** (evitar ABBA). |
| **D26** ✅ | Dedup: **on-write** para notas + **on-read** para `events.jsonl`/containers. |
| **D27** ✅ | Rebuild do índice: **double-buffer** (`.idx.new/` + rename). |
| **D28** ✅ | Containers: **deltas/eventos** (idempotente sob merge). |

## F. Git, diretórios e persistência

| # | Decisão final |
|---|---|
| **D29** ✅ | `.knudge/` resolve no **worktree principal** (`git rev-parse --git-common-dir`); submódulo não conta. |
| **D30** ⚠️ | **Exclusão absoluta via `.git/info/exclude`** — nunca `.gitignore`. O `.knudge/` inteiro e tudo dentro dele é atrelado ao `info/exclude`. (Interação com D34 — ver §Pendências.) |
| **D31** ✅ | `merge=union` para **`events.jsonl`**; containers via deltas; `.idx/` gitignored. |
| **D32** ✅ | `sync` commita `notas/` + `eventos/`; mensagem gerada do evento; guard de worktree. |
| **D33** ✅ | `learn()` a partir de **eventos + `anchors`** (determinístico), não do diff global. |
| **D34** ✅ | `persist_in_project`: `true` = versionado; `false` = local-only. (Ver D30.) |

## G. Retrieval e ranking

| # | Decisão final |
|---|---|
| **D35** ✅ | **BM25** (`k1=1.5, b=0.75`). |
| **D36** ✅ | Tokenização **ASCII explícita** (replicar `\w`); documentar acentos. |
| **D37** ✅ | **IDF por campo** (`statement` domina) + por tipo. |
| **D38** ✅ | **Boost por confirmação**: `score * (1 + 0.1 * (success + partial*0.5))`. |
| **D39** ✅ | `recall` com **4ª coluna `why`** (`id\|statement\|score\|why`). |
| **D40** ✅ | **Orçamento de tokens** no `prime` (`ceil(len/4)`, default 4000). |
| **D41** ✅ | **Auto-context-scope** (`git status -uall` + active work) e **auto-flip** (>100 notas/>5 containers). |
| **D42** ✅ | Embeddings **configuráveis desde já** via `config.toml` (provedor plugável); sempre derivados/opcionais. `enabled=false`/`provider=none` cai para BM25 puro. |

## H. Ciclo de vida e saúde

| # | Decisão final |
|---|---|
| **D43** ✅ | **Decay de âncoras** no rebuild; demove após grace se fração válida < threshold. |
| **D44** ✅ | Adicionar **`observational`** às classificações (foundational/tactical/observational). |
| **D45** ✅ | Supersessão com **detecção de ciclos** (Tarjan/DFS); membros de ciclo não demovem. |
| **D46** ✅ | **Integridade do grafo**: referencial + bidirecionalidade `sup↔superseded_by` + dangling. |
| **D47** ✅ | `compact` **propõe** (concat/keep_latest/merge_outcomes); agente aceita. |
| **D48** ✅ | **`outcomes[]`** (status/duration/agent/notes/recorded_at); confirmação **derivada**. |

## I. Grafo e arestas

| # | Decisão final |
|---|---|
| **D49** ✅ | **Arestas explícitas primárias + regex como sugestão.** O `expand` confia no explícito; `audit` sugere. |
| **D50** ✅ | Extração roda **no write**, gravando como **sugestão revisável**. |
| **D51** ✅ | Vocabulário **fechado e declarativo** (não 3 letras): `references`, `depends_on`, `contradicts`, `supports`, `extends`, `replaces`, `rejects`, `results_in`. |

## J. Tarefas e planos

| # | Decisão final |
|---|---|
| **D52** ✅ | Container/plan = **view derivada** (id + eventos de filiação); backref por marcador. |
| **D53** ✅ | Ciclo de vida completo do plan: submit/adopt/reorder/release/outcome/review; profundidade máxima; `blocks` 1-based; self-reference. |
| **D54** ✅ | `checks` = **catálogo executável** + `AGENTS.md` + `anchors`; severidade. |
| **D55** ✅ | Fechamento por **evidência** (roda validators, infere `outcome`). |
| **D56** ✅ | Scheduling separado de expiração (`not_before` vs `expires_at`) quando surgir. |

## K. Prime, protocolo e hooks

| # | Decisão final |
|---|---|
| **D57** ✅ | **`prime` é o protocolo estático** (token-optimized, byte-idêntico por versão do binário, estilo `help`): tipos, tools, regras, orçamento. `kd` sem argumentos executa `kd prime`. O **estado dinâmico** (situar agentes/rodadas) passa a ser **`kd rewind`**; o handoff 1:1 por `context_id` é `kd rewind --resume <id>`. `kd init` funda `.knudge/` + `AGENTS.md` (marcadores idempotentes). Protocolo e estado ficam **separados**. |
| **D58** ✅ | Session-close como **footer curto do `prime`** + hook opcional. |
| **D59** ✅ | **Hooks opcionais** (`pre-record`, `post-record`, `pre-prime`, `pre-prune`, `pre-compact`); stdin JSON; timeout + process-group kill; redaction. |
| **D60** ✅ | `onboard` com **marcadores idempotentes** + version marker. |

## L. Config

| # | Decisão final |
|---|---|
| **D61** ✅ | **Global (template) + projeto (efetivo, precedência)**; projeto clona o global na instanciação. |
| **D62** ✅ | Instanciação por **cópia literal**; merge fica como evolução futura. |
| **D63** ✅ | Writer TOML com **ordem canônica e quoting estáveis**. |
| **D64** ✅ | `config set/unset` **valida contra o schema**, poda ancestrais vazios, revalida. |

## M. Arquitetura e distribuição

| # | Decisão final |
|---|---|
| **D65** ✅ | **Isolamento por escopo temático** e documentação por escopo: `core`, `cli`, `mcp`, `jsonl`, `toon`, `git`, `retrieval`, `embeddings`, `lifecycle`, etc. Núcleo puro + adaptadores finos. |
| **D66** ✅ | **Rust.** Objetivo: **um único binário executável chamado `kd`**. |
| **D67** ✅ | Nome: **`kd`**. |
| **D68** ✅ | **MCP + CLI** agora; **FFI e WASM apenas a nível de planejamento**. |
| **D79** ✅ | **Provedor de embedding plugável via `config.toml`** (`local`/`http`/`none`); modelo default **`ibm-granite/granite-embedding-97m-multilingual-r2`** (384d, cosseno nativo, multilíngue — **D123**), com alternativas por config (`msmarco-MiniLM-L12-cos-v5`, `paraphrase-multilingual-MiniLM-L12-v2`, `embeddinggemma-300m`). `revision` pinada e re-embed quando o modelo muda. Ver `04_embeddings.md`. (**D101** fixa a execução em **HTTP local**, sem inferência in-process.) |
| **D80** ✅ | **Embedding assíncrono e lazy; nunca bloqueia.** `write`/`recall`/rebuild seguem sem esperar o modelo; notas recém-criadas ficam **“dark”** no espaço vetorial até serem digeridas por uma fila (gap tolerado em rajadas de 10–20). Dedup no write é **lexical**; o semântico é **eventual** (reconciliação). Estado `embedded\|pending\|stale` é derivado, em `.idx/`; `prime` reporta `embeddings_pending`. |
| **D69** ✅ | Binário estático + `kd self setup` (recipes `claude`/`cursor`/`codex`/`pi`) + `kd self completions` + `kd self upgrade` + `kd self version`. |
| **D70** ✅ | **Sem migração.** O knudge é **independente** de seeds e mulch. |

## N. Contrato de saída e erros

| # | Decisão final |
|---|---|
| **D71** ✅ | **Pipe para LLM + `--json`** (`{success, command, error}`) para MCP/scripts. |
| **D72** ✅ | **Catálogo de mensagens congelado por teste.** |
| **D73** ✅ | **EPIPE → exit 0.** |

## O. TOON

| # | Decisão final |
|---|---|
| **D74** ✅ | Parser/emitter **próprios**, subconjunto documentado + corpus/golden. |
| **D75** ✅ | Definir **fallback e detecção de versão** de frontmatter. |

## P. Testes e qualidade

| # | Decisão final |
|---|---|
| **D76** ✅ | **Golden/snapshot + property tests + stress de concorrência + crash-injection.** |
| **D77** ✅ | **`DIVERGENCES.md` do knudge** (Unicode, hash, ordem, lock, atomicidade, TOON). |
| **D78** ✅ | **Matriz de aceite por tool** (pipe, JSON, erro, exit, estado do `.knudge/`). |

---

## Q. Extrações do arags (D81–D92)

> Fechadas seguindo a recomendação de `05_refs-agnostic-rag.md`, com **simplicidade como meta global**: herdar algoritmos e invariantes, recusar a infraestrutura (servidor, SQLite, usearch, múltiplos espaços, modelo fixo).

| # | Decisão final |
|---|---|
| **D81** ✅ | **`recall` funde canais por RRF** (`1/(k+rank+1)`, k=60) com **tie-break determinístico `(score desc, id asc)`** e **degradação graciosa** — canal ausente/falho (âncoras, vetor) cai para o lexical sem quebrar a busca. |
| **D82** ✅ | **Orçamento do `prime` sem tokenizer**: manter a heurística `ceil(len/4)` (D40), aplicada item a item — trunca o último e ignora sobra < 100 tokens. `palavras × 1.3` fica como alternativa só se o corpus exigir. |
| **D83** ✅ | **Cache de embedding por `body_hash`** + estado derivado `indexed\|pending\|stale` (extensão de D80); falha de embedding **marca `pending`, nunca descarta a nota**; worker de reconcile re-embeda do corpo. |
| **D84** ✅ | **Purga do vetor em toda remoção** (supersede, merge, `compact`, TTL, dedup) — vetor é derivado e descartável; `doctor` detecta divergência canônico↔derivado e orienta rebuild. |
| **D85** ✅ | **Flush coalescido (debounce)** do `.idx`/embeddings com *dirty flag*, **flush forçado na saída** — rajadas de 10–20 notas causam um único rewrite. |
| **D86** ✅ | **`anchors` com `path` + `content_hash` derivado** (hashes em `.idx/anchors.jsonl`, **nunca** no frontmatter) e **verify-on-hit**: `cited` invalida, `context` não; nota **stale é sinalizada, não apagada**. |
| **D87** ✅ | **Confiança derivada em tempo de consulta** (evidência + feedback + idade + âncoras), **não armazenada** — separada da `confidence` declarada. |
| **D88** ✅ | **`rewind` emite `context_id` endereçável**; `kd rewind --resume <id>` devolve o **mesmo contexto 1:1**, sem re-busca — handoff reprodutível entre agentes/rodadas. |
| **D89** ✅ | **`provider = "lightweight"`** (embedder determinístico por hash) para testes/CI/offline — sem download de modelo, sem rede. |
| **D90** ✅ | **`kd maintenance eval --ab`** com Recall@k / nDCG@k / MRR sobre um golden pequeno — o jeito de decidir L6 vs L12 vs multilíngue sem chutar. |
| **D91** ✅ | **Projeto por nome lógico** (worktrees do mesmo repo compartilham `.knudge/`); **segredos só no global** — o projeto nunca carrega credenciais. |
| **D92** ✅ | **Disciplina Rust**: arquivos ≤300 linhas de produção, proibido `unwrap/expect/panic` em `src`, **proptest** nos puros (RRF, decay, confiança), clippy `-D warnings`. |

---

## R. Superfície CLI v2 (D93–D94)

> Decidida na revisão da superfície do `kd` (inspirada no Docker). Contrato congelado em
> `implementation/16_cli_surface.md`.

| # | Decisão final |
|---|---|
| **D93** ✅ | **`task` com hierarquia fechada**: novo campo `scope ∈ {plan, epic, issue, task}` (enum fechado; **não** altera o enum de `type`). `plan`/`epic` são `type=container` (view derivada, D52) + `scope`; `issue`/`task` são `type=task` + `scope`. Aninhamento `plan ⊃ epic ⊃ issue ⊃ task` (profundidade máx. **4**), pai único via membership/backref (D52). Tudo de tarefa vive em `kd task`; `kd write` **rejeita** `--type task\|container`. |
| **D94** ✅ | **`strict` é config de projeto** (`[behavior] strict = false` em `.knudge/config.toml`), **não** flag de CLI nem subcomando; vale para warnings de leitura/retrieval/embeddings. |

---

## S. Contrato de bytes (D95)

> Fecha a pendência D01 × D02 e fixa a gramática do frontmatter. Detalhes em `TOON.md`.

| # | Decisão final |
|---|---|
| **D95** ✅ | **Hash curto = SHA-256 truncado aos 4 primeiros bytes** (`u32` big-endian). `body_hash = hex8(normalize(statement) + LF + normalize(body))` (D06); `id = <prefixo>_<base36(8)>(type + U+001F + normalize(statement))` (D01). O `id` é **histórico**: reclassificar o `type` não o reescreve (D02). `normalize` = NFC + trim + colapso de whitespace. A **gramática TOON v1** (raw UTF-8, ordem canônica, inteiros sem `.0`, lista vazia omitida, newline `LF`) está em `TOON.md`. |

---

## T. Persistência e eventos (D96)

> Fecha o contrato do log de eventos e a semântica de `revision`. Implementado em E03.

| # | Decisão final |
|---|---|
| **D96** ✅ | **Evento** = `{id, op, note_id?, at, actor?, data?}` em `eventos/events.jsonl`; `id = evt_<base36(8)>` derivado do conteúdo (D95) e usado como chave de **dedup on-read**. Leitura tolerante (linha ruim → skip + warning). Rotação por tamanho: segmento ativo `events.jsonl` → `events-NNNN.jsonl`; checkpoint derivado em `.idx/events.checkpoint`. `revision` é **contador de versões** (default 1; cada `update` incrementa), não CAS. |

---

## U. Config e worktree (D97)

> Fecha o contrato do codec TOML, do `sync` e do `onboard` (implementado em E04).

| # | Decisão final |
|---|---|
| **D97** ✅ | **TOML subset próprio** (sem dependência externa): aceita comentários, `[seção]`/`[seção.sub]`, chaves bare/citadas pontilhadas, strings básicas/literais de **uma linha**, inteiros com `_`, floats, booleanos e listas (inclusive multilinha). Rejeita `[[...]]`, strings multilinha e `null` com erro `config`. A leitura **preserva a ordem** (diff mínimo) e a escrita é canônica (D63). `sync` executa `git -C <raiz>` (guard de worktree, D32); `onboard` degrada para defaults quando não há config global. |

---

## V. Arestas explícitas (D98)

> Fecha o contrato de serialização do grafo (implementado em E05).

| # | Decisão final |
|---|---|
| **D98** ✅ | **Arestas explícitas são chaves de frontmatter de primeiro nível**, nomeadas pelo `EdgeKind` (`references`, `depends_on`, `contradicts`, `supports`, `extends`, `replaces`, `rejects`, `results_in`), cada uma uma **lista de ids** (omitida quando vazia — D05). Ordem canônica: as 8 arestas ficam logo após `superseded_by` e antes de `revision` — **27 chaves**. `superseded_by` continua sendo o **ponteiro reverso** (id único) de `replaces`; a bidirecionalidade é validada pela integridade (D46). A extração textual é **sugestão derivada** em `.idx/suggestions.jsonl` (nunca aresta; D49/D50) e o `expand` percorre só o explícito. Ciclos de supersessão são detectados por SCC sobre `replaces` (D45) e os membros **não demovem**. (**D100** acrescenta `not_before` logo após `expires_at`, totalizando **28 chaves**.) |

---

## W. Catálogo de validators (D99)

> Fecha o formato do catálogo de `checks` (implementado em E09).

| # | Decisão final |
|---|---|
| **D99** ✅ | O catálogo de validators é **`.knudge/validators.toml`**, no **subset TOML próprio** (D97) — **não** YAML, para não introduzir dependência nem parser novo. Cada validator é uma tabela com `cmd` (obrigatório), `scope` (globs), `severity` (`error\|warn\|info`, default `error`) e `timeout` (ms, default 120000). A chave de topo `globals` lista os validators globais (fonte que o `AGENTS.md` renderiza). A resolução é `checks(task) = explícitos ∪ globais ∪ por_âncora(anchors(task))`; validator explícito ausente vira `missing[]` (não é erro fatal). A **execução** fica na borda (`HookRunner`, E12); o núcleo só resolve e descreve. |

---

## X. Agendamento separado de expiração (D100)

> Fecha o campo de agendamento (implementado em E10-T05).

| # | Decisão final |
|---|---|
| **D100** ✅ | Introduz **`not_before`** (RFC3339, opcional) como **28ª chave canônica**, logo após `expires_at` e antes de `superseded_by`. Agendamento e expiração são **ortogonais** (D56): `expires_at` remove do corpus, `not_before` só **retém** a tarefa em `blocked` até o instante. A view **estática** (`compute_views`, usada pelo `prime` byte-idêntico — D57) **ignora** `not_before`; a view dinâmica (`compute_views_at(now_ms)`) o considera. Sem bump de `schema_version`: é campo opcional on-read (D15), omitido quando ausente (D05). |

---

## Y. Embeddings via HTTP local (D101)

> Fecha a forma de execução do provedor (implementado em E11).

| # | Decisão final |
|---|---|
| **D101** ✅ | O provedor de embedding **não roda in-process**. `provider` é `http` (default) \| `lightweight` \| `none`; o default consome um **servidor local OpenAI-compatible** — `llama-server -m msmarco-MiniLM-L12-cos-v5.Q5_K_M.gguf --embeddings` (ou TEI/Ollama/vLLM) — via `embeddings.endpoint` (`http://127.0.0.1:8080/v1/embeddings`), com `timeout_ms`, `retries` e `api_key_env`. O cliente é HTTP/1.1 **bloqueante** sobre `std::net` (**sem** `tokio`/`reqwest` — R16/R43) e `https://` exige proxy/TLS terminator. `lightweight` (hash, D89) cobre CI/offline; `none` cai para BM25. Inferência `local` in-process (ONNX/candle/llama.cpp) é **recusada** (R16/R43). O índice é **invalidado** quando modelo/revisão/dimensão mudam (D79). |

---

## Impactos no panorama (já propagados)

| Decisão | Onde mudou |
|---|---|
| D66/D67 | Binário `kb` → **`kd`**; stack **Rust**. |
| D35/D36/D37/D38/D39/D40/D41 | §Retrieval: TF-IDF → **BM25**, tokenização ASCII, IDF por campo, boost, coluna `why`, budget, auto-scope. |
| D44 | `classification` ganha **`observational`**. |
| D48 | `outcome` + `confirmations` → **`outcomes[]`** com confirmação derivada. |
| D49/D51 | Arestas passam a **explícitas + declarativas** (`references`, `depends_on`, …). |
| D57 | `prime` reescrito: **protocolo estático** (byte-idêntico); estado/handoff migra para **`rewind`**. |
| D65 | Arquitetura organizada por **escopo temático** (`core`, `cli`, `mcp`, `jsonl`, …). |
| D42/D79 | Embeddings passam a ser **provedor plugável** via config, com default multilíngue (`granite-embedding-97m-multilingual-r2`, **D123**). |
| D101 | Provedor default = **`http`** apontando para um servidor local (`llama-server`); sem inferência in-process. |
| D80 | Embedding **assíncrono/lazy**: retrieval e write nunca bloqueiam; gap vetorial tolerado. |
| D81 | `recall` ganha **fusão RRF determinística** + degradação graciosa. |
| D82 | Orçamento do `rewind` explicitado **sem tokenizer** (D40 refinado). |
| D83/D84/D85 | Derivados confiáveis: cache por hash, `pending`/reconcile, purga em remoção, flush coalescido. |
| D86/D87 | **Âncoras por hash + verify-on-hit** e **confiança derivada** (não armazenada). |
| D88 | `rewind` emite **`context_id`** para handoff 1:1 (`--resume`). |
| D89/D90 | **`lightweight`** offline e **`kd maintenance eval --ab`** com métricas de retrieval. |
| D91 | **Nome lógico de projeto** + segredos só no global. |
| D92 | **Disciplina Rust** (tamanho, sem panic, proptest). |
| D14 | Removida qualquer noção de alias/retrocompatibilidade. |
| D69 | Instalação agrupada em **`kd self`**. |
| D93 | `task` ganha `scope` fechado (`plan\|epic\|issue\|task`) e hierarquia máx. 4. |
| D94 | `strict` vira config de projeto (`[behavior] strict`), sem flag. |
| D95 | Fixa o hash curto (SHA-256→u32), a chave do `id` e a gramática TOON v1 (`TOON.md`). |
| D96 | Fixa o registro de evento (`id` derivado + dedup on-read), rotação por tamanho e a semântica de `revision`. |
| D97 | Fixa o subset TOML (ordem preservada/canônica), o guard `git -C` do `sync` e a degradação do `onboard` sem global. |
| D98 | Fixa as 8 chaves de aresta, a ordem canônica de 27 chaves, o ponteiro reverso `superseded_by` e o storage derivado de sugestões. |
| D99 | Catálogo de validators em **TOML** (`validators.toml`), com `globals` no topo e resolução de `checks` em três fontes. |
| D100 | `not_before` como 28ª chave canônica; agendamento ortogonal à expiração; views dinâmicas o consideram, o `prime` estático não. |
| D101 | Embeddings via **HTTP local** (OpenAI-compatible); `provider = http\|lightweight\|none`; sem inferência in-process. |
| D102 | Canal vetorial no `ask` via `recall.semantic`/`recall.semantic_top_k`; `rank_query`; canal **filtrado** pelos filtros determinísticos; degradação graciosa (R33). |
| D103 | `outcomes[]` vale para **qualquer** nota; `kd write --outcome <status> <ID>`; a confirmação continua **derivada** (D87). Evento `op=outcome`. |
| D104 | Views `ready`/`blocked` como **modos** de `task list` (`--ready`/`--blocked`/`--explain`); motivo de bloqueio derivado (`blocked_by`/`not_before`/`cycle`). |
| D105 | Plano preenchível por LLM: `kd task plan --prompt` (derivado) e `--submit --from` (TOON); templates em `.knudge/templates.toml` (D97) sobrepõem built-ins; submissão **atômica**. |
| D106 | `rewind` emite `next:` (tarefas `ready` abertas por impacto) e `fresh:` (`stale`/`expiring`/`pending`); `prime` permanece estático (D57). `impact` = tarefas abertas com `depends_on` reverso. |
| D107 | `kd ask --tags` lista o vocabulário de tags (`tag\|count`, `count` desc, `tag` asc), ignorando `forgotten`/`superseded`. `kd ask --rank` ranqueia por confiança derivada sem query (`id\|statement\|confidence\|why`), no universo conhecimento (`scope` ausente), ordem `(confidence desc, id asc)`. |
| D108 | Feedback derivado **tarefa→conhecimento**: tarefas com `outcomes` de sucesso que compartilham `anchors` confirmam a nota (`task_confirmation`), alimentando o boost do BM25 e a confiança derivada — sem `write`. Peso em `recall.confirmation_from_tasks` (float, default 0.1); o manifest promove a `star`. |
| D109 | Impacto de desbloqueio **derivado** como modo `kd task list --sort impact` (ordem `impacto desc, created asc, id asc`); `--explain` acrescenta `unblocks=N`; sem chave de prioridade. O modo filtra `closed`/`superseded`/`forgotten`. |
| D110 | `kd write --batch -` aplica um lote de rascunhos **JSONL** pelo mesmo dedup (0.75/0.92); item inválido ⇒ `warnings[]` (R33); `--dry-run` só avalia; teto `write.batch_max` (int, 100). |
| D111 | `learn` ganha o sinal **tarefa→conhecimento**: tarefa com `outcomes` de sucesso cuja âncora não tem nota vira proposta `create_note` (`why="tarefa fechada sem nota"`); read-only (D47). |
| D112 | `kd maintenance prune` **propõe** `forget\|id\|motivo` (shelf-life/decay), excluindo membros de ciclo (D45) e sem gravar (D47); aplicação só via `kd forget`. |
| D113 | `scope` = nível, `type` = espécie; `kd task new --kind`; `scope` exigido só para `task`/`container`, opcional nos demais itens de trabalho. |
| D114 | Dono **derivado de eventos** `claim`/`release`; `kd task claim`; `--owner`/`--mine` (`KNUDGE_AGENT`). Sem chave canônica. |
| D115 | Papel (Initiative/Epic/Feature/Story/Sub-task/Bug/Spike/Risk/Decision) **derivado** de `(scope, type, tem_filhos)`; nunca armazenado. |
| D116 | Modo de execução (sequential/concurrent/supervisor/handoff/magentic) **derivado** do grafo/eventos; `kd task graph` projeta `role\|kind\|status\|owner\|mode`. |
| D119 | **Programa = arquivo externo `plan/*.md` ancorado ao Épico-raiz** (`scope=epic`, sem pai); `scope=plan` deprecado; `kd task graph --program`; check `program-anchor` no `doctor`; config `programs.glob`; `--source` em `task new`. |
| D120 | **Item de trabalho = espécie de trabalho com `scope`** (`NoteType::is_work_kind` + `Graph::is_work_item`); as views `ready`/`blocked`, o `impact` e o `next:` passam a enxergar `--kind error\|question\|risk\|decision`. Containers e conhecimento (sem `scope`) ficam de fora. |
| D121 | `why` do `ask` ganha **`semantic`** (canal vetorial), com precedência `file_match > anchor_match > tracker_match > stars > semantic > recent > universal`; o default de `recall.default_limit` cai de 10 para **5**. |
| D122 | O canal lexical descarta **stopwords PT+EN** e **fragmentos de 1 caractere** (`retrieval::token::content_terms`); corrige votos espúrios (ex.: `de`) que afogavam o canal vetorial em consultas por sinônimo. |
| D123 | **Modelo de embedding default passa a `ibm-granite/granite-embedding-97m-multilingual-r2`** (384d, Apache-2.0, 200+ idiomas com **PT** explícito), substituindo `msmarco-MiniLM-L12-cos-v5` (inglês). Medido na bancada PT-BR (`bench/`): +0.070 nDCG@5 sobre BM25 e melhor R@1/MRR que o default antigo, **mesmo tamanho de índice**. Execução/provedor inalterados (D101); o modelo antigo segue servível por config. |
| D124 | **Fusão RRF passa a ter peso por canal** (`recall.lexical_weight`/`anchor_weight`/`semantic_weight`; revê D81). Default `semantic_weight=30` **Pareto-domina** o neutro no corpus PT-BR (R@1/R@5/MRR/nDCG@5 ≥ 1:1) e `≥80` converge para o ranking vetorial puro; corrige a diluição do canal vetorial por votos lexicais. |
| D125 | **`kd task show` resolve o contexto estrutural** do item — `parent`, `blocked_by`, `blocks` e `children` com **título** e estado (`task::context_of`) — em texto e `--json`. Um comando responde "onde isto se encaixa e o que o bloqueia" sem puxar a árvore inteira; era a lacuna apontada pela bancada de hierarquia. |
| D126 | **Arestas têm via única:** `kd write --link <FROM:ARESTA:TO>`. `kd task new` deixa de aceitar `--depends-on` e `TaskSpec.depends_on` sai do core; `plan submit` cria as dependências dos passos via `write::link`. Reduz a superfície de API e reaproveita o caminho de grafo (valida id/auto-aresta, grava evento e revisão). |
| D127 | **Rollup de progresso por épico é derivado** (`task::epic_of`/`progress_of`): conta os **itens de trabalho folha** (`Graph::is_work_item` sem filhos de trabalho) no subárvore do épico e quantos estão `closed`; sem verdade nova. Folhas = a fronteira acionável (`task` + `issue` não decomposta): fechar um `issue` com tarefas abertas não infla, esquecer de fechá-lo não trava. Exposto no `kd task close` (linha `epico: <id>\|<título> (<done>/<total>)`), no `kd task show` (`epico:`/`progresso:`) e nos containers do `task graph` (`(done/total)`). Fecha a lacuna "nenhum faz rollup automático" da bancada de hierarquia. |
| D128 | **Clusters ganham verbo próprio `kd knowledge` e o eixo `container` passa a usar a hierarquia.** `container_of` sobe pelos pais (`results_in`, D52/D93) — a mesma relação de `Graph::parent`/`belongs_to` — com fallback para `depends_on`; o eixo Container deixa de ser vazio em projetos reais (era `0/53`, `0/125`). `kd knowledge map [--axis A] [--scope C] [--semantic] [--members]` expõe a fase 1 e, com `--semantic`, a fase 2 dentro de cada cluster acima de `clusters.min_volume` — config que antes era ignorada. Read-only (D47). Sobe de `kd maintenance` para um verbo próprio porque o mapa de conhecimento é consulta primária, não manutenção. |
| D129 | **Fase 2 usa complete-link** (`cluster_by_similarity`): um id só entra num cluster se for similar (`>= clusters.similarity_threshold`) a **todos** os membros. Corrige o efeito do leader/single-link em que um item central puxa vizinhos dissimilares entre si (com threshold 0.5, 31 itens viravam um só grupo). |
| D130 | **Verbos falham alto, nunca em silêncio.** `kd ask` sem modo devolve o uso (exit 2) em vez de sair vazio; `kd write`/`kd task new` sem `statement` é `invalid_input` (2) — antes criavam nota/tarefa com `statement: ""`; nota ausente é `not_found` (3), ativando a degradação que o `get` já previa (era `io`=5). Nenhuma entrada vazia grava lixo nem retorna vazio sem explicação. |
| D131 | **Auto-drain ocioso no `lazy`; sem `eager`.** `embeddings.mode` passa a aceitar só `lazy` (default) e `manual` — `eager` é rejeitado (`config`=7). Em `lazy`, ao fim de cada invocação não-`maintenance`, o CLI drena **um lote** (`embeddings.batch`) *best-effort*, **depois** de emitir a saída; nunca altera exit code nem `warnings[]` (não interage com `strict`). O `--drain` explícito continua valendo nos dois modos (coexistem). `KNUDGE_NO_IDLE` desliga o caminho ocioso. O worker contínuo (timer/systemd) fica fora do binário, via `scripts/knudge-idle.sh`. |
| D132 | **`kd maintenance watch-service` gerencia o worker ocioso — com consentimento e sem supply-chain.** Ações exclusivas (default `--status`): `--install` (pré-flight de `systemd --user`/`kd`/`llama`/GGUF/projeto → escreve `idle.conf` + unidades, habilita o timer e cadastra o projeto atual), `--subscribe`/`--unsubscribe` (cadastram/descadastram **um** projeto, multi-projeto; **não** desinstalam o sistema), `--status` (saúde: timer, servidor, fila por projeto) e `--uninstall` (remove o sistema). Ações que mutam perguntam no stderr (`s/N`; stdin não-TTY cancela; `--yes` pula). O worker (`knudge-idle.sh`) é **embutido no binário** (`include_str!`) e materializado no staging de cache — **sem download por padrão**; `--script` (local) e `--url`/`KNUDGE_SCRIPT_URL` (HTTPS via `curl`) sobrescrevem. O GGUF mora ao lado do `config.toml` global (`${XDG_CONFIG_HOME:-~/.config}/local/knudge/`). Multi-projeto = lista `PROJECT=` no `idle.conf` + **um** timer; `run` sem args drena todos; ao esvaziar, o timer é parado, mas as unidades permanecem. |
| D133 | **`watch-service` é multiplataforma e mantém o servidor de embeddings persistente.** O agendador é detectado em runtime: `systemd --user` (Linux) ou `launchd` (macOS, `~/Library/LaunchAgents`); sem nenhum, o `--install` recusa e imprime a linha de cron. O servidor llama.cpp vira unidade/agente próprio (`knudge-embed.service` / `local.knudge.embed.plist`, `Restart=on-failure` / `KeepAlive`), então o `--drain` manual e o auto-drain lazy sempre o encontram; o worker só drena e sobe um efêmero de fallback se o persistente estiver fora. `--uninstall` derruba agendador + servidor. |

## Pendências / pontos de atenção

| # | Questão | Encaminhamento sugerido |
|---|---|---|
| D05 × D14 | D05 diz "sem nota parcial"; D14 diz "sem retrocompatibilidade". Combinadas: **write estrito, sem draft**, e nenhuma migração de corpus antigo. | Confirmar que não há corpus a preservar. |
| D16 × D17 | Chave desconhecida **tolera**, tipo desconhecido **rejeita**. | Garantir que a rejeição (D17) seja **por nota** e que o `doctor` a reporte com orientação — senão uma nota futura derruba a leitura. |
| D30 × D34 | D30 pede exclusão **absoluta** de todo o `.knudge/`; D34 mantém `persist_in_project=true` versionando. | Interpretação adotada: **`persist=true`** versiona as notas mas exclui derivados (`.idx/`, `cache/`, `*.lock`) via `info/exclude`; **`persist=false`** exclui o `.knudge/` inteiro. Confirmar. |
| D01 × D02 | ID endereçado por conteúdo + prefixo histórico. | **Resolvido por D95**: chave = `type + U+001F + normalize(statement)`; `id` é histórico. |
| D17 | Sem retrocompatibilidade + rejeitar tipo desconhecido. | Toda evolução de schema exige **rebuild em massa** — documentar o procedimento. |
