# Panorama geral — Sistema de memória otimizada para LLM

> Síntese do brainstorm em `plan/brainstorm/` (1.md a 7.md). Documento de orientação: descreve a visão, o que já está decidido, o que foi descartado e o que segue em aberto.

---

## 1. A tese central

Externalizar o **conhecimento** gerado em sessões de IA — não a conversa: decisões, contexto, resultados e ajustes não-óbvios — num sistema de arquivos otimizado para **query, correlação e consulta por LLM**.

A inversão que define tudo: a maioria dos sistemas de conhecimento otimiza **armazenamento** e depois tenta encaixar o LLM. Este faz o contrário — otimiza o **contexto injetado** e trata armazenamento/índice como detalhe derivado.

Três consequências que explicam todas as decisões:

1. **O usuário é o LLM.** Não há humano lendo ou editando. Legibilidade humana é irrelevante; tokenização é tudo.
2. **O custo é contexto, não disco.** Índice rápido não vale nada se a resposta injetada for verbosa.
3. **O índice é derivado.** As notas reconstroem o índice do zero. Se isso deixar de ser verdade, o design quebrou.

**Objetivo final (premissa global):** ser o **único** sistema de memória do workflow — um binário único (`kd`, em Rust) e uma estrutura única (`.knudge/`) por projeto, independente, substituindo ferramentas externas como `sd` (seeds) e `ml` (mulch).

---

## 2. Arquitetura

### Camadas (das mais estáveis às mais descartáveis)

```
.knudge/                     # por projeto, independente
  config.toml                # efetivo — clonado do global na instanciação; tem precedência
  notas/*.md                 # verdade — frontmatter TOON + corpo
  eventos/events.jsonl       # log append-only (auditoria)
  templates.yaml             # seções obrigatórias por tipo
  validators.toml            # catálogo de checks executáveis (D99)
  .idx/                      # índice derivado, reconstruível
  cache/                     # descartável

# Config global (template/default) — ver §3:
# ~/.config/local/knudge/config.toml
```

| Camada | Papel | Estabilidade |
|---|---|---|
| `notas/` | Fonte da verdade. Um arquivo = uma unidade de recuperação. | **Fixa** |
| `eventos/` | Append-only. Auditoria, `learn`, `diff`, `audit`. Não define ordem. | Recomendada |
| `.idx/` | Índice derivado (retrieval, checkpoint de eventos, sugestões, contextos de rewind, hashes de âncora, embeddings, clusters). | Oscila |
| `config.toml` | Config efetiva do projeto (clone do global). Precedência sobre o global. | Formato fixo |
| `cache/` | Respostas caras / warm start. | Descartável |

**Regra de ouro:** `notas/` + `eventos/` sempre reconstroem `.idx/` do zero. O índice nunca é fonte da verdade.

**Implementação (Rust, binário `kd`):** o código é isolado e documentado por **escopo temático** — `core` (puro: modelo, store, validação), `cli`, `mcp`, `jsonl`, `toon`, `git`, `retrieval`, `embeddings`, `lifecycle`. O núcleo não conhece terminal nem `argv`; os adaptadores são finos. FFI e WASM ficam apenas no planejamento.

### Formato da nota

Markdown com **frontmatter TOON** (Token-Oriented Object Notation), escolhido por reduzir 40–62% dos tokens vs JSON em estruturas de objetos. Sem título/heading/preâmbulo/conclusão no corpo: o campo `statement` **é a afirmação**; o corpo é só o contexto que a afirmação não diz.

---

## 3. Configuração

Um **TOML** centraliza o comportamento do sistema, em **dois níveis**:

```
Global (template/default)   ~/.config/local/knudge/config.toml                  # Linux (ou ${XDG_CONFIG_HOME}/local/knudge/...)
                            ~/Library/Application Support/knudge/config.toml    # macOS
                            %APPDATA%\knudge\config.toml                        # Windows

Projeto (efetivo)           <projeto>/.knudge/config.toml
```

