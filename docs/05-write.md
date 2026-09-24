# `kd write` — toda a escrita

## O que faz

É o comando de **escrita** de conhecimento: cria notas, versiona (`--update`), cria arestas
(`--link`), anexa evidência (`--outcome`) e aceita lotes (`--batch`/`--params`). Trabalho
(tarefas) **não** entra aqui — use [`kd task`](06-task.md).

O posicional é o **corpo** (Markdown); a afirmação é `--summary`. O `write` faz **dedup** contra o
que já existe e respeita os limiares configurados.

## Em 30 segundos

```bash
kd write --summary "Rate limit é 100 rps por chave" --type fact
```

Corpo via heredoc:

```bash
kd write --summary "Cache usa LRU" --type decision <<'EOF'
Motivo: LRU tem custo O(1) e boa taxa de acerto no padrão de acesso.
EOF
```

## Uso

```
kd write [BODY]... [--summary <TXT>] [--type <TIPO>] [--tag <T>]... [--anchor <PATH>]...
         [--class <CLASSE>] [--status <STATUS>] [--edge <ARESTA:ID>]
kd write --update <ID> [--summary <TXT>] [...] [--params <JSON>] [--clear-anchors]
kd write --link <FROM:ARESTA:TO>
kd write --outcome <OUTCOME> --id <ID> [--note <TXT>]
kd write --batch <FONTE|-> [--dry-run]
kd write --params '<json>' [--dry-run]
```

## Exemplos

### Nível 1 — criar

```bash
kd write --summary "O gateway faz retry exponencial" --type fact \
  --tag gateway --anchor src/gateway.rs
```

Saída (texto): `created|fact_01qejflt|r1` (ação `created`/`merged`/`rejected`/`unchanged`,
id e revisão).

### Nível 2 — dedup

Antes de criar, o `write` roda o `ask` internamente:

| Score | Ação |
|---|---|
| `< 0.75` | `created` — nota nova |
| `0.75 – 0.92` | `merged` — incorpora à existente |
| `≥ 0.92` | `rejected` — duplicata |

Se quiser revisar antes, rode `kd ask "<rascunho>"` você mesmo.

### Nível 3 — versionar

```bash
kd write --update fact_01qejflt --summary "O gateway faz retry exponencial com jitter"
```

Mudar `type` ou `statement` **cria novo id e supersede** o antigo (não reescreve o id). O
`revision` incrementa a cada update.

**Id legado/não derivável** (prefixo histórico, ex.: `container_*`) **revisa no lugar** quando só
corpo/tags/âncoras mudam; o id só é reescrito (supersede) se a chave de conteúdo (`type` +
`statement`) mudar.

Editar campos sem `--summary` é mais fácil com `--params` — o mesmo objeto do lote, aplicado como
patch:

```bash
kd write --update fact_01qejflt --params '{"body":"novo corpo","tags":["cache"]}'
kd write --update fact_01qejflt --params '{"anchors":[]}'   # limpa as âncoras
kd write --update fact_01qejflt --clear-anchors              # atalho para limpar
```

Âncora vazia (`--anchor ""`) é **rejeitada** (exit 2); para limpar use `--clear-anchors`.

### Nível 4 — relacionar

```bash
kd write --link "decision_01abc:extends:fact_01xyz"
kd write --link "task_01def:depends_on:task_01ghi"
```

A aresta é a **única** via de vínculo explícito (D126) e usa o vocabulário fechado de 8 arestas:
`references`, `depends_on`, `contradicts`, `supports`, `extends`, `replaces`, `rejects`,
`results_in`.

### Nível 5 — evidência

```bash
kd write --outcome success --id task_01def --note "testes verdes em CI"
kd write --outcome failure --id fact_01xyz --note "medição refutou a hipótese"
```

Evidência confirma (`success`) ou enfraquece a nota e entra no cálculo da **confiança derivada**
(vista em [`kd knowledge rank`](07-knowledge.md)).

### Nível 6 — lote e objeto

```bash
# Um item (atalho do lote)
kd write --params '{"statement":"Cache expira em 30 dias","type":"fact","tags":["cache"]}'

# Vários itens por JSONL (uma linha por rascunho)
kd write --batch - --dry-run <<'EOF'
{"statement":"Nota A","type":"fact"}
{"statement":"Nota B","type":"decision","tags":["x"]}
EOF
```

O schema do lote é o **canônico** (`statement`, `body`, `type`, `tags`, `anchors`,
`classification`, `status`) — não os nomes das flags. `--dry-run` só avalia. O teto é
`write.batch_max` (default 100).

## Referência de flags

| Flag | Efeito |
|---|---|
| `[BODY]...` | Corpo Markdown (posicional; `-`/pipe lê stdin) |
| `--summary <TXT>` | Afirmação (chave TOON `statement`) |
| `--type <TIPO>` | Espécie (default `fact`; `task` é rejeitado) |
| `--tag <T>` | Tag (repetível) |
| `--anchor <PATH>` | Âncora (repetível; aceita vírgula; alias `--anchors`) |
| `--clear-anchors` | Com `--update`, limpa todas as âncoras (conflita com `--anchor`) |
| `--class <CLASSE>` | `foundational`/`tactical`/`observational` |
| `--status <STATUS>` | Status inicial |
| `--edge <ARESTA:ID>` | Aresta a partir da nota criada |
| `--update <ID>` | Versiona a nota existente |
| `--id <ID>` | Id alvo (usado com `--outcome`) |
| `--link <FROM:ARESTA:TO>` | Cria aresta entre notas existentes |
| `--outcome <OUTCOME>` | Anexa evidência (`success`/`partial`/`failure`/`abandoned`) |
| `--note <TXT>` | Texto da evidência |
| `--batch <FONTE>` | Lote JSONL (`-` lê stdin) |
| `--params <JSON>` | Objeto de um rascunho (`-` lê stdin) |
| `--dry-run` | Simula sem gravar |

## Resultados

- **Texto:** `acao|id|rN` (ex.: `created|fact_01abc|r1`).
- **`--json`:** `data.action`, `data.id`, `data.revision`; no lote, `data.items[]`.
- **`--type task`** é rejeitado (`invalid_input`, exit 2).
- Erros de schema (chave/tipo desconhecido) → exit 8; `--update` de id inexistente/`forgotten` →
  exit 4.

## Quando (não) usar

- **Use** para fatos, decisões, erros, riscos, perguntas, definições, snippets, links e metas.
- **Não use** para trabalho — `--type task` é rejeitado; use [`kd task new`](06-task.md).
- **Não use** para apagar — use [`kd forget`](11-forget.md).

## Próximo passo

➡️ [`kd task`](06-task.md) · [`kd knowledge`](07-knowledge.md)
