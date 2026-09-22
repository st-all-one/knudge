# 16 — Superfície do `kd` v2 (contrato congelado)

> Revisão da superfície do binário inspirada no **Docker** (verbos amplos + subcomandos),
> decidida após o esqueleto de E01. Este documento é o **contrato congelado**: os nomes, o
> agrupamento e as opções abaixo são o que o LLM e as máquinas veem.
>
> **Decisões:** D57 (reescrita), D69, D88 (ajustada), D93, D94.
> **Substitui:** a lista de verbos de `12_cli_mcp_distribuicao.md` (E12-T01) e a tabela de tools
> do `00_panorama.md` §5.

---

## 1. Princípios

1. **Poucos verbos, muitos modos.** O LLM escolhe entre ~10 verbos; o resto é flag/subcomando.
2. **`prime` é estático; `rewind` é dinâmico.** Protocolo nunca muda entre execuções; estado muda.
3. **stdout = dados, stderr = logs** (R20). `--json` é o contrato de máquina (D71).
4. **`strict` é config de projeto** (D94), não flag.
5. **Tudo de tarefa vive em `kd task`** (D93); `kd write` rejeita `--type task|container`.

## 2. Verbos de topo

| Verbo | Absorve | Papel |
|---|---|---|
| `kd` (sem args) | = `kd prime` | protocolo estático |
| `kd init` | `onboard` | funda `.knudge/` + prompt inicial |
| `kd prime` | — | protocolo de uso, byte-idêntico |
| `kd rewind` | `prime(scope)`, `get_context`, `diff` | estado/handoff ponto-no-tempo |
| `kd ask` | `recall`, `get`, `expand` | toda pesquisa |
| `kd write` | `write`, `update`, `link` | toda escrita |
| `kd task` | `plan`, containers de tarefa | plan/epic/issue/task |
| `kd maintenance` | `doctor`, `audit`, `compact`, `eval`, `embed`, `learn` | manutenção |
| `kd config` | `config` | `.knudge/config.toml` |
| `kd forget` | `forget`, `restore` | soft-delete |
| `kd sync` | `sync` | commit git |
| `kd self` | `setup`, `completions`, `upgrade`, `version` | instalação |

## 3. `kd ask` — toda pesquisa

Default: **recall completo** (filtros → BM25 → âncoras → RRF). Pipe `id|statement|score|why`.

```
kd ask [QUERY]
  --id <ID>...            # get: corpo só dos ids pedidos
  --around <ID>           # expand no grafo explícito
  --via <ARESTA>          # tipo de aresta (default: todas)
  --depth <N>             # profundidade do expand (default 1)
  --brief                 # saída mínima: id|statement
  --with-body             # inclui corpo dos hits
  --type <T>...           # filtros determinísticos
  --class <C>...          # foundational|tactical|observational
  --tag <T>...
  --status <S>...
  --container <ID>
  --anchor <PATH>...      # repetível; aceita vírgula (`--anchor a,b`)
  --since <TS> / --until <TS>
  --limit <N>             # default: config recall.default_limit
  --json
```

## 4. `kd write` — toda escrita

Create idempotente por conteúdo + protocolo de dedup (0.75/0.92). `--update` versiona;
`--link` cria aresta explícita. **Rejeita `--type task|container`** (use `kd task`).

```
kd write [STATEMENT]
  --type <T>              # default: fact
  --body <TXT|->          # '-' lê stdin
  --tag <T>... --anchor <PATH>... --check <NAME>...
  --source <S> --class <C> --status <S> --expires-at <TS> --confidence <F>
  --edge <ARESTA:ID>      # aresta explícita na criação
  --update <ID>           # modo update (patch versionado)
  --link <ARESTA:ID>      # cria aresta (substitui o antigo `link`)
  --outcome <S> <ID>      # anexa evidência a qualquer nota (D103); com [--note <TXT>]
  --dry-run
  --json                  # {action: created|merged|rejected|updated, id}
```

