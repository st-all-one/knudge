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
5. **Tudo de tarefa vive em `kd task`** (D93); `kd write` rejeita `--type task`.

## 2. Verbos de topo

| Verbo | Absorve | Papel |
|---|---|---|
| `kd` (sem args) | = `kd help` (D171) | help completo, exit 0 |
| `kd init` | `onboard` | funda `.knudge/` + prompt inicial |
| `kd prime` | — | protocolo de uso, byte-idêntico |
| `kd rewind` | `prime(scope)`, `get_context`, `diff` | estado/handoff ponto-no-tempo |
| `kd ask` | `recall`, `get`, `expand` | toda pesquisa |
| `kd write` | `write`, `update`, `link` | toda escrita |
| `kd task` | `epic`, grupos de tarefa | epic/issue/task |
| `kd knowledge` | `clusters` | mapa/ranking/vocabulário de conhecimento (D128/D146) |
| `kd drain` | — | fila de embeddings: status/digestão (D170) |
| `kd doctor` | — | saúde do corpus: 13 checks + auditoria (`--fix`, `--explain`) — D163 |
| `kd maintenance` | `compact`, `learn`, `prune`, `watch-service` | manutenção |
| `kd config` | `config` | `.knudge/config.toml` |
| `kd forget` | `forget`, `restore` | soft-delete |
| `kd sync` | `sync` | commit git |
| `kd self` | `setup`, `completions`, `upgrade`, `version` | instalação |

## 3. `kd ask` — toda pesquisa

Default: **recall completo** (filtros → BM25 → âncoras → RRF). Pipe `id|statement|score|why`.

```
kd ask [QUERY|-]
  --params '<JSON>'       # query + filtros (D147); `-` lê o objeto do stdin
  --id <ID>...            # get: corpo só dos ids pedidos
  --around <ID>           # expand no grafo explícito
  --via <ARESTA>          # tipo de aresta (default: todas)
  --depth <N>             # profundidade do expand (default 1)
  --brief                 # saída mínima: id|statement
  --full-content          # inclui o corpo completo dos hits (ex-`--with-body`, D146)
  --with-task             # inclui itens de trabalho (notas com `scope`); default = só conhecimento (D146)
  --type <T>...           # filtros determinísticos
  --class <C>...          # foundational|tactical|observational
  --tag <T>...
  --status <S>...
  --scope <ID>
  --anchor <PATH>...      # repetível; aceita vírgula (`--anchor a,b`)
  --since <TS> / --until <TS>
  --limit <N>             # default: config recall.default_limit (5 — D121)
  --json
```

Ranking (`--rank`) e vocabulário de tags (`--tags`) saíram do `ask` e viraram
`kd knowledge rank`/`tags` (D146).

O canal **vetorial** entra automaticamente quando `recall.semantic = true` (default) e há índice
(`recall.semantic_top_k`); provedor fora do ar degrada para BM25 com `warnings` (D102). Um hit
que veio pelo vetor aparece com `why = semantic` (D121); o canal lexical descarta stopwords e
fragmentos de 1 char (`content_terms`, D122).

O `--json` traz, por hit, `channels: {lexical, anchor, semantic, recent, stars}` (D151): as três
primeiras são as **parcelas RRF** (somam o `score`); `recent`/`stars` são boosts informativos em
`[0,1]`. O pipe `id|statement|score|why` **não** muda. Busca sem hit → stdout `[no_results]`
(literal fixo, exit 0; `--json` com `hits: []`) — D152.

Sem nenhum modo (query e âncora vazias, sem `--id`/`--around`), o `ask` devolve
o **uso** do comando com exit 2 em vez de sair vazio (D130). `ask -` (ou pipe sem posicional) lê a
query do stdin (D147). `--id` de nota ausente degrada para `warnings`; `--around` de nota ausente
é `not_found` (3).

O **feedback tarefa→conhecimento** (X1/D108) é derivado em tempo de consulta: tarefas com
`outcomes` de sucesso que compartilham `anchors` confirmam a nota — o peso
`recall.confirmation_from_tasks` (float, default `0.1`) entra no boost do BM25 e em
`hits[].confidence`, e o manifest de `rewind` promove a `star`. Sem `write` (D87).