O global é o **modelo** (defaults curados pelo usuário). O do projeto é o **efetivo** e tem **precedência**. Na instanciação, o projeto simplesmente **clona** o global — a partir daí pode divergir.

Dados do projeto — `templates.yaml` e `validators.toml` — continuam dentro de `.knudge/`, porque são **conteúdo**, não configuração.

### Precedência e instanciação

- **Leitura:** valor do projeto → se ausente, valor global.
- **Instanciação:** `onboard()` copia o global para `.knudge/config.toml`. Se o global não existir, cria com os defaults embutidos.
- **Idempotência:** re-rodar `onboard()` **não** sobrescreve um `config.toml` de projeto existente (só com `--force`).
- **Escrita:** `config set` grava no projeto por padrão; `config set --global` grava no global. O LLM escreve só no projeto; o global é curado pelo usuário.

### Persistência do conhecimento no projeto

A configuração principal decide **onde o conhecimento mora**:

```toml
[knowledge]
# true (padrão): cria .knudge/ no projeto e versiona junto.
# false: cria .knudge/ no projeto, mas o exclui do git localmente.
persist_in_project = true
```

| valor | efeito |
|---|---|
| `true` (padrão) | Cria `.knudge/` na raiz do projeto; tudo (notas, eventos, índice, cache) vive ali e **acompanha o repositório**. |
| `false` | Cria `.knudge/` igual, mas adiciona a entrada em **`.git/info/exclude`** automaticamente — o conhecimento fica **local-only**, nunca commitado, invisível para outros clones. |

**Por que `.git/info/exclude` e não `.gitignore`:** `.gitignore` é versionado e compartilhado; editá-lo poluiria o projeto para todos. `.git/info/exclude` é por-clone e não versionado — a exclusão é efetiva, local e não deixa rastro no repositório. (Fora de um repositório git, a exclusão é um no-op.)

O `onboard()` aplica essa regra na inicialização e é **idempotente**: não duplica a linha de exclude e reverte a exclusão se `persist_in_project` voltar a `true`. A mesma chamada clona o `config.toml` (ver acima).

### Outras chaves (não exaustivo)

```toml
[dedup]
create_below = 0.75      # score < X  → cria
merge_below  = 0.92      # X ≤ score < Y → merge; score ≥ Y → rejeita

[recall]
default_limit = 10
expand_depth  = 1
rrf_k         = 60       # fusão de canais (BM25 + âncoras + vetor)

[mcp]
observation_mode = true  # modo observação antes de injetar hints
hints_cap        = 3

[behavior]
strict = false           # promove warnings a erro; config de projeto (D94)

[retention]
foundational_days  = 0    # 0 = nunca expira (D44)
tactical_days      = 365
observational_days = 30
retired_days       = 30   # janela antes da purga do conteúdo aposentado (E10-T03)

[decay]
anchor_threshold = 0.5    # fração válida mínima de âncoras (D43)
grace_days       = 30     # carência antes de demover

[clusters]
min_volume           = 10 # volume mínimo da fase 2 semântica (E10-T07)
similarity_threshold = 0.8

[ids]
prefix_style = "declarative"   # declarative | compact

[embeddings]
enabled    = true
provider   = "http"            # http | lightweight | none (D101)
model      = "ibm-granite/granite-embedding-97m-multilingual-r2"
                              # multilíngue (PT explícito); alternativa EN: msmarco-MiniLM-L12-cos-v5
revision   = "main"            # pinada p/ reprodutibilidade
dimensions = 384
similarity = "cosine"
mode       = "lazy"            # lazy | eager | manual
async      = true              # nunca bloqueia write/read
batch      = 32
max_pending = 1000             # backpressure; acima, força catch-up
cache      = true              # cache por body_hash em .idx/
cache_max_bytes = 33554432     # teto com eviction LRU (32 MiB)
cache_ttl_days  = 30
flush_ms   = 2000              # flush coalescido do índice (debounce)
endpoint   = "http://127.0.0.1:8080/v1/embeddings"   # OpenAI-compatible (llama-server/TEI/Ollama)
timeout_ms = 30000
retries    = 2
api_key_env = "KNUDGE_EMBEDDING_API_KEY"

[hooks]
pre_record  = ""              # comando externo; bloqueia/muta o write (D59)
post_record = ""
pre_prime   = ""              # reservado (prime é byte-idêntico — D57)
pre_prune   = ""
pre_compact = ""
timeout_ms  = 30000           # timeout por hook; acima, kill do grupo de processos
```

