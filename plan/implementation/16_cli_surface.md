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
| `kd knowledge` | `clusters` | mapa de conhecimento (D128) |
| `kd maintenance` | `doctor` (`--audit`), `compact`, `eval`, `index`, `learn`, `prune` | manutenção |
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
  --limit <N>             # default: config recall.default_limit (5 — D121)
  --tags                  # vocabulário de tags: tag|count (count desc, D107)
  --rank                  # ranking por confiança derivada, sem query (D107)
  --json
```

O canal **vetorial** entra automaticamente quando `recall.semantic = true` (default) e há índice
(`recall.semantic_top_k`); provedor fora do ar degrada para BM25 com `warnings` (D102). Um hit
que veio pelo vetor aparece com `why = semantic` (D121); o canal lexical descarta stopwords e
fragmentos de 1 char (`content_terms`, D122).

Sem nenhum modo (query e âncora vazias, sem `--id`/`--around`/`--rank`/`--tags`), o `ask` devolve
o **uso** do comando com exit 2 em vez de sair vazio (D130). `--id` de nota ausente degrada para
`warnings`; `--around` de nota ausente é `not_found` (3).

O **feedback tarefa→conhecimento** (X1/D108) é derivado em tempo de consulta: tarefas com
`outcomes` de sucesso que compartilham `anchors` confirmam a nota — o peso
`recall.confirmation_from_tasks` (float, default `0.1`) entra no boost do BM25 e em
`hits[].confidence`, e o manifest de `rewind` promove a `star`. Sem `write` (D87).

## 4. `kd write` — toda escrita

Create idempotente por conteúdo + protocolo de dedup (0.75/0.92). `--update` versiona;
`--link` cria aresta explícita. **Rejeita `--type task|container`** (use `kd task`) e
`statement` vazio é `invalid_input` (2) — nunca cria nota vazia (D130).

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
  --batch <FONTE|->       # lote JSONL de rascunhos (D110); com [--dry-run]
  --dry-run
  --json                  # {action: created|merged|rejected|updated, id}
```

## 5. `kd prime` — protocolo estático

**Sempre a mesma resposta** para uma dada versão do binário (cacheável, byte-idêntico):
tipos, tools, regras, orçamento, formato de saída. `kd` sem argumentos executa `kd prime`.
O corpo é organizado por **fluxo** — `CICLO` (ask→write→task→sync), `CONHECIMENTO`,
`PESQUISA`, `TAREFAS` — e recomenda `--limit`/`--brief` para economizar contexto (D130).

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

O manifest (default) ganha `next:` (tarefas `ready` abertas por impacto) e `fresh:`
(`stale`/`expiring`/`pending`) — D106. `K` deriva do orçamento; o excedente vira `dropped`.

## 7. `kd task` — plan/epic/issue/task (D93)

Hierarquia **fechada**: `plan ⊃ epic ⊃ issue ⊃ task`, profundidade máx. **4**. Campo `scope`
(enum fechado) marca o **nível**; o `type` é a **espécie** (D113): `container` para plan/epic,
`task`/`error`/`question`/`risk`/`decision` para itens de trabalho. Pai único via
membership/backref (D52); dependências via aresta `depends_on`, criada **só** com
`kd write --link <FROM:depends_on:TO>` (via única, D126).

```
kd task new <STATEMENT> --scope <plan|epic|issue|task>
  [--kind <task|error|question|risk|decision>] [--parent <ID>]
  [--body <TXT|->] [--checks <NAME>...] [--anchor <PATH>...] [--tag <T>...] [--source <F>]
  [--not-before <TS>] [--expires-at <TS>]
kd task list [--scope ...] [--status ...] [--kind ...] [--parent <ID>]
  [--ready|--blocked [--explain]] [--sort impact] [--tag <T>...] [--anchor <PATH>...]
  [--since <TS>] [--owner <A>|--mine]
kd task show <ID> [<ID>...] [--history]   # + pai/bloqueadores/filhos/épico (D125/D127)
kd task graph [--program <PATH>|--root <ID>]   # containers com progresso (D127)
kd task update <ID> [--statement <S>] [--status <S>] [--parent <ID>] [--checks ...]
kd task close <ID> [--outcome success|partial|failure|abandoned] [--note <TXT>]
kd task claim <ID> --by <A>|--release
kd task plan <ID> [--prompt [--template <NOME>] | --submit --from <TXT|->]
  [--step <TXT>...] [--adopt|--reorder <N>|--release|--review]
```

