# 07 · `kd task` — planejar e executar trabalho

## Para que serve

Gerencia o **trabalho** do projeto numa árvore simples:

```
epic ⊃ { issue ⊃ task | task }
```

- **`epic`** é a raiz (pode viver sozinho); **`issue`** é opcional; **`task`** é a folha.
- Cada item tem uma **espécie** (`--kind`): `task` (default), `error`, `question`, `risk` ou
  `decision`.

Subcomandos: `new`, `list`, `show`, `update`, `close`, `graph`, `flow`, `plan`.

## Quando usar

- **Use** para qualquer coisa que precise **ser feita** (aberta, em progresso, fechada com
  evidência).
- **Não use** para conhecimento ([`kd write`](06_write.md)) nem para atribuir dono: o knudge assume
  **um agente principal** — "estou fazendo isto" é `--status in_progress`.

## Sintaxe

```
kd task new --summary <TXT> [<corpo>|-] --scope <epic|issue|task> [--kind K] [--parent ID] ...
kd task list <filtro> [--sort impact] [--full-content]
kd task show --id <ID>... [--history]
kd task update --id <ID> [--statement TXT] [--status S] [--parent ID] [--checks NOME]...
kd task close --id <ID> [--outcome S] [--note TXT]
kd task graph [--program <PATH> | --root <ID>]
kd task flow [--window-days N]
kd task plan <ID> --prompt [--template T] | --submit [--step TXT] [--from FONTE]
```

## Exemplos

### `task new` — criar

```bash
# 1. o épico (âncora em plan/*.md é fortemente recomendada)
kd task new --summary "Migração para o schema V2" --scope epic --anchor plan/v2.md

# 2. uma tarefa filha
kd task new --summary "Converter ids históricos" --scope task --parent epic_01abc

# 3. uma issue intermediária (opcional)
kd task new --summary "Story do cache" --scope issue --parent epic_01abc
```

Com espécie, corpo, checks e tags:

```bash
# 1. espécie não-trivial
kd task new --summary "Corrigir off-by-one" --scope task --kind error --parent issue_01def

# 2. checks de aceite + corpo
kd task new --summary "Escrever testes do parser" --scope task \
  --checks test --checks lint --tag parser --anchor src/toon/parse.rs <<'EOF'
Cobrir: whitespace Unicode, aspas, listas aninhadas.
EOF

# 3. via objeto/lote
kd task new --params '{"statement":"Via objeto","scope":"task"}'
```

### `task new --batch` — criar em lote

```bash
# 1. pai e filho no mesmo lote
kd task new --batch - --dry-run <<'EOF'
{"key":"p","statement":"Pai em lote","scope":"issue"}
{"key":"c","statement":"Filho em lote","scope":"task","parent":"p"}
{"statement":"Depende do filho","scope":"task","depends_on":["c"]}
EOF

# 2. aplicar de verdade
kd task new --batch plano.jsonl

# 3. atualizar em lote (com `id`, a linha vira update)
kd task new --batch updates.jsonl
```

O lote processa em ordem (pai antes do filho), é **best-effort** (avisa o que falhou) e tem teto
configurável. `--dry-run` só avalia.

### `task list` — listar

```bash
# 1. o que está pronto para começar
kd task list --ready

# 2. o que está bloqueado, com o motivo
kd task list --blocked --explain

# 3. priorizar por impacto (o que destrava mais)
kd task list --ready --sort impact
```

Outros recortes:

```bash
# 1. por nível
kd task list --scope epic

# 2. subárvore inteira de um épico
kd task list --scope epic_01abc

# 3. por tag/âncora
kd task list --tag parser --anchor src/toon/parse.rs
```

`task list` **exige um filtro** ou `--universe`. `--sort` e `--explain` não contam como filtro.

### `task show` — ver em detalhe

```bash
# 1. uma tarefa
kd task show --id task_01abc

# 2. várias de uma vez (vírgula ou repetindo)
kd task show --id task_01abc,task_01def
kd task show --id task_01abc --id task_01def

# 3. com histórico de substituição
kd task show --id task_01abc --history
```

Mostra corpo, checks, âncoras, tags, evidências e o **contexto** (pai, o que bloqueia, o que
destrava, filhos, épico e progresso).

### `task update` — editar

```bash
# 1. mudar status
kd task update --id task_01abc --status in_progress

# 2. renomear e re-parentar
kd task update --id task_01abc --statement "Novo texto" --parent issue_01def

# 3. trocar checks ou âncoras
kd task update --id task_01abc --checks test --checks lint
kd task update --id task_01abc --anchor src/cache.rs      # substitui o conjunto
kd task update --id task_01abc --clear-anchors            # limpa todas
```

### `task close` — fechar com evidência

```bash
# 1. sucesso
kd task close --id task_01abc --outcome success --note "testes verdes"

# 2. parcial
kd task close --id task_01abc --outcome partial --note "2 de 3 casos"

# 3. abandonado
kd task close --id task_01abc --outcome abandoned --note "requisito mudou"
```