O provedor de embedding é plugável via config (global como template, projeto com precedência). O default **`http`** consome um **servidor local** OpenAI-compatible — o usuário sobe `llama-server -m granite-embedding-97M-multilingual-r2-Q8_0.gguf --embeddings` e o knudge só aponta a URL (D101); **não** há inferência in-process (R16/R43). Detalhes, avaliação do modelo e alternativas multilíngues em **`04_embeddings.md`**. Os vetores são **derivados** (`.idx/embeddings.jsonl`, com cabeçalho `meta`), nunca gravados no frontmatter; trocar de modelo força re-embed. `lightweight` (hash) cobre testes/CI e `none` cai para BM25.

Acesso via tool `config get/set/list`. Precedência (quando houver override): flag de CLI > `config.toml`. O LLM **não** altera limiares em runtime — são config, não decisão do agente.

---

## 4. Schema canônico

### Frontmatter

| chave | tipo | obrig. | descrição |
|---|---|---|---|
| `id` | string | sim | prefixo de tipo + base36(8) |
| `type` | enum | sim | tipo (ver abaixo) |
| `statement` | string | sim | a **afirmação**, ≤ 120 chars |
| `created_at` | iso8601 | sim | criação (UTC, segundos) |
| `confidence` | float 0–1 | sim | confiança (default 0.7) |
| `body_hash` | hex8 | sim | hash do corpo (dedup O(1)) |
| `schema_version` | int | sim | versão do schema |
| `tags` | string[] | não | ≤ 5, lowercase, kebab |
| `source` | string/url | não | proveniência |
| `expires_at` | iso8601 | não | expiração |
| `not_before` | iso8601 | não | agendamento (só libera `ready` a partir de; separado da expiração — D56/D100) |
| `superseded_by` | id | não | id da nota que a substituiu (ponteiro reverso de `replaces`) |
| `references` | id[] | não | aresta explícita: menção simples |
| `depends_on` | id[] | não | aresta explícita: depende de |
| `contradicts` | id[] | não | aresta explícita: contradiz |
| `supports` | id[] | não | aresta explícita: suporta / confirma |
| `extends` | id[] | não | aresta explícita: estende / refina |
| `replaces` | id[] | não | aresta explícita: substitui |
| `rejects` | id[] | não | aresta explícita: rejeita |
| `results_in` | id[] | não | aresta explícita: resultado |
| `revision` | int | não | contador de edições (default 1) |
| `outcomes` | array | não | histórico `{status, duration, agent, notes, recorded_at}`; confirmação **derivada** |
| `classification` | enum | não | `foundational\|tactical\|observational` (default tactical) |
| `anchors` | string[] | não | paths/globs de arquivo |
| `status` | enum | não | `active\|in_progress\|blocked\|closed\|superseded\|forgotten` |
| `scope` | enum | não | `plan\|epic\|issue\|task` (só `type=task`/`container`; D93) |
| `checks` | string[] | não | validators (só `type=task`) |
| `evidence` | map | não | resultados de validators (preenchido no close) |

Chave desconhecida = **rejeita no write**. Sem `author` e sem `updated_at` (o evento cobre).

O bloco de arestas (`references`…`results_in`) fica **entre `superseded_by` e `revision`** na ordem canônica — **28 chaves** ao todo (D98/D100), com `not_before` logo após `expires_at`. Cada aresta é uma lista de ids derivada do `link()` e validada quanto ao formato; o `doctor` cobra a bidirecionalidade `replaces ↔ superseded_by`.

### Tipos (`type`) — enum fechado

