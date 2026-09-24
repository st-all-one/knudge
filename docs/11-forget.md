# `kd forget` — esquecer notas

## O que faz

**Soft-delete** de uma nota: marca `status: forgotten` em vez de apagar. Notas esquecidas ficam
**fora** do `ask` por padrão, mas continuam no disco e no git. `--restore` desfaz; `--purge` remove
de fato **após a retenção**.

## Em 30 segundos

```bash
kd forget --id fact_01abc              # soft-delete
kd forget --id fact_01abc --restore    # desfaz
kd forget --id fact_01abc --purge      # remove fisicamente (após retenção)
```

## Uso

```
kd forget --id <ID> [--restore | --purge [--force]]
```

| Flag | Efeito |
|---|---|
| `--id <ID>` | Id da nota (obrigatório) |
| `--restore` | Restaura em vez de esquecer |
| `--purge` | Remove fisicamente após a retenção |
| `--force` | Com `--purge`: ignora a retenção e purga um tombstone já `forgotten`/`superseded` |

## Exemplos

### Nível 1 — esquecer

```bash
kd forget --id fact_01abc
```

Saída: `forget|fact_01abc|r2` (a revisão incrementa). A nota some do `ask`:

```bash
kd ask "termo" --status forgotten   # inspeciona a linhagem
```

### Nível 2 — restaurar

```bash
kd forget --id fact_01abc --restore
```

### Nível 3 — purgar

```bash
kd forget --id fact_01abc --purge
```

O `--purge` só age quando a nota está aposentada **e** a retenção venceu
(`retention.retired_days`, default 30). Para purgar imediatamente um tombstone
(`superseded`/`forgotten`) sem mexer na política:

```bash
kd forget --id fact_01abc --purge --force
```

`--force` **não** purga nota viva: exige status `forgotten` ou `superseded`.

O `--purge` também remove as **arestas de entrada** das demais notas (sem pontas soltas no grafo).

## Resultados

- Texto: `forget|<id>|rN`, `restore|<id>|rN` ou `purge|<id>`.
- `--json`: `{action, id, revision}`.
- `--purge` sem retenção vencida → exit 2 (`invalid_input`); use `--force` para tombstones.
- Nota ausente → exit 3.

## Quando (não) usar

- **Use** para aposentar conhecimento obsoleto ou remover o que não serve mais.
- **Prefira `--restore`** se estiver em dúvida — o soft-delete é reversível.
- **Não use** como controle de acesso; o corpus é local e versionado.

## Próximo passo

➡️ [`kd sync`](12-sync.md) · [`kd maintenance`](09-maintenance.md) (`prune` propõe `forget`)
