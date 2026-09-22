# Gaps e melhorias — o que a investigação profunda de `mulch-rs` e `seeds-rs` revela

> Leitura de `refs/mulch-rs/plan-rs/` e `refs/seeds-rs/plan-rs/` (EPIC, ARCHITECTURE, STORAGE, RECORDS, CLI, COMMANDS, COMPATIBILITY, DIVERGENCES, TESTING, WBS).
> Objetivo: extrair o que só aparece no **nível de implementação** — depois que as ferramentas foram dissecadas byte a byte — e confrontar com o panorama do knudge (`00_panorama.md`).
>
> **TL;DR:** o knudge acertou a *tese* (estrutura antes de estatística, índice derivado, protocolo de escrita). Os `plan-rs` revelam que falta a **camada de contrato de implementação**: normalização de bytes, aliases de schema, lock mesmo com 2 agentes, resolução de worktree, decay de âncoras, orçamento de tokens no `prime`, tolerância na leitura, e uma disciplina de documentação (DIVERGENCES / acceptance matrix) que o knudge ainda não tem.

---

## 1. O que os `plan-rs` validam

O knudge já está alinhado com as decisões estruturais mais caras dessas ferramentas:

| Decisão do knudge | Confirmação no `plan-rs` |
|---|---|
| Índice é derivado, reconstruível | `mulch`/`seeds` tratam `.mulch/`/`.seeds/` como verdade e nunca reconstroem do índice |
| Núcleo puro + adaptador externo | Ambos migram para **core puro + CLI/FFI/WASM** com ports (`Clock`, `Rng`, `Git`, `Fs`) |
| Tipos fechados | O registry do mulch tem 6 built-ins + custom types, mas com **20+ regras de validação** — fechar o enum foi a escolha certa para o knudge |
| Domínios como partição física rejeitados | O mulch precisa de `move` justamente porque a partição rígida cria atrito; namespace flat + índice é superior |
| `compact`/`prune` são batch, off-path | `RS-M7` trata ciclo de vida como épico separado; nada no caminho crítico |
| Embeddings fora do núcleo | Nenhuma das duas ferramentas usa embeddings; retrieval é BM25 + estrutura |
| Escrita atômica (`tmp` + `rename`) | `STORAGE.md` §3.2 e `COMPATIBILITY.md` §2.5 — contrato Nível A |
| Dedup é do escritor | `findDuplicate` roda no `write`, não no rebuild |

---

## 2. Descobertas que expõem gaps no knudge

### 2.1 Contrato de bytes (serialização) — o maior buraco

Os `plan-rs` gastam centenas de linhas em casos que o knudge nem menciona. Como o knudge também é *byte-sensitive* (frontmatter TOON, hashes, índice derivado), cada um desses é um gap real:

| Descoberta (`DIVERGENCES`) | Gap no knudge | Proposta |
|---|---|---|
| Ordem de chaves é contrato; `IndexMap`, nunca reconstruir de struct (`mulch` DIV-001) | O knudge não fixa a **ordem canônica** das chaves do frontmatter | Fixar ordem no schema (ex.: `id, type, statement, created_at, …`) e testar round-trip |
| Opcional ausente ≠ `null` (DIV-002) | Não especificado | `skip_serializing_if` equivalente: chave ausente, nunca vazia |
| Lista vazia → arquivo **zero bytes** (DIV-003) | Não especificado | Definir para `events.jsonl`, listas de container, índice |
| Append: mulch **não** normaliza newline; seeds **normaliza** (DIV-004 / seeds DIV-022) | Não especificado — e as duas ferramentas divergem entre si | Escolher e congelar uma semântica; documentar como contrato |
| `1` vs `1.0` (DIV-006 / seeds DIV-003) | TOON pode ter o mesmo problema | Normalizar inteiros na decodificação |
| `outcome` singular → `outcomes[]` no read (DIV-005) | Sem mecanismo de **normalização legada** | Ver §2.2 (aliases) |
| `body_hash`: qual normalização? | O knudge diz "hash do corpo normalizado — lowercase, whitespace colapsado" mas não define Unicode (NFC?), pontuação, ordem | Definir `normalize(body)` por escrito e congelar em teste; considerar hashear `statement + body` |