```
fact       assertiva verificável
decision   escolha + justificativa
question   aberto, não resolvido
task       ação pendente
def        termo → significado
error      problema + causa + fix
snippet    trecho de código reutilizável
link       referência externa + por quê
meta       conhecimento sobre o próprio sistema
container  agregação explícita (sem verdade própria)
risk       conhecimento preditivo (confidence = probabilidade)
```

Fechado de propósito: tipo aberto faz o LLM inventar categoria e degrada o retrieval.

### IDs

`<prefixo>_<base36(8)>`, com prefixo declarativo por tipo:

| tipo (`type`) | prefixo | exemplo |
|---|---|---|
| `fact` | `fact_` | `fact_7a3c1b2d` |
| `decision` | `decision_` | `decision_4b22e901` |
| `question` | `question_` | `question_2e09aa13` |
| `task` | `task_` | `task_9a3c77f0` |
| `def` | `def_` | `def_8f12c4d5` |
| `error` | `error_` | `error_7c11b6e2` |
| `snippet` | `snippet_` | `snippet_1e09d3a8` |
| `link` | `link_` | `link_5a12f0b7` |
| `meta` | `meta_` | `meta_3d44a9c1` |
| `container` | `container_` | `container_c9a3e2f4` |
| `risk` | `risk_` | `risk_6b77d1a0` |

O prefixo é **filtro de graça**: o LLM sabe o tipo sem parsear frontmatter. O trade-off é o ID mais longo; em troca, o tipo fica legível diretamente no `id + statement + score` do `recall`, sem consulta ao schema.

### Arestas — vocabulário fechado

```
references    menção simples
depends_on    depende de
contradicts   contradiz
supports      suporta / confirma
extends       estende / refina
replaces      substitui
rejects       rejeita (decision → alternativa descartada)
results_in    resultado (decision → o que aconteceu)
```

**Arestas explícitas são a fonte primária** (via `link()` ou campo explícito). A extração por regex conservadora (ids, wikilinks, padrões verbais) roda no **write** como **sugestão revisável**, nunca como aresta definitiva. O `expand` confia no explícito; o `audit` sugere as faltantes.

---

## 5. Superfície (`kd`)

O teto inicial de "sete" caiu. O critério passou a ser **poucos verbos, muitos modos** (inspirado
no Docker): a superfície v2 está congelada em `implementation/16_cli_surface.md`.

| Verbo | Absorve | Papel |
|---|---|---|
| `kd` (sem args) | = `kd prime` | protocolo estático |
| `kd init` | `onboard` | funda `.knudge/` + prompt inicial |
| `kd prime` | — | protocolo de uso, byte-idêntico |
| `kd rewind` | `prime(scope)`, `get_context`, `diff` | estado/handoff ponto-no-tempo |
| `kd ask` | `recall`, `get`, `expand` | toda pesquisa |
| `kd write` | `write`, `update`, `link` | toda escrita |
| `kd task` | `plan`, containers de tarefa | plan/epic/issue/task (D93) |
| `kd maintenance` | `doctor`, `audit`, `compact`, `eval`, `embed`, `learn` | manutenção |
| `kd config` | `config` | `.knudge/config.toml` |
| `kd forget` | `forget`, `restore` | soft-delete |
| `kd sync` | `sync` | commit git |
| `kd self` | `setup`, `completions`, `upgrade`, `version` | instalação |

### Contratos importantes