- `close` roda os validators e grava `outcomes[]`/`evidence` (D48/D55) — nunca declara sem
  evidência; acrescenta o **épico mais próximo** e o **progresso** dele (D127).
- `show` resolve o **contexto** de cada id — `parent`, `blocked_by`, `blocks`, `children` e o
  **épico com progresso** — com **título** e estado, para responder "onde isto se encaixa e o
  que o bloqueia" num só comando (D125/D127); no `--json`, os campos vêm estruturados.
- **Rollup de progresso por épico (D127):** `epic_of`/`progress_of` contam os **itens de trabalho
  folha** (`is_work_item` sem filhos de trabalho) no subárvore e quantos estão `closed` —
  derivado, sem verdade nova. Aparece no `close` (`epico: <id>|<título> (<done>/<total>)`), no
  `show` e nos containers do `task graph` (`(done/total)`).
- **Arestas (inclui `depends_on`) têm uma via única:** `kd write --link <FROM:ARESTA:TO>` (D126).
  `kd task new` não cria arestas — reduz a superfície e reaproveita o caminho de grafo.
- `plan` implementa o ciclo de vida de D53 (`blocks` 1-based, sem self-reference, detecção de ciclo).
- `plan`/`epic` são **views derivadas** (sem verdade própria); `issue`/`task` são atômicas.
- `--kind` grava a **espécie** mantendo o `scope` (D113); `--owner`/`--mine` filtram pelo dono
  **derivado** de `claim`/`release` (D114, `KNUDGE_AGENT`).
- `list --ready|--blocked` filtra pelas views derivadas; `--explain` acrescenta o motivo (D104).
- `list --sort impact` ordena o **caminho crítico** por impacto de desbloqueio
  (`impacto desc, created asc, id asc`); `--explain` acrescenta `unblocks=N` e o `--json` traz
  `impact`. O modo ignora `closed`/`superseded`/`forgotten`, como o `next:` do `rewind`
  (D109/D106).
- `plan --prompt` deriva o prompt TOON do template (`feature`/`bug`/`refactor`,
  `.knudge/templates.toml`); `--submit --from -` lê o plano TOON e valida tudo **antes** de
  escrever (D105).
- `graph` projeta **papel** (`role`, D115) e **modo** (`mode`, D116) derivados da árvore/eventos:
  `role|kind|status|owner|mode|statement`.
- **Programa externo** (D119): `plan/<slug>.md` ancorado a um Épico-raiz (`--anchors`);
  `task graph --program` imprime a subárvore; `programs.glob` define o que é um programa.

## 8. `kd knowledge` — mapa de conhecimento (D128)

```
kd knowledge map [--axis <anchor|type|classification|container>] [--scope <CONTAINER>]
                [--semantic] [--members]
```

- **Fase 1** é determinística e sem embeddings: agrega por `anchor`, `type`, `classification` e
  `container` (o container ancestral sobe pela **hierarquia** `results_in`, com fallback para
  `depends_on`).
- **`--semantic`** roda a fase 2 (`complete-link`) dentro de cada cluster acima de
  `clusters.min_volume`, com `clusters.similarity_threshold` — off-path, read-only (D47).
- `--scope` restringe aos membros de um container; `--members` lista os membros.
- Pipe: `<axis>|<key>|<count>` (container acrescenta `|<título>`); com `--members`, membros
  indentados `id|statement`. Fase 2: `semantic|<axis>|<key>|groups=N` + grupos.