## 4. `kd write` — toda escrita

Create idempotente por conteúdo + protocolo de dedup (0.75/0.92). `--update` versiona;
`--link` cria aresta explícita. **Rejeita `--type task`** (use `kd task`) e
`statement` vazio é `invalid_input` (2) — nunca cria nota vazia (D130).

```
kd write --summary <TXT> [<BODY>|-]
  --type <T>              # default: fact
  --tag <T>... --anchor <PATH>...
  --class <C> --status <S>
  --edge <ARESTA:ID>      # aresta explícita na criação
  --update <ID>           # modo update (patch versionado); aceita [--params '<JSON>']
  --clear-anchors         # com --update, limpa as âncoras (não combina com --anchor)
  --link <ARESTA:ID>      # cria aresta (substitui o antigo `link`)
  --outcome <S> --id <ID> # anexa evidência a qualquer nota (D103); com [--note <TXT>]
  --params '<JSON>'       # item único (D147)
  --batch <FONTE|->       # lote JSONL de rascunhos (D110); com [--dry-run]
  --dry-run
  --json                  # {action: created|merged|rejected|updated, id}
```

O **posicional é o corpo** (a verdade do knudge); a afirmação vai em `--summary` (D140). O corpo
aceita `-` (stdin) e pipe/heredoc sem posicional (`cat body.md | kd write --summary S`).
`--anchor` é a **única** grafia (o alias `--anchors` foi removido — D168); o canônico do corpo
completo é `--full-content`.

## 5. `kd prime` — protocolo estático

**Sempre a mesma resposta** para uma dada versão do binário (cacheável, byte-idêntico):
tipos, tools, regras, orçamento, formato de saída. O default é **compacto** (D166);
`--long` inclui a gramática TOON e o schema completo. `kd` sozinho mostra o **help** (D171);
`prime` é sempre explícito.
O corpo é organizado por **fluxo** — `CICLO` (ask→write→task→sync), `CONHECIMENTO`,
`PESQUISA`, `TAREFAS` — e recomenda `--limit`/`--brief` para economizar contexto (D130).

```
kd prime [--long]         # default compacto; --long inclui a gramática TOON e o schema completo
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
  --tag <T>... / --anchor <PATH>... / --type <T>... / --class <C>...  # escopo de conhecimento (D143)
  --around <ID> [--depth <N>]                                          # vizinhança de uma nota
  --json
```

O manifest (default) ganha `next:` (tarefas `ready` abertas por impacto) e `fresh:`
(`stale`/`expiring`/`pending`) — D106. `K` deriva do orçamento; o excedente vira `dropped`.
Os filtros de corpus restringem o que entra no handoff (D143).

## 7. `kd task` — epic/issue/task (D93)

Hierarquia **fechada**: `epic ⊃ { issue ⊃ task | task }` — épico é a raiz, issue opcional. Campo `scope`
(enum fechado) marca o **nível**; o `type` é a **espécie** (D113): `epic` (grupo derivado de `scope=epic`) para epic,
`task`/`error`/`question`/`risk`/`decision` para itens de trabalho. Pai único via
membership/backref (D52); dependências via aresta `depends_on`, criada **só** com
`kd write --link <FROM:depends_on:TO>` (via única, D126).

```
kd task new --summary <TXT> [<BODY>|-] --scope <epic|issue|task>
  [--kind <task|error|question|risk|decision>] [--parent <ID>]
  [--checks <NAME>...] [--anchor <PATH>...] [--tag <T>...]
  [--params '<JSON>'|--batch <FONTE|->] [--dry-run]
kd task list <filtro> [--sort impact] [--full-content]
  # filtro: --scope (nível epic|issue|task OU id do container)/--status/--kind/--parent/--ready/--blocked/--tag/--anchor, ou --universe (D144)
  # `--sort`/`--explain` não contam como escopo
  [--ready|--blocked [--explain]] [--sort impact] [--tag <T>...] [--anchor <PATH>...]
  [--full-content]
kd task show --id <ID> [<ID>...] [--history]   # + corpo/checks/âncoras/tags/outcomes (D137)
kd task graph [--program <PATH>|--root <ID>]   # escopos com progresso (D127)
kd task update --id <ID> [--statement <S>] [--status <S>] [--parent <ID>] [--checks ...]
  [--anchor <PATH>...] [--clear-anchors]       # substitui/limpa âncoras
kd task close --id <ID> [--outcome success|partial|failure|abandoned] [--note <TXT>]
kd task plan <ID> [--prompt [--template <NOME>] | --submit --from <TXT|->]
  [--step <TXT>...]
```

