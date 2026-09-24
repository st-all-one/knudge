# `kd task` — planejar e executar trabalho

## O que faz

Gerencia o **trabalho** do projeto numa hierarquia fechada: **`epic ⊃ { issue ⊃ task | task }`**.
O `epic` é a raiz e pode viver sozinho; o `issue` é opcional; a folha é uma `task`. A espécie
(`type`) pode variar (`task`, `error`, `question`, `risk`, `decision`) sem mudar o nível.

Subcomandos: `new`, `list`, `show`, `update`, `close`, `graph`, `plan`.

## Em 30 segundos

```bash
kd task new --summary "Sync offline-first" --scope epic
kd task new --summary "Resolver conflito de merge" --scope task --parent <epic>
kd task list --ready --sort impact
kd task close --id <task> --outcome success --note "testes verdes"
```

## Uso

```
kd task new --summary <TXT> [<corpo>|-] --scope <epic|issue|task> [--kind K] [--parent ID] ...
kd task list <filtro> [--sort impact] [--full-content]
kd task show --id <ID>... [--history]
kd task update --id <ID> [--statement TXT] [--status S] [--parent ID] [--checks NOME]...
kd task close --id <ID> [--outcome S] [--note TXT]
kd task graph [--program <PATH> | --root <ID>]
kd task plan <ID> --prompt [--template T] | --submit [--step TXT] [--from FONTE]
```

## `task new` — criar

### Nível 1 — o épico e a tarefa

```bash
kd task new --summary "Migração para o schema V2" --scope epic --anchor plan/v2.md
kd task new --summary "Converter ids históricos" --scope task --parent epic_01abc
```

Ancorar o épico em `plan/*.md` é **fortemente recomendado**: o `doctor` avisa se não houver
programa, e o `task graph --program` usa essa âncora.

### Nível 2 — issue opcional e espécie

```bash
kd task new --summary "Story do cache" --scope issue --parent epic_01abc
kd task new --summary "Corrigir off-by-one" --scope task --kind error --parent issue_01def
kd task new --summary "Risco de timeout" --scope task --kind risk
```

`--kind` aceita `task|error|question|risk|decision` (default `task`).

### Nível 3 — corpo, checks, tags

```bash
kd task new --summary "Escrever testes do parser" --scope task \
  --checks test --checks lint --tag parser --anchor src/toon/parse.rs <<'EOF'
Cobrir: whitespace Unicode, aspas, listas aninhadas.
EOF
```

### Nível 4 — lote e objeto (D141/D147)

```bash
kd task new --params '{"statement":"Via objeto","scope":"task"}'

kd task new --batch - --dry-run <<'EOF'
{"key":"p","statement":"Pai em lote","scope":"issue"}
{"key":"c","statement":"Filho em lote","scope":"task","parent":"p"}
{"statement":"Depende do filho","scope":"task","depends_on":["c"]}
EOF
```

O lote usa chaves canônicas (`statement`, `body`, `scope`, `kind`, `parent`, `checks`, `anchors`,
`tags`, `classification`, `status`, `blocks`) + `key` local para resolver `parent`/`depends_on`
dentro do próprio lote. Processa em ordem (pai antes do filho), é **best-effort** com `warnings[]`,
e tem teto `task.batch_max` (default 100).

## `task list` — listar

`task list` **exige um filtro** ou `--universe` (D144). `--sort`/`--explain` **não** contam como
escopo.

```bash
kd task list --ready                      # dependências resolvidas
kd task list --blocked --explain          # + motivo (blocked_by=<id>/cycle)
kd task list --ready --sort impact        # + unblocks=N
kd task list --tag parser --anchor src/toon/parse.rs
kd task list --ready --full-content       # bloco completo (multilinha)
kd task list --universe                   # panorama geral explícito
```

A linha padrão é `id|scope|status|statement`. `--full-content` renderiza cada item como o bloco do
`show` (corpo/`checks`/âncoras/tags/`outcomes`), separado por `\n---\n` — **não é pipe-safe**.

> `--ready` lista o que **não está bloqueado por dependência** (a view do grafo); itens fechados
> aparecem. Para só o acionável, combine com `--sort impact` (que ignora fechados) ou `--status`.

## `task show` — ver

```bash
kd task show --id task_01abc
kd task show --id task_01abc task_01def --history
```

Mostra o bloco completo: `id|statement`, `scope`, `tipo`, `status`, `corpo:`, `checks:`,
`ancoras:`, `tags:`, `outcomes:`, `context:` (pai/bloqueado_por/bloqueia/filhos/épico/progresso),
`historico:`. Ids ausentes viram `warnings[]` (parcial).

## `task update` — editar

```bash
kd task update --id task_01abc --status in_progress
kd task update --id task_01abc --statement "Novo texto" --parent issue_01def
kd task update --id task_01abc --checks test --checks lint
```

Edita campos **no lugar** (incrementa `revision`). Transições de status ficam aqui.

## `task close` — fechar com evidência

```bash
kd task close --id task_01abc --outcome success --note "testes verdes"
```

Roda os **validators** (`checks`) e grava a evidência em `outcomes`. Fechar **exige** evidência
(`--outcome`); `--note` explica o porquê. Quando a folha pertence a um épico, a saída acrescenta
uma linha `epico: <id>|<título> (<done>/<total>)`.

## `task graph` — ver a árvore

```bash
kd task graph --root epic_01abc            # árvore de um épico
kd task graph --program plan/v2.md         # floresta de todos os épicos ancorados ao programa
```

Saída enxuta: `id|kind|status|statement (done/total)`, indentada. `--program` aceita **vários**
épicos-raiz ancorados ao mesmo `plan/*.md` (floresta).

## `task plan` — gerar/submeter plano

```bash
kd task plan epic_01abc --prompt --template feature     # prompt read-only (TOON)
kd task plan epic_01abc --submit --from plano.toon       # cria os filhos
kd task plan epic_01abc --submit --from -                # lê o plano do stdin
```

`--prompt` é read-only e imprime o template (`feature`/`bug`/`refactor`); `--submit` valida tudo
antes de gravar e cria N filhos. Nada é aplicado parcialmente em caso de erro.

## Resultados

- **`task new`** — `data.id` (ou `data.items[]` no lote); texto `key|id|scope|status|statement`.
- **`task list`** — pipe `id|scope|status|statement`; `data.tasks[]`.
- **`task close`** — `data.outcome`, `data.evidence[]`, `data.epic{done,total}`.
- **`task graph`** — `data.program`, `data.roots[]`, `data.nodes[]`.
- **Exit codes** — `2` sem escopo em `list` ou `scope` inválido; `3` raiz/programa ausente;
  `4` colisão de id no `--submit`; `8` `--kind` incoerente.

## Quando (não) usar

- **Use** para trabalho (aberto, em progresso, fechado com evidência).
- **Não use** para conhecimento — `kd write --type task` é rejeitado; use `kd write`.
- **Não use** posse/atribuição: o knudge assume **um agente principal**; "estou fazendo isto" é
  `status=in_progress` (D136).

## Próximo passo

➡️ [`kd knowledge`](07-knowledge.md) · [`kd rewind`](08-rewind.md)
