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
| **D57** ✅ | Protocolo vive no **`onboard`** (uma vez, `AGENTS.md`); `prime` só estado. **Caveat: `prime` é o comando que situa o estado entre agentes/rodadas.** |
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
| **D79** ✅ | **Provedor de embedding plugável via `config.toml`** (`local`/`http`/`none`); modelo default **`sentence-transformers/msmarco-MiniLM-L12-cos-v5`** (384d, cosseno nativo), com **`msmarco-MiniLM-L6-cos-v5` como perfil rápido** (~2× mais rápido, mesmo índice). `revision` pinada e re-embed quando o modelo muda. Avaliar multilíngue se o corpus for PT-BR. Ver `04_embeddings.md`. |
| **D80** ✅ | **Embedding assíncrono e lazy; nunca bloqueia.** `write`/`recall`/rebuild seguem sem esperar o modelo; notas recém-criadas ficam **“dark”** no espaço vetorial até serem digeridas por uma fila (gap tolerado em rajadas de 10–20). Dedup no write é **lexical**; o semântico é **eventual** (reconciliação). Estado `embedded\|pending\|stale` é derivado, em `.idx/`; `prime` reporta `embeddings_pending`. |
| **D69** ✅ | Binário estático + `completions` + `setup` + `upgrade`. |
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
| **D88** ✅ | **`prime` emite `context_id` endereçável**; `kd get-context <id>` devolve o **mesmo contexto 1:1**, sem re-busca — handoff reprodutível entre agentes/rodadas. |
| **D89** ✅ | **`provider = "lightweight"`** (embedder determinístico por hash) para testes/CI/offline — sem download de modelo, sem rede. |
| **D90** ✅ | **`kd eval --ab`** com Recall@k / nDCG@k / MRR sobre um golden pequeno — o jeito de decidir L6 vs L12 vs multilíngue sem chutar. |
| **D91** ✅ | **Projeto por nome lógico** (worktrees do mesmo repo compartilham `.knudge/`); **segredos só no global** — o projeto nunca carrega credenciais. |
| **D92** ✅ | **Disciplina Rust**: arquivos ≤300 linhas de produção, proibido `unwrap/expect/panic` em `src`, **proptest** nos puros (RRF, decay, confiança), clippy `-D warnings`. |

---

## Impactos no panorama (já propagados)

| Decisão | Onde mudou |
|---|---|
| D66/D67 | Binário `kb` → **`kd`**; stack **Rust**. |
| D35/D36/D37/D38/D39/D40/D41 | §Retrieval: TF-IDF → **BM25**, tokenização ASCII, IDF por campo, boost, coluna `why`, budget, auto-scope. |
| D44 | `classification` ganha **`observational`**. |
| D48 | `outcome` + `confirmations` → **`outcomes[]`** com confirmação derivada. |
| D49/D51 | Arestas passam a **explícitas + declarativas** (`references`, `depends_on`, …). |
| D57 | `prime` explicitado como **handoff de estado entre agentes/rodadas**. |
| D65 | Arquitetura organizada por **escopo temático** (`core`, `cli`, `mcp`, `jsonl`, …). |
| D42/D79 | Embeddings passam a ser **provedor plugável** via config, com default `msmarco-MiniLM-L12-cos-v5`. |
| D80 | Embedding **assíncrono/lazy**: retrieval e write nunca bloqueiam; gap vetorial tolerado. |
| D81 | `recall` ganha **fusão RRF determinística** + degradação graciosa. |
| D82 | Orçamento do `prime` explicitado **sem tokenizer** (D40 refinado). |
| D83/D84/D85 | Derivados confiáveis: cache por hash, `pending`/reconcile, purga em remoção, flush coalescido. |
| D86/D87 | **Âncoras por hash + verify-on-hit** e **confiança derivada** (não armazenada). |
| D88 | `prime` emite **`context_id`** para handoff 1:1. |
| D89/D90 | **`lightweight`** offline e **`kd eval --ab`** com métricas de retrieval. |
| D91 | **Nome lógico de projeto** + segredos só no global. |
| D92 | **Disciplina Rust** (tamanho, sem panic, proptest). |
| D14 | Removida qualquer noção de alias/retrocompatibilidade. |

## Pendências / pontos de atenção

| # | Questão | Encaminhamento sugerido |
|---|---|---|
| D05 × D14 | D05 diz "sem nota parcial"; D14 diz "sem retrocompatibilidade". Combinadas: **write estrito, sem draft**, e nenhuma migração de corpus antigo. | Confirmar que não há corpus a preservar. |
| D16 × D17 | Chave desconhecida **tolera**, tipo desconhecido **rejeita**. | Garantir que a rejeição (D17) seja **por nota** e que o `doctor` a reporte com orientação — senão uma nota futura derruba a leitura. |
| D30 × D34 | D30 pede exclusão **absoluta** de todo o `.knudge/`; D34 mantém `persist_in_project=true` versionando. | Interpretação adotada: **`persist=true`** versiona as notas mas exclui derivados (`.idx/`, `cache/`, `*.lock`) via `info/exclude`; **`persist=false`** exclui o `.knudge/` inteiro. Confirmar. |
| D01 × D02 | ID endereçado por conteúdo + prefixo histórico. | Definir a chave exata do hash (inclui `type`?) e o que acontece quando só o `statement` muda (novo id + `superseded_by`). |
| D17 | Sem retrocompatibilidade + rejeitar tipo desconhecido. | Toda evolução de schema exige **rebuild em massa** — documentar o procedimento. |