- `new` tem o **posicional como corpo** e a afirmação em `--summary` (D140); `--params` cria um
  item por objeto JSON e `--batch` aplica um lote JSONL com `key`/`id`/`parent`/arestas,
  best-effort com `--dry-run` (D141).

- `close` roda os validators e grava `outcomes[]`/`evidence` (D48/D55) — nunca declara sem
  evidência; acrescenta o **épico mais próximo** e o **progresso** dele (D127).
- `show` resolve o **contexto** de cada id — `parent`, `blocked_by`, `blocks`, `children` e o
  **épico com progresso** — com **título** e estado, para responder "onde isto se encaixa e o
  que o bloqueia" num só comando (D125/D127); no `--json`, os campos vêm estruturados. O texto
  também traz `scope`/`tipo`/`status`, o **corpo**, `checks`/`ancoras`/`tags`/`outcomes` —
  paridade com o `--json` (D137).
- `list --full-content` renderiza cada item como esse **bloco completo**, separado por `\n---\n`,
  compondo com todos os filtros; o `--json` traz os mesmos objetos do `show`. Não é pipe-safe
  (multilinha); o pipe enxuto `id|scope|status|statement` segue como default (D137).
- **Rollup de progresso por épico (D127):** `epic_of`/`progress_of` contam os **itens de trabalho
  folha** (`is_work_item` sem filhos de trabalho) no subárvore e quantos estão `closed` —
  derivado, sem verdade nova. Aparece no `close` (`epico: <id>|<título> (<done>/<total>)`), no
  `show` e nos containers do `task graph` (`(done/total)`).
- **Arestas (inclui `depends_on`) têm uma via única:** `kd write --link <FROM:ARESTA:TO>` (D126).
  `kd task new` não cria arestas — reduz a superfície e reaproveita o caminho de grafo.
- `plan` só tem `--prompt`/`--submit` (D138); a ordem entre irmãos vem do `PlanStep.blocks` do
  plano submetido, e fechar é só `kd task close` (com evidência, D55).
- `epic` é uma **view derivada** (sem verdade própria); `issue`/`task` são atômicas.
- `--kind` grava a **espécie** mantendo o `scope` (D113).
- `list --ready|--blocked` filtra pelas views derivadas; `--explain` acrescenta o motivo (D104).
- `list --sort impact` ordena o **caminho crítico** por impacto de desbloqueio
  (`impacto desc, created asc, id asc`); `--explain` acrescenta `unblocks=N` e o `--json` traz
  `impact`. O modo ignora `closed`/`superseded`/`forgotten`, como o `next:` do `rewind`
  (D109/D106).
- `plan --prompt` deriva o prompt TOON do template (`feature`/`bug`/`refactor`,
  `.knudge/templates.toml`); `--submit --from -` lê o plano TOON e valida tudo **antes** de
  escrever (D105).
- `graph` projeta no `--json` o **papel** (`role`, D115) e o **modo** (`mode`, D116/D136); o
  **texto** é enxuto — `id|kind|status|statement (done/total)` (D139).
- **Programa externo** (D119): `plan/<slug>.md` pode ancorar **vários** Épicos-raiz (`--anchors`);
  `task graph --program` imprime a **floresta** (ordem de `id`); `--root <ID>` rende uma árvore só;
  `programs.glob` define o que é um programa (D139).

## 8. `kd knowledge` — mapa/ranking/vocabulário de conhecimento (D128/D146)

