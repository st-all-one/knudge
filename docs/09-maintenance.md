# `kd doctor` e `kd maintenance` — saúde e propostas

## O que faz

`kd doctor` (verbo de topo) é a **saúde**: 13 checks + auditoria, com `--fix` e `--explain` (D163).
`kd maintenance` reúne as **propostas** (o worker de auto-drain vive em `kd drain service`, D186):

| Subcomando | O que dá |
|---|---|
| `compact` | **Propõe** merge/supersede de quase-duplicatas |
| `learn` | **Sugere** notas/links/merges a partir de eventos e âncoras |
| `prune` | **Propõe** aposentadoria (`forget`) por shelf-life/decay |

`compact`/`learn`/`prune` **só propõem** (D47) e **exigem escopo** ou `--universe` (D144). Nada
muda sem o seu aceite.

## Em 30 segundos

```bash
kd doctor                  # como está a saúde?
kd doctor --fix            # repara o reversível
kd maintenance learn --universe        # o que merece virar nota?
```

## `kd doctor`

### Nível 1 — relatório

```bash
kd doctor
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
kd doctor --fix
```

Repara o **reversível**: migra layout plano legado (`notas/<id>.md` → `notas/<tipo>/<id>.md`),
normaliza `scope: plan`→`scope: epic` e remove `type: container`/`type: epic`, remove chaves fora
do schema (`confidence`/`expires_at`/`not_before`), recalcula `body_hash`, remove âncoras
quebradas (inclusive diretórios) e locks stale, e reconstrói o índice divergente. É
**idempotente** e nunca apaga notas.

### Nível 3 — detalhes dos achados

```bash
kd doctor --explain
```

A auditoria é **sempre** incluída (D163): integridade de grafo/arestas — âncoras quebradas,
ciclos de dependência, duplicatas, supersessão e arestas sugeridas. O `--json` traz os
**detalhes** (ids/pares): `duplicate_pairs`, `broken_anchor_details`, `missing_edge_details`,
`stale_lock_details`, `integrity_issues` e os ciclos — para agir sem rodar nada à parte. Com
`--explain`, cada achado mostra `esperado` × `encontrado` × `ação`; `--fix` re-audita e lista o
residual.

## `maintenance compact`

```bash
kd maintenance compact --universe
kd maintenance compact --tag retry
```

Propõe `merge`/`supersede` de quase-duplicatas. A saída é `estrategia|keep|ids|score`. Aplique com
[`kd write --update`](05-write.md) ou [`kd forget`](11-forget.md) — nunca automaticamente. Em
corpus grande e denso, a busca de candidatos usa *blocking* `MinHash`/LSH (D204), mantendo o mesmo
limiar de similaridade.

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
O prazo é **elástico** (D190): cada `outcome` de sucesso estende o shelf-life em
`retention.growth_percent` (default 50 %) e reseta o relógio; o `--json` informa a `retention`
atual de cada candidata (`R < 0,5`). O motivo é um de
`expired`/`anchor_decay`/`contradicted`/`defeated`/`drifted`: uma aresta `contradicts` declarada
(o **lado perdedor**, de menor confiança derivada) entra como candidato (D177); um **dependente**
de premissa retratada (`forgotten`/`superseded`) ou derrotada por `replaces` entra por TMS
(`defeated`, D208); e notas de um **tópico** (âncora) cujo vocabulário mudou muito
(`JS ≥ 0,5` entre a metade antiga e a nova, ≥ 4 notas) entram por **drift** (`drifted`, D208).
O comando também persiste o **drift** de âncoras (`.idx/drift.jsonl`, D203) do
mesmo walk — é o derivado que o `ask`/`ask --rank` leem para descontar a confiança.

## `drain service` (worker)

Gerencia o worker de auto-drain **fora** do `kd` (systemd `--user`/launchd).

```bash
kd drain service --install        # agendador + servidor + cadastra o projeto
kd drain service --status         # saúde (default)
kd drain service --subscribe      # cadastra outro projeto
kd drain service --unsubscribe    # descadastra (mantém o sistema)
kd drain service --uninstall      # remove agendador + servidor (preserva o GGUF)
```

- `--install` **baixa `llama.cpp` e o GGUF se faltarem** (script oficial + fallback para
  `brew`/`winget`/`scoop`/`choco`/`apt`/`dnf`/`pacman`/`zypper`); `--no-deps` pula.