- **Formato de retorno é parte do schema.** `recall` retorna pipe-delimitado, sem JSON: `fact_7a3c1b2d|Rust > JSON por decodificador formal|0.91|file_match` (~15 tokens/hit). A 4ª coluna é `why` (por que o hit apareceu). Corpo só chega via `get`, e só dos ids pedidos.
- **Protocolo de escrita em duas fases.** `write` é precedido por `recall` obrigatório; score < 0.75 cria, 0.75–0.92 faz merge, ≥ 0.92 rejeita. O limiar é **config do sistema**, não do LLM. Com embeddings **assíncronos**, o score é **lexical (BM25)** enquanto o vetor não existe; o dedup **semântico é eventual** (reconciliação em background).
- **`update` versiona, não sobrescreve.** Mudança de `type` = nova nota + `replaces`. Supersede deixa cadeia caminhável (`get(id, history=True)`).
- **`status` unifica ciclo de vida.** `forget`, `supersede` e `close` viram `update(id, status=...)`. `ready`/`blocked` são **views** computadas no `recall` (via `depends_on` transitivo), não tools.
- **`prime` é estático; `rewind` é dinâmico.** `prime` imprime o protocolo token-optimized
  (tipos, tools, regras, orçamento), **byte-idêntico** por versão do binário — é o "help" da IA;
  `kd` sem argumentos = `kd prime`. `rewind` situa o estado entre agentes/rodadas: manifest ~30
  tokens, `--scope` (container/domínio), `--files` (working set ancorado), orçamento de tokens
  (default 4000), ranking por trust-tier, auto-context-scope (`git status` + working set) e
  contagem de **`embeddings_pending`**. Emite um **`context_id`** endereçável, recuperável
  **1:1** por `kd rewind --resume <id>` (D88).
- **`kd init` funda o projeto** (`.knudge/` + `AGENTS.md` com marcadores idempotentes + prompt
  inicial de fundação de conhecimento/tarefas). O protocolo de uso vive no `prime`.

---

## 6. Retrieval — estrutura antes de estatística

Regra geral que emergiu da comparação com seeds e mulch:

> **Estrutura é sinal determinístico; similaridade é sinal probabilístico. Use o primeiro para filtrar; o segundo só para o resíduo.**

Cascata de recall, da camada mais barata à mais cara:

```
recall(q, type, classification, anchors, container, tags, status)
  ↓ filtro por campos do frontmatter      (determinístico, O(1))
  ↓ filtro por arestas no grafo           (determinístico, BFS limitado)
  ↓ similaridade textual no resíduo       (BM25, O(N') com N' << N)
  ↓ similaridade semântica se BM25 falha  (embeddings, opcional)
  ↓ fusão RRF dos canais                  (BM25 + âncoras + vetor; k=60)
  ↓ clusters semânticos só dentro de clusters estruturais (batch, off-path)
```

### BM25 / Embeddings / Clusters — veredicto

- **BM25** (`k1=1.5, b=0.75`) é a similaridade de primeira linha, com tokenização **ASCII explícita**, **IDF por campo** (o `statement` domina) e por tipo, e **boost por confirmação** (`score * (1 + 0.1 * (success + partial*0.5))`).
- **Embeddings** são um **provedor plugável** via config (`.idx/embeddings.jsonl`), justificados por 3 casos: descoberta de links não-declarados, `learn()` e dedup semântico. Default `granite-embedding-97m-multilingual-r2` (384d, cosseno nativo, multilíngue — D123); `none` desliga e cai para BM25 puro. Consumo **assíncrono e lazy**: notas recém-criadas ficam “dark” no espaço vetorial até serem digeridas — **gap tolerado**; `recall` nunca espera. Ver `04_embeddings.md`.
- **Fusão RRF + determinismo:** canais (BM25, âncoras, vetor) fundidos por `1/(k+rank+1)` (k=60) e ordenados por `(score desc, id asc)` — duas execuções idênticas nunca divergem. Canal ausente/falho degrada para o lexical **sem quebrar** a busca (D81).
- **Âncoras são canal de recall**, não só campo: match determinístico por `path`/`id` antes da estatística.
- **Similaridade clampada `[0,1]`** e **confiança derivada** (`sim × drift × idade + feedback`) calculada no `recall` — **nunca armazenada** (a `confidence` declarada é outra coisa) (D87).
- **Clusters** viram **duas fases**: (1) estrutural/determinístico (agregação por `anchor`, `type`, `classification`, container — auditoria barata); (2) semântico/probabilístico só dentro de um cluster estrutural. `compact` consolida redundância acumulada que o `recall` capado nunca mostra.
- **Rejeitados:** sqlite-vec, Chroma, LanceDB — resolvem problemas (escala, concorrência) que o sistema não tem e quebram a premissa "índice é só arquivos".

---

## 7. Concorrência