## 5. `kd prime` — protocolo estático

**Sempre a mesma resposta** para uma dada versão do binário (cacheável, byte-idêntico):
tipos, tools, regras, orçamento, formato de saída. `kd` sem argumentos executa `kd prime`.

```
kd prime [--long]         # --long inclui a gramática TOON e o schema completo
```

## 6. `kd rewind` — estado/handoff ponto-no-tempo

O antigo `prime(context)` + `get_context` + `diff`.

```
kd rewind
  --scope <CONTAINER>
  --files <PATH>...
  --budget <N>            # default 4000 (ceil(len/4), D40/D82)
  --since <TS> / --until <TS>
  --resume <CONTEXT_ID>   # handoff 1:1 (D88)
  --json
```

## 7. `kd task` — plan/epic/issue/task (D93)

Hierarquia **fechada**: `plan ⊃ epic ⊃ issue ⊃ task`, profundidade máx. **4**. Campo `scope`
(enum fechado) em `type=container` (plan/epic) ou `type=task` (issue/task). Pai único via
membership/backref (D52); dependências via aresta `depends_on`.

```
kd task new <STATEMENT> --scope <plan|epic|issue|task> [--parent <ID>]
  [--body <TXT|->] [--checks <NAME>...] [--anchor <PATH>...] [--source <F>]
  [--depends-on <ID>...] [--not-before <TS>] [--expires-at <TS>]
kd task list [--scope ...] [--status ...] [--parent <ID>]
  [--ready|--blocked [--explain]]
kd task show <ID> [--history]
kd task graph --program <PATH>
kd task update <ID> [--statement <S>] [--status <S>] [--parent <ID>] [--checks ...]
kd task close <ID> [--outcome success|partial|failure|abandoned]
kd task plan <ID> [--submit|--adopt|--reorder <N>|--release|--review]
  --step <TXT>...
```

- `close` roda os validators e grava `outcomes[]`/`evidence` (D48/D55) — nunca declara sem evidência.
- `plan` implementa o ciclo de vida de D53 (`blocks` 1-based, sem self-reference, detecção de ciclo).
- `plan`/`epic` são **views derivadas** (sem verdade própria); `issue`/`task` são atômicas.
- `list --ready|--blocked` filtra pelas views derivadas; `--explain` acrescenta o motivo (D104).
- **Programa externo** (D119): `plan/<slug>.md` ancorado a um Épico-raiz (`--anchors`);
  `task graph --program` imprime a subárvore; `programs.glob` define o que é um programa.

## 8. `kd maintenance` — manutenção

```
kd maintenance doctor [--fix] [--audit]   # relatório por padrão; --fix corrige o reversível
kd maintenance compact [--scope <C>]      # propõe merge/supersede (nunca em silêncio)
kd maintenance eval --ab <A> <B>          # Recall@k / nDCG@k / MRR
kd maintenance index [--drain|--status]   # fila de embeddings
kd maintenance learn [--scope <C>]        # sugestões de notas/links/merges
```

- `audit` virou modo do `doctor` (relatório de integridade + arestas sugeridas).
- `link` **não** mora aqui: virou `kd write --link`.
- `index` é, por padrão, interno (worker); `--status`/`--drain` são diagnóstico.

## 9. `learn` em profundidade

`learn` é o **motor de institucionalização de conhecimento**: uma operação **read-only** que olha
para o que **aconteceu** (eventos + `anchors` + grafo + vetores) e responde
*"o que deveria ter virado nota e não virou?"* e *"o que virou redundância e deveria ser
consolidado?"*. **Nunca escreve**: emite **propostas** que o agente aplica via `write`/
`write --link`/`maintenance compact` (D47).

Três sinais:

1. **Atividade sem registro (write-gap).** Eventos + `anchors` mostram arquivos tocados num
   escopo, mas zero `write` ali. Ex.: a sessão editou `V2/Modules/Noticias/**` e não criou nota.
   `learn` propõe uma nota candidata com `type` inferido (`decision`/`error`/`snippet`) e
   `statement` derivado do diff de âncoras/eventos.
