# `kd maintenance` — saúde, propostas e worker

## O que faz

Reúne a **manutenção** do corpus e o **worker de embeddings**:

| Subcomando | O que dá |
|---|---|
| `doctor` | Relatório de saúde (e reparo reversível com `--fix`); `--audit` foca integridade |
| `compact` | **Propõe** merge/supersede de quase-duplicatas |
| `learn` | **Sugere** notas/links/merges a partir de eventos e âncoras |
| `prune` | **Propõe** aposentadoria (`forget`) por shelf-life/decay |
| `watch-service` | Gerencia o worker de auto-drain e o servidor de embeddings persistente |

`compact`/`learn`/`prune` **só propõem** (D47) e **exigem escopo** ou `--universe` (D144). Nada
muda sem o seu aceite.

## Em 30 segundos

```bash
kd maintenance doctor                  # como está a saúde?
kd maintenance doctor --fix            # repara o reversível
kd maintenance learn --universe        # o que merece virar nota?
```

## `maintenance doctor`

### Nível 1 — relatório

```bash
kd maintenance doctor
```

Uma linha por check:

```
ok schema todas as notas parseiam
ok integrity grafo íntegro
fail derived índice derivado ausente/divergente; `--fix` reconstrói
...
```

Checks: `schema`, `integrity`, `cycles`, `anchors`, `program-anchor`, `duplicates`, `locks`,
`config`, `body_hash`, `events`, `derived`, `embeddings`. `program-anchor` (épico-raiz sem
`plan/*.md`) é **warn**: aparece no relatório mas **não** deixa o corpus "não saudável" — corpus
importado costuma não ter o programa externo.

### Nível 2 — reparar

```bash
kd maintenance doctor --fix
```

Repara o **reversível**: migra layout plano legado (`notas/<id>.md` → `notas/<tipo>/<id>.md`),
normaliza `scope: plan`→`scope: epic` e remove `type: container`/`type: epic`, remove chaves fora
do schema (`confidence`/`expires_at`/`not_before`), recalcula `body_hash`, remove âncoras
quebradas (inclusive diretórios) e locks stale, e reconstrói o índice divergente. É
**idempotente** e nunca apaga notas.

### Nível 3 — auditoria

```bash
kd maintenance doctor --audit
```

Foca integridade de grafo/arestas: âncoras quebradas, ciclos de dependência, duplicatas,
supersessão e arestas sugeridas. O `--json` traz os **detalhes** (ids/pares): `duplicate_pairs`,
`broken_anchor_details`, `missing_edge_details`, `stale_lock_details`, `integrity_issues` e os
ciclos — para agir sem rodar `compact`/`audit` à parte.

## `maintenance compact`

```bash
kd maintenance compact --universe
kd maintenance compact --tag retry
```

Propõe `merge`/`supersede` de quase-duplicatas. A saída é `estrategia|keep|ids|score`. Aplique com
[`kd write --update`](05-write.md) ou [`kd forget`](11-forget.md) — nunca automaticamente.

## `maintenance learn`

```bash
kd maintenance learn --universe
kd maintenance learn --anchor src/gateway.rs
```

Sugere `create_note` (trabalho fechado que virou conhecimento), `link` (notas que compartilham
âncoras) e `merge`. Saída `kind|ids|score`. Itens com `scope` (trabalho) são comparados só pelo
`statement` — corpo template de import não gera `merge`/`supersede` falso.

## `maintenance prune`

```bash
kd maintenance prune --universe
kd maintenance prune --class observational
```

Propõe `forget` por **shelf-life/decay**. É sempre read-only; a aplicação é
[`kd forget`](11-forget.md). Âncora literal que aponta para diretório conta como **quebrada**.

## `maintenance watch-service`

Gerencia o worker de auto-drain **fora** do `kd` (systemd `--user`/launchd).

```bash
kd maintenance watch-service --install        # agendador + servidor + cadastra o projeto
kd maintenance watch-service --status         # saúde (default)
kd maintenance watch-service --subscribe      # cadastra outro projeto
kd maintenance watch-service --unsubscribe    # descadastra (mantém o sistema)
kd maintenance watch-service --uninstall      # remove agendador + servidor
```

- `--install` **baixa `llama.cpp` e o GGUF se faltarem** (script oficial + fallback para
  `brew`/`winget`/`scoop`/`choco`/`apt`/`dnf`/`pacman`/`zypper`); `--no-deps` pula.
- O servidor sobe como unidade/agente próprio (`knudge-embed`), com `-b 2048 -ub 2048` — o
  `--drain` manual e o auto-drain ocioso sempre o encontram.
- Sem `systemd`/`launchd`, o comando recusa o `--install` e imprime a linha de cron equivalente.
- Ações que mudam perguntam no stderr (`s/N`); `--yes` pula; stdin não-TTY cancela.

## Referência de flags

| Subcomando | Flags |
|---|---|
| `doctor` | `--fix`, `--audit` |
| `compact`/`learn`/`prune` | `--scope`, `--type`/`--class`/`--tag`/`--anchor`, `--around`/`--depth`, `--universe` (escopo obrigatório) |
| `prune` | + `--dry-run` (paridade; já é read-only) |
| `watch-service` | `--install`/`--subscribe`/`--unsubscribe`/`--status`/`--uninstall`, `--yes`, `--dry-run`, `--every`, `--port`, `--model`, `--no-deps`, `--script`, `--url` |

## Resultados

- `doctor` — `{checks[], healthy, fixed[]}`; cada check tem `ok`/`warn`; texto `ok|warn|fail <check> <msg>`.
- `compact`/`learn`/`prune` — `{proposals[]}`; texto pipe por linha.
- `watch-service` — `{action, done, script}` ou `{dry_run, action, source, reference, command}`.
- `compact`/`learn`/`prune` sem escopo → exit 2.

## Quando (não) usar

- **Use** periodicamente: `doctor` para saúde, `learn`/`prune` para revisar, `compact` para
  duplicatas.
- **Não use** esperando que algo seja aplicado sozinho: `compact`/`learn`/`prune` **só propõem**.
- Para a fila de embeddings, prefira [`kd knowledge digest`](07-knowledge.md).

## Próximo passo

➡️ [Embeddings](15-embeddings.md) · [`kd config`](10-config.md)