O brainstorm testou dois cenários:

- **50 agentes não-orquestrados** quebraria o pressuposto de escritor único (dedup read-then-write, `update` lost-update, rebuild vs leitores, containers stale, `sync` concorrente). Exigiria log-first + CAS + double-buffer + dedup eventual.
- **Cenário adotado: 2 agentes orquestrados** (1 master + 1 subagente, nunca concorrentes). **Quase todas as contenções caem.** Volta-se ao ADR-002 (`notas/` é a verdade).

O que **sobrevive** (não por concorrência, mas por outras razões):

1. Escrita atômica (`fsync` + `rename`) — crash safety.
2. `eventos.jsonl` append-only — raciocínio temporal (`learn`, `diff`, `audit`).
3. `revision` — sinal de volatilidade, não CAS.
4. **Canônico antes do derivado** — um crash deixa no pior caso um vetor órfão, **nunca** uma nota ausente.
5. **Vetor é descartável** — toda remoção (supersede, merge, `compact`, TTL, dedup) **purga o vetor**; `doctor` detecta divergência canônico↔derivado (D84).
6. **Flush coalescido** do `.idx` (debounce, flush forçado na saída) — rajadas não viram N rewrites (D85).

O ganho da orquestração é **escopo delegado**: o master define `{containers, anchors, type}` no `prime(scope, files)`; o subagente só vê e escreve dentro do escopo; o master reconcilia (reindex incremental, dedup, sync, limpeza de `tmp/`).

> A variável que importa é **orquestração**, não contagem de agentes.

---

## 8. MCP proativo

Vale, mas **estreitamente** — o MCP deve ser **reativo a comportamento**, não proativo por iniciativa própria. Três gatilhos que se pagam:

1. **Pré-`write`** (mais forte): retornar quase-duplicados na resposta do `write` — o único momento em que a duplicata é acionável.
2. **Pré-edição de arquivo**: ao editar um path, anexar os `statement` de notas ancoradas (`kd rewind --files` contínuo).
3. **Fechamento de sessão**: se houve diff significativo e zero writes, injetar `"kd maintenance learn?"`.

Restrições: hint é **ponteiro, nunca conteúdo** (`id + statement + score`); dedup por sessão; cap de 3 hints. Modo observação por N sessões antes de produção, calibrado por taxa de seguimento (> 50%). Injeção por similaridade de conversa, padrão de erro ou "decisão detectada" foi rejeitada como ruído.

**Transporte (E14).** O servidor é local e sem rede: JSON-RPC 2.0 sobre **stdio**, uma linha JSON por mensagem, expondo `knudge_pre_write`/`pre_edit`/`session_end`/`status` como tools. O binário `knudge-mcp` lê `mcp.hints_cap`/`mcp.observation_mode`/`mcp.observation_sessions` de `.knudge/config.toml` e cai nos defaults quando ausente. `resources`/`prompts` do MCP ficam de fora: os hints são tools e o conteúdo mora no `kd`.

---

## 9. Gestão de tarefas

Task **não é subsistema** — é `type=task`, um tipo entre onze. Campos mínimos:

```yaml
type: task
status: active | in_progress | blocked | closed | superseded | forgotten
outcomes: [{status: success|partial|failure|abandoned}]   # só ao fechar
checks: [validator-names]                          # delta declarado
anchors: ["V2/Modules/Noticias/**"]
depends_on: [decision_4b22e901]
```

**Cortados como resíduo de workflow humano:** `priority`, `assignee`/`claimed_by`, `acceptance` em prosa. Nenhum resolve problema de 2 agentes, 1 projeto, orquestrado, sem deadline.

**Acceptance reformulado:** não prosa por task, mas **referência a um catálogo de validators executáveis** (`validators.toml`, D99). A resolução de checks combina três fontes:

```
checks(task) = checks_explícitos(task)
             ∪ validators_globais(AGENTS.md)
             ∪ validators_por_anchor(anchors(task))
```

O fechamento deixa de ser declaração e passa a ser **evidência**: o sistema roda os comandos e infere o `outcome` (acrescentado a `outcomes[]`). Resultado vai para `evidence` (leitura rápida) e `eventos/` (auditoria).