2. **Repetição / quase-duplicata (dedup eventual).** `recall`/`write` repetidos sobre o mesmo
   tema, ou vetores próximos (E11-T09), propõem `compact` (merge/supersede).
3. **Lacuna de grafo (link faltante).** Notas relacionadas por âncora/tema sem aresta explícita
   propõem `write --link`.

Formato da proposta (ponteiro, nunca conteúdo): `{kind, ids, why, score}` com
`kind ∈ {create_note, merge, supersede, link}`.

Restrições:
- **Determinístico**: deriva só de eventos + âncoras (+ vetores), **nunca** do diff global do git
  (D33). O `git status` alimenta o *working set* do `rewind`, não o `learn`.
- **Nada sem aceite** (D47): toda proposta é revisável.
- **Session-close**: o MCP gatilho 3 chama `learn` quando houve diff significativo e zero writes.
- **Onde vive**: `kd maintenance learn` (é relatório/sugestão, como `doctor`/`compact`).

## 10. `kd config`, `kd forget`, `kd sync`, `kd init`, `kd self`

```
kd config get <KEY>
kd config set <KEY> <VALUE> [--global]     # projeto por padrão; grava .knudge/config.toml
kd config unset <KEY> [--global]
kd config list [--global]

kd forget <ID>             # status=forgotten (soft; nunca apaga arquivo)
kd forget <ID> --restore   # volta a active
kd forget <ID> --purge     # hard-delete só após a janela de retenção

kd sync [--message <MSG>]  # commit de notas/ + eventos/ no worktree certo

kd init [--force] [--no-prompt]   # estrutura canônica + prompt inicial de fundação

kd self setup <claude|cursor|codex|pi>
kd self completions <shell>
kd self upgrade
kd self version
```

## 11. `strict` como config (D94)

`strict` **não é flag nem subcomando**: é chave de projeto em `.knudge/config.toml`:

```toml
[behavior]
strict = false   # true promove warnings (leitura, retrieval, embeddings) a erro
```

## 12. Mapa antigo → novo

| Antes | Agora |
|---|---|
| `recall` / `get` / `expand` | `kd ask` / `kd ask --id` / `kd ask --around` |
| `write` / `update` | `kd write` / `kd write --update` |
| `link` | `kd write --link` |
| `forget` / `restore` | `kd forget` / `kd forget --restore` |
| `prime` | `kd prime` (estático) |
| `prime(scope)` / `get_context` / `diff` | `kd rewind` / `kd rewind --resume` / `kd rewind --since` |
| `learn` | `kd maintenance learn` |
| `plan` | `kd task` |
| `compact` / `audit` / `doctor` / `eval` / `embed` | `kd maintenance …` |
| `onboard` | `kd init` |
| `setup` / `completions` / `upgrade` / `version` | `kd self …` |

## 13. MCP espelhado

As tools MCP usam os mesmos nomes e modos. Os 3 gatilhos (E12-T03) passam a apontar para:
pré-`write` (quase-duplicados), pré-edição (`kd rewind --files` contínuo), fim de sessão
(`kd maintenance learn`).

## 14. Aceite

- [x] `kd` sem argumentos == `kd prime`; `prime` byte-idêntico por versão (teste de golden).
- [x] `ask` cobre recall/get/expand com um só envelope.
- [x] `write` rejeita `task`/`container`; `--link` cobre arestas.
- [x] `task` valida hierarquia fechada (profundidade ≤ 4, pai único, sem ciclo).
- [x] `strict` lido só do config; nenhuma flag `--strict` existe.
- [x] `learn` só propõe; nenhum write sem aceite.

Detalhe por verbo (pipe/`--json`/erro/exit/estado) em
[`17_matriz_aceitacao.md`](17_matriz_aceitacao.md).
