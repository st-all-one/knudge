# 11 · `kd maintenance` — propostas de limpeza

## Para que serve

Reúne as **propostas** de manutenção do conhecimento. Nenhum destes comandos altera o corpus: eles
listam o que **poderia** ser feito, e você decide aplicar (com [`kd write`](06_write.md) ou
[`kd forget`](14_forget.md)).

| Subcomando | Propõe |
|---|---|
| `compact` | fundir/marcar quase-duplicatas |
| `learn` | criar notas/links/merges a partir do que aconteceu |
| `prune` | aposentar notas obsoletas (shelf-life, âncora quebrada, contradição, drift) |

Todos **exigem um filtro** ou `--universe`.

## Quando usar

- **Use** periodicamente para revisar a memória: `learn` para achar o que merece virar nota,
  `prune` para achar o que já não vale, `compact` para juntar duplicatas.
- **Não use** esperando que algo mude sozinho: aqui é só proposta.

## Sintaxe

```
kd maintenance compact [--scope C] [--verify] [--universe] [filtros]
kd maintenance learn   [--scope C] [--verify] [--universe] [filtros]
kd maintenance prune   [--scope C] [--dry-run] [--universe] [filtros]
```

## Exemplos

### `maintenance compact`

```bash
# 1. propostas no projeto inteiro
kd maintenance compact --universe

# 2. restringir a uma tag
kd maintenance compact --tag retry

# 3. checar evidência de cada proposta
kd maintenance compact --universe --verify
```

Saída: `estratégia|keep|ids|score`. Aplique com `kd write --update` ou `kd forget` — nunca
automaticamente.

### `maintenance learn`

```bash
# 1. o que merece virar nota
kd maintenance learn --universe

# 2. restringir a um arquivo
kd maintenance learn --anchor src/gateway.rs

# 3. com portão de evidência
kd maintenance learn --universe --verify
```

Saída: `kind|ids|score`. Tipos de sugestão incluem criar nota a partir de trabalho concluído, criar
link entre notas que compartilham âncoras e fundir duplicatas.

### `maintenance prune`

```bash
# 1. o que já pode ser aposentado
kd maintenance prune --universe

# 2. só conhecimento observacional
kd maintenance prune --class observational

# 3. ver o motivo e a validade de cada candidata
kd --json maintenance prune --universe | jq '.data.proposals[].reason'
```

Saída: as notas candidatas e o motivo (`expired`, `anchor_decay`, `contradicted`, `defeated`,
`drifted`). A aplicação é [`kd forget`](14_forget.md).

## Flags

| Comando | Flags |
|---|---|
| `compact` | `--scope`, `--verify`, `--type`/`--class`/`--tag`/`--anchor`, `--around`/`--depth`, `--universe` |
| `learn` | idem `compact` |
| `prune` | idem + `--dry-run` (paridade; já é read-only) |

O `--verify` roda o portão de evidência configurado sobre cada proposta e marca
`gate=passed|failed`. É read-only. Os filtros de corpus (`--type`/`--class`/`--tag`/`--anchor`)
são repetíveis e aceitam vírgula: tipo (`fact`/`decision`/`question`/`task`/`def`/`error`/
`snippet`/`link`/`meta`/`risk`), classificação (`foundational`/`tactical`/`observational`), tag e
âncora.

## Resultado esperado

- **Texto:** uma linha por proposta (`tipo|ids|score` ou `estratégia|keep|ids|score`).
- **`--json`:** `{proposals[]}`.
- Sem filtro nem `--universe` → exit 2.
- O `prune` também atualiza o registro de **drift** de âncoras, que o [`kd ask`](05_ask.md) usa para
  descontar a confiança de notas cuja proveniência se perdeu.

## Quando não usar

- Para aplicar uma correção pontual → [`kd write --update`](06_write.md) ou [`kd forget`](14_forget.md).
- Para diagnosticar problemas estruturais → [`kd doctor`](10_doctor.md).

## Veja também

➡️ [`kd doctor`](10_doctor.md) · [`kd forget`](14_forget.md) · [`kd drain`](12_drain.md)