---

## 10. Absorvido de seeds e mulch

| Padrão | Origem | Forma no sistema |
|---|---|---|
| `alternatives` + `rejected_because` | seeds | aresta `rejects` + nota `def` |
| split container/atômico | seeds | `type=container` (view, sem verdade própria) |
| `risks` | seeds | `type=risk` com `confidence` e `expires_at` |
| `outcome` | seeds | campo `outcomes[]` (confirmação derivada) |
| `revision` | seeds | campo `revision` |
| templates/moléculas | seeds | `templates.yaml` |
| `prime` / fechamento | seeds | família `prime()` + `onboard()` |
| git-native | seeds | `sync()` |
| `foundational/tactical` | mulch | campo `classification` (+ `observational`) |
| `dir_anchors` / `prime --files` | mulch | campo `anchors` + `prime(files)` |
| `learn` (git diff) | mulch | `learn()` |
| lifecycle (rank/prune/archive) | mulch | `outcomes[]`, `expires_at`, `status` |
| `audit` | mulch | `audit()` |
| `diff` | mulch | `diff(since, until, scope)` |
| `compact` | mulch | `compact(scope)` |
| `plan` | seeds | `plan(task_id, steps)` |
| `restore` / `onboard` / `config` | mulch | tools equivalentes |

**Rejeitados:** domínios como partição física (containers + tags + anchors resolvem), record sem split frontmatter/corpo (perde indexação sem ler o corpo), `priority`/`assignee`/`acceptance` em prosa.

**Aprendizado central:** o sistema tem **três tempos** — presente (`recall`/`get`/`expand`), passado (`diff`/`superseded_by`/`revision`) e futuro (`plan`/`risk`/`question`). E **consolidação (`compact`) é a única operação que o LLM nunca faz sozinho**, porque exige ver 15 notas simultaneamente — algo que o `recall` capado por tokens nunca mostra.

---

## 11. Fixo vs. oscilante

| Peça | Estabilidade |
|---|---|
| Schema (tipos, chaves, IDs, arestas) | **Fixo** — bump de versão para mudar |
| Contrato das tools (assinaturas e formato de retorno) | **Fixo** — é o que o LLM vê |
| Protocolo de escrita (dedup, limiares) | **Fixo** — limiar é config, não runtime |
| Estrutura de diretórios | **Fixo** — `notas/` é verdade, resto derivado |
| Stack e binário (`kd`, Rust) | **Fixo** — um único executável |
| `config.toml` (global + projeto) | **Formato fixo** — o do projeto tem precedência; os valores oscilam |
| Formato do índice | Oscila (JSON → binário quando justificar) |
| Embeddings | Oscila (opcional, derivado) |
| Cache/estado de embedding (`indexed\|pending\|stale`) | Oscila (derivado, em `.idx/`) |
| Âncoras com `content_hash` | Oscila (hash derivado em `.idx/`; o `path` fica na nota) |
| Confiança/salience e decay derivados | Oscila (calculados no `recall`; `confidence` declarada é fixa) |
| Clusters | Oscila (batch, opcional) |
| Watcher | Oscila (conveniência) |
| Eventos | Oscila (recomendado) |

O que fica fixo é o **contrato semântico**; o que oscila é a **implementação do índice**.

---

## 12. Ordem de adoção

1. **Schema v1** + bootstrap do `config.toml` (com `persist_in_project`) com `classification`, `anchors`, `status`, `outcomes[]`, `type=risk`, `type=container`, arestas `rejects`/`results_in`. Sem isso, nada funciona.
2. **Índice por campos + grafo** — filtros determinísticos (já resolve o caso comum).
3. **BM25 sobre corpus filtrado** — similaridade residual, barata.
4. **`kd rewind --files` + `kd maintenance learn`** — os dois casos que justificam ancoragem e git diff.
5. **`audit()` estrutural** — clustering fase 1, sem estatística.
6. **Embeddings** — só quando `learn()` e dedup semântico existirem; provedor plugável, cache por `body_hash`, consumo assíncrono/lazy e estado `pending` reconciliável (D79/D80/D83).
7. **Clustering semântico** — só com volume que justifique.