Fechar **exige evidência**. Roda os `checks` configurados e grava o resultado. Quando a folha
pertence a um épico, a saída mostra o progresso (`done/total`).

### `task graph` — ver a árvore

```bash
# 1. de um épico
kd task graph --root epic_01abc

# 2. de um programa externo (plan/*.md)
kd task graph --program plan/v2.md

# 3. JSON para automação
kd --json task graph --root epic_01abc | jq '.data.nodes'
```

Saída indentada: `id|espécie|status|afirmação (done/total)`.

### `task flow` — fluxo e caminho crítico

```bash
# 1. resumo + throughput (janela padrão de 7 dias) + caminho crítico
kd task flow

# 2. janela de 1 dia
kd task flow --window-days 1

# 3. JSON
kd --json task flow | jq '.data.critical_path'
```

Mostra tempo de ciclo, tempo em voo e o caminho crítico das dependências (o que mais atrasa o
projeto). Nada é gravado.

### `task plan` — gerar e submeter plano

```bash
# 1. ver o prompt (read-only) de um template
kd task plan epic_01abc --prompt --template feature

# 2. submeter um plano de um arquivo
kd task plan epic_01abc --submit --from plano.toon

# 3. submeter lendo do stdin
kd task plan epic_01abc --submit --from -
```

`--prompt` imprime o que preencher (template `feature`/`bug`/`refactor`); `--submit` valida **tudo**
antes de gravar e cria os filhos de uma vez.

## Flags

### `new`

| Flag | Efeito |
|---|---|
| `[BODY]...` | Corpo (posicional; `-`/pipe lê stdin) |
| `--summary <TXT>` | Afirmação |
| `--scope <S>` | Nível: `epic`/`issue`/`task` |
| `--kind <K>` | Espécie: `task`/`error`/`question`/`risk`/`decision` (default `task`) |
| `--parent <ID>` | Pai na hierarquia |
| `--checks <NOME>` | Critério de aceite (repetível; aceita vírgula) |
| `--anchor <PATH>` | Âncora (repetível; aceita vírgula) |
| `--tag <T>` | Tag (repetível; aceita vírgula) |
| `--params <JSON>` | Uma tarefa por objeto (`-` lê stdin) |
| `--batch <FONTE>` | Lote JSONL (`-` lê stdin) |
| `--dry-run` | Só avalia o lote |

### `list`

| Flag | Efeito |
|---|---|
| `--scope <S>` | Nível ou id do épico (traz a subárvore) |
| `--status <S>` | Filtro por status: `active`/`in_progress`/`blocked`/`closed`/`superseded`/`forgotten` |
| `--kind <K>` | Filtro por espécie: `task`/`error`/`question`/`risk`/`decision` |
| `--parent <ID>` | Filtro por pai |
| `--ready` | Só o que não está bloqueado |
| `--blocked` | Só o bloqueado |
| `--explain` | Acrescenta o motivo / `unblocks=N` |
| `--sort impact` | Ordena por impacto de desbloqueio (`impact` é o único valor) |
| `--tag`/`--anchor` | Filtros (repetíveis; aceitam vírgula) |
| `--full-content` | Bloco completo (não é pipe-safe) |
| `--universe` | Panorama geral explícito |

### Demais

| Comando | Flags |
|---|---|
| `show` | `--id <ID>...` (repetível; aceita vírgula), `--history` |
| `update` | `--id`, `--statement`, `--status` (`active`/`in_progress`/`blocked`/`closed`), `--parent`, `--checks`, `--anchor`, `--clear-anchors` |
| `close` | `--id`, `--outcome` (`success`/`partial`/`failure`/`abandoned`), `--note` |
| `graph` | `--program <PATH>` ou `--root <ID>` |
| `flow` | `--window-days <N>` |
| `plan` | `--prompt`/`--submit`, `--template` (`feature`/`bug`/`refactor`), `--step`, `--from` |

## Resultado esperado

- **`new`** — o id criado (ou a lista no lote); texto `key|id|scope|status|afirmação`.
- **`list`** — `id|scope|status|afirmação`.
- **`close`** — o resultado e as evidências; `data.epic{done,total}`.
- **`graph`** — a árvore indentada.
- **`flow`** — resumo, throughput e caminho crítico.
- **Erros comuns:** `list` sem filtro → exit 2; raiz/programa ausente → exit 3; colisão de id no
  `--submit` → exit 4; `--kind` incoerente com o escopo → exit 8.

## Quando não usar

- Para conhecimento → [`kd write`](06_write.md).
- Para atribuir dono → não há dono; use `--status in_progress`.
- Para listar conhecimento → [`kd ask`](05_ask.md).

## Veja também

➡️ [`kd rewind`](08_rewind.md) · [`kd map`](09_map.md) · [Conceitos do trabalho](02_ciclo.md)