- O servidor sobe como unidade/agente próprio (`knudge-embed`), com `-b 2048 -ub 2048` — o
  `kd drain --digest` manual e o auto-drain ocioso sempre o encontram.
- Sem `systemd`/`launchd`, o comando recusa o `--install` e imprime a linha de cron equivalente.
- Ação explícita = aceite (D180): mutações executam direto, sem prompt.

## Referência de flags

| Comando | Flags |
|---|---|
| `kd doctor` (topo) | `--fix`, `--explain` |
| `compact`/`learn`/`prune` | `--scope`, `--type`/`--class`/`--tag`/`--anchor`, `--around`/`--depth`, `--universe` (escopo obrigatório) |
| `prune` | + `--dry-run` (paridade; já é read-only) |
| `drain service` | `--install`/`--subscribe`/`--unsubscribe`/`--status`/`--uninstall`/`--reconcile`, `--dry-run`, `--every`, `--port`, `--model`, `--no-deps`, `--keep-model`/`--remove-model`, `--script`, `--url`, `--sha256` |

## Resultados

- `kd doctor` — `{checks[], healthy, degraded, status, fixed[], audit{...}, suggestions[]}`; cada
  check tem `ok`/`warn`/`fail`; texto `ok|warn|fail <check> <msg>` + `auditoria:` + `próximos:`.
- `compact`/`learn`/`prune` — `{proposals[]}`; texto pipe por linha.
- `drain service` — `{action, done, script}` ou `{dry_run, action, source, reference, command, plan?}`.
- `compact`/`learn`/`prune` sem escopo → exit 2.

## Quando (não) usar

- **Use** periodicamente: `doctor` para saúde, `learn`/`prune` para revisar, `compact` para
  duplicatas.
- **Não use** esperando que algo seja aplicado sozinho: `compact`/`learn`/`prune` **só propõem**.
- Para a fila de embeddings, prefira [`kd drain`](07-knowledge.md).

## Próximo passo

➡️ [Embeddings](15-embeddings.md) · [`kd config`](10-config.md)

## `--verify` — portão de evidência (D156)

`compact --verify`/`learn --verify` rodam os gates configurados (`proposals.gate`, catálogo
`validators.toml` com `kind = "gate"`) sobre cada proposta e anexam `gate=passed|failed` ao
pipe (e `gate` no `--json`). É **read-only**. Com `proposals.enforce=true`, o mesmo gate roda
no `write` e **bloqueia** a gravação quando reprova (exit 4).

## Check `body` e gate de corpo (D162/D191)

O `kd doctor` tem o check **advisório** `body`: conta notas de conhecimento ativas **sem corpo**
**sem lastro** (sem corpo **e** sem `outcome` **e** sem âncora) e notas com **slots mínimos
ausentes** (D191). Como é advisório, não deixa o corpus `unhealthy`; mas deixa o `status`
`degraded` — aparece em `warnings`/`--json`.

O **data contract por tipo** (D191) espera seções de corpo: `decision` → `Alternativas`/`Por quê`/
`Consequência`; `error` → `Causa`/`Correção`; `risk` → `Probabilidade`/`Impacto`; `def` →
`Significado`; `snippet` → `Linguagem` + âncora; `question` → âncora/`depends_on`. `kd write`
emite **aviso soft** (ou `invalid_input`, exit 2, com `behavior.strict=true`) e `--dry-run` lista
os slots em `missing_slots`. Casamento por cabeçalho (`## Alternativas`) ou rótulo (`Alternativas:`),
sem caixa/acento, em PT-BR ou inglês.

Para **exigir** corpo em `decision`, configure um gate (D156) no `validators.toml`:

```toml
[body]
cmd = "/caminho/para/gate-body.sh"   # lê {op,before,after} no stdin; escreve {passed,...}
kind = "gate"
```

```sh
# gate-body.sh: reprova decision sem corpo
#!/bin/sh
input=$(cat)
echo "$input" | grep -q '"type":"decision"' || { echo '{"passed":true,"score_before":0,"score_after":1}'; exit 0; }
echo "$input" | grep -q '"body":""' && { echo '{"passed":false,"score_before":0,"score_after":0}'; exit 0; }
echo '{"passed":true,"score_before":0,"score_after":1}'
```

Com `proposals.gate=body` e `proposals.enforce=true`, o `kd write` bloqueia (`conflict`, exit 4)
quando o gate reprova.