### 2.2 Aliases de campo e migração de schema

O `mulch` tem `applyAliases`: campo legado → canônico no read, canônico vence, legado é removido. É exatamente o mecanismo que o knudge precisa **agora**, depois de renomear `k→type`, `t→statement`, etc.

**Gap:** o panorama diz "migração on-read (nota antiga é lida com defaults)", mas não há mecanismo para **chave renomeada**. Uma nota antiga com `k:` seria rejeitada ou lida como tipo ausente.

**Proposta:** tabela de aliases versionada (`k→type`, `t→statement`, `c→confidence`, `cls→classification`, …), aplicada no read; `doctor --fix` reescreve na forma canônica. É a única forma de renomear schema sem quebrar o corpus existente.

### 2.3 IDs: aleatório vs endereçado por conteúdo

O `mulch` gera `mx-<sha256(type:idKey)[0..6]>` — **determinístico e endereçado por conteúdo**. O knudge gera `<prefixo>_base36(8)` aleatório.

| | aleatório (knudge) | determinístico (mulch) |
|---|---|---|
| Colisão | precisa re-rolar | zero (hash de conteúdo) |
| Idempotência | `write` duplicado cria duas notas → depende do dedup | mesmo conteúdo → mesmo id → write idempotente |
| Retry do LLM | pode duplicar | seguro |
| Renomear campo-chave | id estável | id muda |

**Gap/proposta:** decidir explicitamente. Para um LLM que **repete e re-tenta**, ID endereçado por conteúdo (`type + statement`) elimina a classe inteira de duplicatas antes mesmo do dedup semântico. Híbrido possível: id derivado do hash, com `superseded_by` preservando a linhagem quando a chave muda.

### 2.4 Leitura tolerante, escrita estrita

Os `plan-rs` separam rigorosamente:
- **Write**: valida schema, rejeita chave desconhecida.
- **Read**: `--allow-unknown-types`, **skip-on-read** de linhas JSONL malformadas, dedup-on-read (última vence, primeira posição).

**Gap no knudge:** só há "rejeita no write". Se uma nota tiver chave desconhecida ou o índice encontrar uma linha corrompida, o `recall` inteiro pode quebrar — uma nota ruim envenena a base. 

**Proposta (Postel):** `read` tolera tipo desconhecido e JSONL malformado (com warning + skip); `doctor` reporta; `write` continua estrito. `--strict` opcional para CI.

### 2.5 Dedup: on-write vs on-read

- `mulch`: dedup **on-write** (`findDuplicate`), sem dedup on-read.
- `seeds`: dedup **on-read** (`Map`, última vence, primeira posição preservada) porque `merge=union` pode produzir duplicatas.

**Gap no knudge:** o knudge só tem dedup on-write (com limiares 0.75/0.92). Mas `events.jsonl` + merge de git podem duplicar entradas. 

**Proposta:** dedup on-write para `notas/` (como já é) **e** dedup on-read para `events.jsonl` e para listas de container (idempotência sob merge).

### 2.6 Locking — o knudge cortou cedo demais

O knudge assumiu 2 agentes orquestrados e **removeu todo lock**. Os `plan-rs` mostram que o problema não é só concorrência entre agentes:

- **CLI + MCP rodando ao mesmo tempo** (o MCP proativo do knudge monitora enquanto o agente trabalha — logo há dois processos).
- **Índice reconstruído enquanto o `recall` lê.**
- **Worktrees.**
- **Reclaim de lock stale sem apagar lock alheio** (`seeds` DIV-040: rename sidecar + comparação de inode/mtime — nunca `unlink` direto).
- **Ordem de aquisição de locks** para evitar ABBA (`seeds` DIV-042: externo=plans, interno=issues).