## 9. `kd maintenance` — manutenção

```
kd maintenance doctor [--fix] [--audit]   # relatório por padrão; --fix corrige o reversível
kd maintenance compact [--scope <C>]      # propõe merge/supersede (nunca em silêncio)
kd maintenance eval --ab <A> <B>          # Recall@k / nDCG@k / MRR
kd maintenance index [--drain|--status]   # fila de embeddings
kd maintenance learn [--scope <C>]        # sugestões de notas/links/merges
kd maintenance prune [--scope <C>]        # propõe forget por shelf-life/decay (nunca age, D112)
kd maintenance watch-service [--install|--subscribe|--unsubscribe|--status|--uninstall]
                             [--yes] [--dry-run] [--every 1h] [--port 8999]
                                          # gerencia o worker e o servidor de embeddings (systemd/launchd)
```

- `audit` virou modo do `doctor` (relatório de integridade + arestas sugeridas).
- `link` **não** mora aqui: virou `kd write --link`.
- `index` é, por padrão, interno (worker); `--status`/`--drain` são diagnóstico. Com
  `embeddings.mode=lazy` (default) o CLI ainda drena **um lote** ao fim de qualquer verbo
  não-`maintenance` (auto-drain ocioso, D131); `manual` desliga esse caminho.
- `prune` **só propõe** (`forget|id|motivo`); a aplicação é `kd forget` (D47/D112).
- `watch-service` gerencia o worker **sem supply-chain**: o `knudge-idle.sh` é embutido no binário
  (`--script`/`--url` sobrescrevem). Ações (exclusivas; default `--status`): `--install` faz
  pré-flight (`kd`/`llama`/GGUF/projeto), instala o agendador — `systemd --user` (Linux) ou
  `launchd` (macOS) —, sobe o **servidor de embeddings persistente** (`knudge-embed`) e cadastra o
  projeto atual; `--subscribe`/`--unsubscribe` cadastram/descadastram **um** projeto
  (multi-projeto; não desinstalam o sistema); `--status` mostra agendador/servidor/fila por
  projeto; `--uninstall` remove agendador + servidor. Sem systemd/launchd, o `--install` recusa e
  imprime a linha de cron. Mutar exige confirmação (stderr; não-TTY cancela; `--yes` pula). O GGUF
  mora ao lado do `config.toml` global (D132/D133).

## 10. `learn` em profundidade

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

## 11. `kd config`, `kd forget`, `kd sync`, `kd init`, `kd self`

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

## 12. `strict` como config (D94)

`strict` **não é flag nem subcomando**: é chave de projeto em `.knudge/config.toml`:

```toml
[behavior]
strict = false   # true promove warnings (leitura, retrieval, embeddings) a erro
```

## 13. Mapa antigo → novo

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
| `compact` / `doctor --audit` / `eval` / `index` / `learn` / `prune` | `kd maintenance …` |
| `onboard` | `kd init` |
| `setup` / `completions` / `upgrade` / `version` | `kd self …` |

## 14. MCP espelhado

As tools MCP usam os mesmos nomes e modos. Os 3 gatilhos (E12-T03) passam a apontar para:
pré-`write` (quase-duplicados), pré-edição (`kd rewind --files` contínuo), fim de sessão
(`kd maintenance learn`).

## 15. Aceite

- [x] `kd` sem argumentos == `kd prime`; `prime` byte-idêntico por versão (teste de golden).
- [x] `ask` cobre recall/get/expand com um só envelope.
- [x] `write` rejeita `task`/`container`; `--link` cobre arestas.
- [x] `task` valida hierarquia fechada (profundidade ≤ 4, pai único, sem ciclo).
- [x] `strict` lido só do config; nenhuma flag `--strict` existe.
- [x] `learn` só propõe; nenhum write sem aceite.

Detalhe por verbo (pipe/`--json`/erro/exit/estado) em
[`17_matriz_aceitacao.md`](17_matriz_aceitacao.md).