Os passos 1–5 são pré-requisitos entre si; 6–7 são independentes e podem esperar. A tentação é começar por embeddings ("parece RAG moderno") — mas com a estrutura de seeds e mulch, embeddings são a **última** peça.

### Mínimo viável

```
.knudge/
  notas/                  # .md com frontmatter TOON
  .idx/retrieval.jsonl    # índice invertido + forward (E06)
  .idx/events.checkpoint  # checkpoint do log de eventos (E03)
  .idx/contexts/          # contextos de rewind (handoff 1:1 — E08)
  .idx/anchors.jsonl      # hashes de âncora (verify-on-hit — E09)

kd                 # binário único (Rust), módulos por escopo:
                   # core, cli, mcp, jsonl, toon, git, store, graph, retrieval, write, handoff, maintenance, task, lifecycle
```

MVP em Rust, com núcleo puro + adaptadores `cli`/`mcp`. Sem eventos, embeddings, clusters ou watcher. O resto é otimização incremental.

---

## 13. Em uma frase

**Arquivos Markdown como verdade, um índice derivado reconstruível, e um conjunto enxuto de tools que entrega o mínimo de contexto necessário.** O LLM navega por um grafo de conhecimento atômico, puxando só o que precisa quando precisa; o binário `kd` é a única peça externalizada, e pode começar pequeno, evoluindo só quando o volume exigir.

---

## Anexo — mapa dos arquivos do brainstorm

| Arquivo | Tema |
|---|---|
| `1.md` | Ideia inicial (TOON + file-based) e **Schema canônico v1** (frontmatter, tipos, IDs, arestas, tools, protocolo de escrita, dedup/supersede, versionamento) |
| `2.md` | Proximidade sem banco vetorial — níveis 0 (TF-IDF) a 3 (clustering batch) |
| `3.md` | **Panorama do sistema** + absorção do **seeds** (espaço de decisão, `alternatives`, `prime`) |
| `4.md` | Absorção do **mulch** (`classification`, `anchors`, família `prime`, `learn`, `audit`) + revisão de TF-IDF/embeddings/clusters |
| `5.md` | Concorrência: 50 agentes vs. 2 orquestrados (escopo delegado) |
| `6.md` | **MCP proativo** (3 gatilhos), premissa global, `status` unificado, corte de `priority`/`assignee`/`acceptance`, e **validators** |
| `7.md` | Lacunas finais: `diff`, `compact`, `plan`, `restore`, `onboard`, `config` |

> **Complemento:** `01_gaps-plan-rs.md` confronta este panorama com a investigação profunda de `refs/mulch-rs/plan-rs/` e `refs/seeds-rs/plan-rs/`, listando gaps de contrato de bytes, aliases de schema, lock, worktree, decay de âncoras, BM25/boost/orçamento, hooks, arquitetura core/adapter e disciplina de testes — com priorização.
>
> **Decisões:** `02_decisoes.md` é o registro dos 78 pontos de decisão (bloqueantes/estruturantes/adiáveis), com opções, recomendação e o problema que cada um previne.
>
> **Decisões fechadas:** `03_decisoes-fechadas.md` contém as respostas finais e os impactos já propagados neste panorama.
>
> **Embeddings:** `04_embeddings.md` avalia os modelos MS MARCO, define o provedor de vetores via `config.toml` e a escolha default (`granite-embedding-97m-multilingual-r2`).
>
> **Referência arags:** `05_refs-agnostic-rag.md` revisa `refs/agnostic-rag-rlm-tool/` (a mesma busca de memória por um caminho servidor-first); as extrações estão fechadas nas decisões **D81–D92**, com simplicidade e localidade como meta.
>
> **Implementação:** `implementation/` contém o plano por **épicos e tarefas** (E01–E13), com fases, dependências, aceite e rastreabilidade `D→épico`. Comece por `implementation/README.md`.