**Gap/proposta:** manter a disciplina de escritor único, mas com um **lock advisory** (`O_CREAT|O_EXCL`, stale 30s, retry com jitter) em torno de `write`/`update`/`rebuild`, e uma **ordem de lock documentada** para escritas multi-arquivo (nota → eventos → container). Custo baixo, remove uma classe de bugs que só aparece em produção.

### 2.7 Worktrees e Git

O `mulch` resolve `.mulch/` no **worktree principal** via `git rev-parse --git-common-dir` (exclui submódulos). O knudge cria `.knudge/` na raiz do projeto — em um git worktree, cada worktree teria sua própria `.knudge/` (conhecimento fragmentado) ou nenhuma.

**Gap/proposta:** definir a resolução do diretório de conhecimento (worktree principal), e o comportamento do `.git/info/exclude` (que é **por worktree**, não por clone — verificar). Isso interage diretamente com a config `persist_in_project`.

### 2.8 `.gitattributes merge=union`

`mulch` e `seeds` escrevem `merge=union` para os JSONL. O knudge usa um arquivo por nota (minimiza conflito), **mas** `events.jsonl` e listas de container são arquivos únicos que vão conflitar.

**Gap/proposta:** decidir o que é union-merged (candidatos: `events.jsonl`), o que é gitignored (`.idx/`, `cache/`, `*.lock`), e documentar. Idealmente containers não são arquivos com lista materializada (ver §2.10).

### 2.9 Retrieval: TF-IDF vs BM25 + boost + orçamento

Os `plan-rs` mostram o motor real:
- **BM25** com `k1=1.5, b=0.75`, `IDF = ln((N-df+0.5)/(df+0.5)+1)`, `matchedFields`, tokenização **ASCII** (`\w` do JS ≠ Unicode do Rust — DIV-020/DIV-045).
- **Boost por confirmação**: `score * (1 + 0.1 * (success + partial*0.5))`.
- **Trust-tier ranking** para `prime`: `star*100 + foundational*50 + tactical*20 + observational*10`.
- **Orçamento de tokens**: `estimateTokens = ceil(len/4)`, budget default 4000, prioridade por tipo → classificação → score → timestamp.
- **`why surfaced`**: `file_match`, `tracker_match`, `stars`, `recent`, `universal`.
- **Auto-context-scope**: `git status -uall` + active-work; **auto-flip** para manifest se `>100 records` ou `>5 domains`.

**Gaps no knudge:**
1. O panorama diz "TF-IDF". Para notas curtas e técnicas, **BM25 é estritamente melhor** e já é o que o mulch usa. Trocar.
2. Falta **boost por confirmação** (o knudge tem `confirmations`, mas não usa no score).
3. Falta **orçamento de tokens** — contraditório num sistema cujo custo é contexto.
4. Falta **`why surfaced`** no retorno do `recall` (ajuda o LLM a decidir o que puxar).
5. Falta **auto-context-scope** e **auto-flip** no `prime` (o knudge só tem `prime(files)` explícito).
6. Falta a decisão sobre **unidade de contagem de tokens**.

**Proposta:** substituir TF-IDF por BM25; adicionar boost de confirmação; incluir uma 3ª coluna `why` no `recall`; dar `--budget` ao `prime`; implementar auto-scope por `git status`.

### 2.10 Grafo: supersessão com ciclos e integridade referencial

O `mulch` usa **Tarjan iterativo** para SCCs: membros de ciclo **não** são demovidos; arestas fora de ciclo geram `supersededIds`; cross-domain. O `seeds` valida **referential-integrity**, **bidirectional-consistency** e **circular-dependencies** no `doctor`.

**Gap no knudge:** cadeias de `sup`/`superseded_by` podem **ciclar** (A substitui B substitui A), e arestas `dep`/`rej`/`res` podem ficar **penduradas** (alvo apagado). Nada disso está tratado.

