# 14 · `kd forget` — esquecer notas

## Para que serve

Aposenta uma nota **sem apagá-la de imediato**. O `forget` marca a nota como esquecida (soft-delete):
ela sai da busca por padrão, mas continua no disco e no git. `--restore` desfaz; `--purge` remove de
fato depois da carência.

Esquecer é **reversível** — é a forma segura de dizer "isto não vale mais".

## Quando usar

- **Use** para aposentar conhecimento obsoleto ou que não serve mais.
- **Use `--restore`** se estiver em dúvida (o soft-delete desfaz).
- **Use `--purge`** só quando tiver certeza de que a nota nunca mais será necessária.
- **Não use** como controle de acesso: o corpus é local e versionado.

## Sintaxe

```
kd forget --id <ID> [--restore | --purge [--force]]
```

## Exemplos

### 1. Esquecer (soft-delete)

```bash
kd forget --id fact_01abc
```

Saída: `forget|fact_01abc|r2`. A nota some da busca; você ainda a inspeciona com:

```bash
kd ask "termo" --status forgotten
```

### 2. Restaurar

```bash
# 1. desfazer
kd forget --id fact_01abc --restore

# 2. conferir que voltou
kd ask "termo"

# 3. JSON
kd --json forget --id fact_01abc --restore | jq '.data.action'
```

### 3. Purgar (remoção física)

```bash
# 1. purgar após a carência
kd forget --id fact_01abc --purge

# 2. purgar imediatamente um tombstone (já esquecido/substituído)
kd forget --id fact_01abc --purge --force

# 3. purgar várias? uma por vez (o `--id` é único)
kd forget --id fact_01def --purge
```

`--purge` só age quando a nota está aposentada **e** a carência venceu. `--force` exige que a nota
já esteja esquecida/substituída; **não** purga nota viva. O `--purge` também remove as arestas de
entrada das outras notas.

## Flags

| Flag | Efeito |
|---|---|
| `--id <ID>` | Id da nota (obrigatório) |
| `--restore` | Restaura em vez de esquecer |
| `--purge` | Remove fisicamente após a carência |
| `--force` | Com `--purge`: ignora a carência para um tombstone |

## Resultado esperado

- **Texto:** `forget|<id>|rN`, `restore|<id>|rN` ou `purge|<id>`.
- **`--json`:** `{action, id, revision}`.
- `--purge` sem carência vencida → exit 2 (use `--force` para tombstones).
- Nota ausente → exit 3.

## Quando não usar

- Para corrigir o texto de uma nota → [`kd write --update`](06_write.md).
- Para aposentar em massa → primeiro [`kd maintenance prune`](11_maintenance.md), depois aplique.

## Veja também

➡️ [`kd maintenance`](11_maintenance.md) · [`kd sync`](15_sync.md) · [`kd write`](06_write.md)