```
kd knowledge map [--axis <anchor|type|classification|scope>] [--scope <ESCOPO>]
                [--semantic] [--members] [--write]
                [--tag <T>...] [--anchor <PATH>...] [--type <T>...] [--class <C>...]
                [--around <ID>] [--depth <N>] [--universe]
kd knowledge rank [--tag ...] [--anchor ...] [--type ...] [--class ...]
                  [--around <ID>] [--depth <N>] [--universe] [--limit <N>]
kd knowledge tags [--limit <N>]
```

- **Escopo obrigatório (D143):** `map`/`rank` sem filtro e sem `--universe` são `invalid_input`
  (2). `--universe` é a varredura explícita do projeto inteiro; `--around <ID> --depth N` limita à
  vizinhança de uma nota.
- **Fase 1** é determinística e sem embeddings: agrega por `anchor`, `type`, `classification` e
  `scope` (o escopo ancestral sobe pela **hierarquia** `results_in`, com fallback para `depends_on`).
- **`--semantic`** roda a fase 2 (`complete-link`) dentro de cada cluster acima de
  `clusters.min_volume`, com `clusters.similarity_threshold` — off-path, read-only (D47).
- `--scope` restringe aos membros de um escopo; `--members` lista os membros.
- **`--write`** materializa `notas/MAP.md` + uma nota-hub (`meta` + `references`) por cluster
  (D150) — versionado, buscável pelo `ask`.
- Pipe: `<axis>|<key>|<count>` (container acrescenta `|<título>`); com `--members`, membros
  indentados `id|statement`. Fase 2: `semantic|<axis>|<key>|groups=N` + grupos.

## 9. `kd doctor` e `kd maintenance` — saúde e manutenção

```
kd doctor [--fix] [--explain]             # 13 checks + auditoria; --fix corrige o reversível
                                          # --explain detalha cada achado (esperado/encontrado/ação)
kd maintenance compact [--scope <C>] [--tag ...|--anchor ...|--type ...|--class ...|--around ...|--universe]
                                          # propõe merge/supersede (nunca em silêncio; escopo obrigatório — D144)
kd maintenance learn [--scope <C>] [--tag ...|--anchor ...|--type ...|--class ...|--around ...|--universe]
                                          # sugestões de notas/links/merges (escopo obrigatório — D144)
kd maintenance prune [--scope <C>] [--tag ...|--anchor ...|--type ...|--class ...|--around ...|--universe]
                                          # propõe forget por shelf-life/decay (nunca age, D112)
kd maintenance watch-service [--install|--subscribe|--unsubscribe|--status|--uninstall]
                             [--yes] [--dry-run] [--every 1h] [--port 8999]
                                          # gerencia o worker e o servidor de embeddings (systemd/launchd)
kd drain [--status | --digest [--force]]   # fila de embeddings: estado rico e digestão (D170)
```

- `audit` virou modo do `doctor` (relatório de integridade + arestas sugeridas).
- `link` **não** mora aqui: virou `kd write --link`.
- `index` é, por padrão, interno (worker); `kd drain --status`/`--digest` é o diagnóstico e a
  digestão manual. Com `embeddings.mode=lazy` (default) o CLI ainda drena **um lote** ao fim de
  qualquer verbo não-`maintenance`/`doctor`/`drain` (auto-drain ocioso, D131); `manual` desliga.
- `prune` **só propõe** (`forget|id|motivo`); a aplicação é `kd forget` (D47/D112).
- **Verbosidade (D165):** `compact`/`learn`/`prune` terminam com `propostas: <kind>=N` e
  `próximos:` (aplicar via `kd write`/`kd forget`); `watch-service` termina com `próximos:`.
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
kd config get --key <CHAVE>
kd config set --key <CHAVE> --value <VALOR> [--global]   # projeto por padrão; grava .knudge/config.toml
kd config unset --key <CHAVE> [--global]
kd config list [--global]

kd forget --id <ID>             # status=forgotten (soft; nunca apaga arquivo)
kd forget --id <ID> --restore   # volta a active
kd forget --id <ID> --purge [--force]  # hard-delete só após a janela de retenção; --force libera tombstone

kd sync [--message <MSG>]  # commit de notas/ + eventos/ no worktree certo

kd init [--force] [--no-prompt]   # estrutura canônica + prompt inicial de fundação