**Proposta:** detecção de ciclo no rebuild (SCC ou DFS com marcação), bidirecionalidade `sup ↔ superseded_by`, e `doctor` com `referential-integrity` + `circular-dependencies`.

### 2.11 Decay de âncoras e staleness

O `mulch` tem `computeAnchorValidity` (fração de `files`/`dir_anchors`/`evidence.file` que ainda existem), grace period, threshold, e shelf life por classificação (`foundational` nunca expira; `tactical` 14d; `observational` 30d).

**Gap no knudge:** `anchors` são estáticos. Quando um arquivo é renomeado/apagado, a nota fica órfã e o `prime(files)` nunca mais a encontra. Sem staleness, o corpus apodrece em silêncio.

**Proposta:** no rebuild, validar âncoras (existe? glob ainda casa?); no `audit`/`prune`, demover notas com âncoras inválidas após grace period. Adicionar `observational` à classificação (shelf life curta) — o knudge tem só 2 valores.

### 2.12 `doctor --fix` — auto-reparo

O `mulch` tem 17 checks com `--fix` (remove linhas inválidas, migra `outcome` legado, poda stale, remove âncoras quebradas, cria arquivos/domínios faltantes). O `seeds` tem 13 checks.

**Gap no knudge:** só `audit()` (leitura). Um knowledge base sem auto-reparo **degenera em silêncio** — duplicatas, órfãos, tipos desconhecidos, locks stale.

**Proposta:** `doctor [--fix]` com checks: schema, JSONL/TOON integrity, aliases legados, referential-integrity, ciclos, âncoras quebradas, duplicatas, locks stale, config válida, `body_hash` desatualizado, `events.jsonl` malformado.

### 2.13 Prime: contrato + hooks + sessão

O `plan-rs` detalha o que o `prime` do mulch contém, e o knudge absorveu a *ideia* mas não o *conteúdo*:
- **Contract block** (protocolo inline).
- **Footer session-close** (checklist), com estilo `conditional | custom`.
- **`--export <path>`**, **`--dry-run` JSON**.
- **Anotações "why surfaced"**.
- **Budget** e **formatos** (markdown/compact/xml/plain).
- **Hooks** de ciclo de vida (`pre-record`, `post-record`, `pre-prime`, `pre-prune`, `pre-compact`) com stdin JSON, mutação de payload, bloqueio, **process-group kill** e **redaction de segredos**.

**Gaps no knudge:** `prime` sem budget/why/export/dry-run; ausência total de **hooks** (o `validators.yaml` cobre checks de task, mas não hooks gerais). O protocolo de fechamento está no papel, não num mecanismo que o sistema emite.

**Proposta:** formalizar o bloco de fechamento como saída do `prime` (não só do `onboard`); adicionar hooks opcionais com timeout e kill de process-group; `prime --dry-run`/`--export`.

### 2.14 Evidência e outcomes ricos

O `mulch` tem `evidence` (`commit, date, issue, file, bead, seeds, gh, linear`) e `outcomes[]` (status, duration, test_results, agent, notes, recorded_at) com score derivado. O knudge tem `source` (string/url), `outcome` (escalar) e `confirmations` (int).

**Gaps/proposta:**
- Trocar `source` por `evidence` (mapa) — proveniência de engenharia (commit, arquivo, issue) é o que permite reconstruir decisões.
- Trocar `outcome` escalar + `confirmations` por `outcomes[]` — histórico temporal vale mais que contador; a confirmação é **derivada** (`success + partial*0.5`), não armazenada.

### 2.15 Arestas explícitas vs extraídas

O knudge decidiu que **o LLM não declara arestas**; elas emergem do corpo por regex. `mulch` e `seeds` fazem o oposto: campos explícitos `relates_to`, `supersedes`, `blocks`/`blockedBy`.

**Gap/tensão:** extração por regex é frágil (falso positivo polui o `expand` para sempre — o próprio knudge reconhece). Links explícitos são mais confiáveis e são o que o `doctor` pode validar.

**Proposta:** inverter a prioridade — **campo explícito `links`** (declarado pelo LLM ou por `link()`) como fonte primária, e extração automática como **sugestão** (revisável), nunca como aresta definitiva. O `expand` confia no explícito; o `audit` sugere arestas faltantes.

### 2.16 Plano/container: semântica sub-especificada

O motor de planos do `seeds` é o núcleo mais complexo (2.549 LOC): JSON Schema gerado, backrefs `<!-- seeds:plan-backref:start/end -->`, máquina de estados (`computeNextPlanStatus`), `submit/overwrite/adopt/reorder/release/edit/outcome/review`, `max_plan_depth`, `requires_plan`, `plan_step_index`, `adoptedChildren`, `blocks` 1-based, self-reference.

**Gap no knudge:** `plan(task_id, steps)` e `container` são uma linha cada. Falta: backref no corpo da task, profundidade máxima, adoção de tasks existentes, reordenação, estado derivado, integridade bidirecional pai↔filho.

**Proposta:** especificar o ciclo de vida do container/plan antes de implementar; adotar backref delimitado por marcador; profundidade máxima configurável; validação de `blocks` (range, self-reference).

### 2.17 Arquitetura: core/adapter + ports

Ambos os `plan-rs` convergem para **workspace Cargo com núcleo puro** (sem terminal, `argv`, `exit`, clock/RNG global) e **adaptadores finos**, com traits `Clock`, `Rng`, `Git`, `Fs`, `HookRunner`, `Logger`, `Env`.

**Gap no knudge:** o panorama diz "um binário único (`kb`)" e "o utilitário é a única peça externalizada", mas não separa núcleo de adaptador. Isso trava testes determinísticos e embedding (MCP/agentes).

**Proposta:** `knudge-core` (puro, com ports) + `knudge-cli` + `knudge-mcp` (+ FFI/WASM futuros). O MCP proativo do knudge vira naturalmente um adaptador sobre o core.

### 2.18 Distribuição e adoção

`mulch`/`seeds` têm `onboard` com **marcadores HTML + version marker**, `setup` (recipes `claude`/`cursor`/`codex`), `completions`, `upgrade`, `migrate-from-beads`.

**Gaps no knudge:** `onboard` sem formato de marcador definido; sem `setup` de provider; sem completions; sem `upgrade`; **sem caminho de migração** (o knudge quer substituir seeds/mulch, mas não há `migrate-from-seeds`/`migrate-from-mulch`).

**Proposta:** adotar marcadores idempotentes (`<!-- knudge:start -->` … `<!-- knudge:end -->` + `<!-- knudge-onboard:v1 -->`); adicionar `setup`/`completions`/`upgrade`; especificar migração.

### 2.19 Formato de saída como contrato de máquina

O knudge define o retorno do `recall` em pipe (`id|statement|score`) — ótimo para tokens. Mas `mulch`/`seeds` expõem **envelope JSON** (`{success, command, error}`) consumido por outros programas (warren, agentes). O MCP proativo e o embedding precisam de saída estruturada.

**Gap:** o knudge só tem formato para LLM; falta o **envelope JSON** para máquinas (MCP, scripts), com exit codes e mensagens de erro congeladas por teste.

**Proposta:** pipe para LLM, `--json` para máquinas; catálogo de mensagens congelado (o LLM lê erros, logo a mensagem é contrato).

### 2.20 Disciplina de documentação e testes

Os `plan-rs` são, em si, um achado: **EPIC + WBS + ARCHITECTURE + COMPATIBILITY + DIVERGENCES + TESTING + PARITY-CHECKLIST**. O `DIVERGENCES.md` cataloga ~60/57 riscos de borda com mitigação e teste; o `TESTING.md` define harness diferencial, fuzz e crash-injection.