kd self setup <claude|cursor|codex|pi>
kd self completions <shell>
kd self upgrade
kd self version
```

- **Verbosidade (D165):** `init` lista o que criou/alterou (`novo`/`atualizado`/`inalterado`) e
  `próximos:`; `config` mostra `(projeto|global) — <path do arquivo>`; `sync` mostra
  `<branch>: N arquivo(s) commitados (<hash curto>)` ou `nada a sincronizar`; `self version`
  aponta o help; `self setup` lista o próximo passo; `self completions` mantém o **script puro**
  no stdout (resumo só em stderr).

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
| `doctor` / `doctor --audit` | `kd doctor` (auditoria é o padrão) |
| `compact` / `learn` / `prune` | `kd maintenance …` |
| `eval` / `index` | **removidos** — avaliação offline (`bench/`); fila é `kd drain` (D170) |
| `onboard` | `kd init` |
| `setup` / `completions` / `upgrade` / `version` | `kd self …` |

## 14. MCP espelhado

As tools MCP usam os mesmos nomes e modos. Os 3 gatilhos (E12-T03) passam a apontar para:
pré-`write` (quase-duplicados), pré-edição (`kd rewind --files` contínuo), fim de sessão
(`kd maintenance learn`).

## 15. Aceite

- [x] `kd` sem argumentos == `kd help`; `prime` byte-idêntico por versão (teste de golden).
- [x] `ask` cobre recall/get/expand com um só envelope.
- [x] `write` rejeita `task`/`container`; `--link` cobre arestas.
- [x] `task` valida hierarquia fechada (profundidade ≤ 4, pai único, sem ciclo).
- [x] `strict` lido só do config; nenhuma flag `--strict` existe.
- [x] `learn` só propõe; nenhum write sem aceite.

Detalhe por verbo (pipe/`--json`/erro/exit/estado) em
[`17_matriz_aceitacao.md`](17_matriz_aceitacao.md).

## 16. Delta v0.3.2 (D154–D159)

Adições que **não** mudam os verbos existentes (só flags/subcomandos) e nenhuma chave TOON:

| Superfície | Forma | Papel |
|---|---|---|
| `kd ask --as-of <TS>` | flag de `ask` | corpus ativo em `T` (D155); `--json` ganha `as_of`/`historical` |
| `kd knowledge suggest [--top-k N] [--relation R] [--limit N]` | subcomando | sugestões semânticas `duplicate`/`contradiction`/`link` (D158) |
| `kd knowledge promote recommend\|approve\|edit\|remove\|list` | subcomando + sub-subcomandos | regras governadas no `AGENTS.md` (D157) |
| `kd maintenance learn --verify` / `compact --verify` | flag | anexa o veredito do portão (read-only, D156) |
| `retention.renew_on_use` | config | renovação de shelf-life por uso (D154) |
| `proposals.gate` / `min_delta` / `enforce` | config | portão de evidência (D156) |
| `suggestions.enabled` / `contradiction_low` / `contradiction_high` | config | banda semântica (D158) |
| `rules.enabled` / `max_promoted` / `min_confidence` | config | promoção de regras (D157) |

`validators.toml` ganha o campo opcional `kind = "check"|"gate"` (default `check`); `kind="gate"`
usa stdin `{op,before,after}` → stdout `{passed,score_before,score_after}` (D156).

### Delta D160–D162

| Superfície | Forma | Papel |
|---|---|---|
| `kd ask` (padrão) | comportamento | revelação progressiva do corpo: 1º hit completo, 2–5 truncado, resto padrão (D161) |
| `kd ask --json` | contrato | por hit: `body_match`/`body_snippet` (D161) |
| `recall.preview_chars` | config | limite de caracteres dos hits parciais (default 280, D161) |
| `kd init` / `onboard` | artefato | cria `.agents/skill/kd/SKILL.md` idempotente (D162) |
| `kd prime` | texto | seção **CORPO** com template (D162) |
| sessão (`run_session`) | comportamento | varredura de resíduos `*.tmp`/`*.stale` no início (D160) |

Nenhum verbo novo; nenhuma chave TOON nova.