**Gap no knudge:** um panorama + brainstorm, sem catálogo de casos de borda nem matriz de aceite. Como o knudge não tem implementação de referência, o análogo do harness diferencial é **golden/snapshot + property tests + stress de concorrência**.

**Proposta:** criar `DIVERGENCES.md` do knudge (Unicode, timestamps, hashing, ordem, lock, atomicidade, TOON) e uma **matriz de aceite por tool** (formato pipe, JSON, erro, exit, estado do `.knudge/`).

---

## 3. Priorização

### Top 10 (maior valor / menor custo)

1. **Aliases de schema** (§2.2) — desbloqueia a renomeação recém-feita sem perder corpus.
2. **Contrato de bytes** (§2.1) — ordem de chaves, `null` vs ausente, zero-byte, append, hash.
3. **Leitura tolerante + `doctor --fix`** (§2.4, §2.12) — evita que uma nota ruim envenene a base.
4. **Lock advisory + ordem de lock** (§2.6) — CLI + MCP são dois processos.
5. **BM25 + boost de confirmação + orçamento de tokens** (§2.9) — núcleo do valor do sistema.
6. **Decay de âncoras + staleness** (§2.11) — sem isso o corpus apodrece.
7. **Integridade do grafo** (ciclos, bidirecionalidade, dangling) (§2.10).
8. **Core/adapter + ports** (§2.17) — habilita determinismo e MCP.
9. **Evidência + outcomes[]** (§2.14) — proveniência de engenharia.
10. **Arestas explícitas como primárias** (§2.15) — corrige a fragilidade da extração por regex.

### Depois

- Worktrees + `.gitattributes` (§2.7, §2.8).
- `prime` com contract block/why/export/dry-run + hooks (§2.13).
- Semântica de plan/container (§2.16).
- `onboard` com marcadores, `setup`, `completions`, `upgrade`, migração (§2.18).
- Envelope JSON + catálogo de erros congelado (§2.19).
- `DIVERGENCES.md` + matriz de aceite do knudge (§2.20).
- ID endereçado por conteúdo (§2.3) — decisão de design, avaliar com o dedup.

---

## 4. Decisões em aberto (não resolvidas pelos `plan-rs`)

Os `plan-rs` são sobre **paridade com um produto existente**; o knudge é **spec-first**. Isso significa que algumas perguntas deles não têm resposta pronta:

1. **ID aleatório vs endereçado por conteúdo** (§2.3) — trade-off real, sem análogo direto.
2. **Arestas explícitas vs extraídas** (§2.15) — os `plan-rs` usam explícitas, mas o knudge quer que o LLM escreva pouco. Talvez: extração sugere, LLM confirma no `write`.
3. **Unidade de contagem** (`statement ≤ 120`): escalares Unicode, UTF-16 (AJV) ou grafemas? O `seeds` DIV-009 mostra que isso muda a validação.
4. **`persist_in_project=false` + worktree**: onde vive o `.knudge/` e o que exatamente o `.git/info/exclude` cobre (por worktree, não por clone).
5. **Formato TOON**: nenhum dos `plan-rs` usa TOON. Não há biblioteca de referência; o parser/emitter do knudge será inédito e precisa de corpus/golden próprio.

---

## 5. A observação que fecha

O brainstorm do knudge foi forte em **semântica** (o que é uma nota, quando criar, como recuperar) e os `plan-rs` são fortes em **contrato de implementação** (o que cada byte significa, como cada borda falha). São complementares.

O que a investigação profunda ensina, no fim, é que a maior parte do custo real de um sistema assim **não está no modelo, está nas bordas**: Unicode, newline, ordem de chaves, locks, worktrees, âncoras que apontam para arquivos que não existem mais. O knudge já decidiu o modelo; os `plan-rs` dizem quais 40 casos de borda ele ainda precisa decidir antes de escrever a primeira linha de código.
