# `kd knowledge` — mapa, ranking, tags e digestão

## O que faz

Reúne as visões **agregadas** do corpus de conhecimento:

| Subcomando | O que dá |
|---|---|
| `map` | Clusters por eixo (âncora, tipo, classificação, escopo) e, opcionalmente, semânticos |
| `digest` | Estado/dreno da fila de embeddings (ex-`maintenance index`, D145) |
| `rank` | As notas mais **confiáveis**, sem pergunta textual (ex-`ask --rank`, D146) |
| `tags` | O vocabulário de tags (`tag\|count`) |

`map` e `rank` **exigem escopo** (`--tag`/`--anchor`/`--type`/`--class`/`--around`) ou
`--universe` (D143). `digest` e `tags` não são varredura de corpus.

## Em 30 segundos

```bash
kd knowledge map --universe --axis type     # panorama por tipo
kd knowledge rank --universe --limit 10     # mais confiáveis
kd knowledge tags                           # vocabulário de tags
kd knowledge digest --status                # fila de embeddings
```

## `knowledge map`

### Nível 1 — visão por eixo

```bash
kd knowledge map --universe --axis type
```

```
docs=467 clusters=7
type|fact|80
type|decision|12
type|task|262
...
```

Eixos: `anchor` (arquivo), `type`, `classification`, `scope`. O pipe é
`<axis>|<key>|<count>`; com `--members`, os membros vêm indentados `id|statement`.

### Nível 2 — escopar o mapa

```bash
kd knowledge map --tag retry --axis anchor
kd knowledge map --around fact_01abc --depth 2
kd knowledge map --scope epic_01abc
```

Os filtros são aplicados **antes** de clusterizar. Sem filtro nem `--universe`, o comando é exit 2
(D143).

### Nível 3 — fase semântica

```bash
kd knowledge map --universe --semantic
```

Roda o agrupamento semântico (complete-link) **dentro** de cada cluster estrutural. Requer
embeddings ([`kd knowledge digest`](15-embeddings.md)); sem índice, degrada com `warnings[]`.

### Nível 4 — materializar o mapa (D150)

```bash
kd knowledge map --universe --write
```

Materializa `notas/MAP.md` (árvore de grupos + clusters) e uma **nota-hub** (`meta` +
`references`) por cluster — tudo versionado e buscável pelo `ask`. Dá ponto de entrada humano ao
corpus.

## `knowledge rank`

```bash
kd knowledge rank --universe --limit 10
kd knowledge rank --tag retry --limit 5
```

Ranqueia por **confiança derivada** (evidência, uso, idade, confirmação por tarefas) — bom para
revisar o que merece atenção. Saída `id|statement|score|why`. Exige escopo ou `--universe`.

## `knowledge tags`

```bash
kd knowledge tags --limit 20
```

Lista `tag|count` (count desc) — ajuda a escolher tags consistentes.

## `knowledge digest`

```bash
kd knowledge digest --status     # estado da fila (pending)
kd knowledge digest --drain      # digere um lote agora (repita para mais)
```

Digere o conteúdo num vetor (384d). `--status` é o default. Com `embeddings.mode=lazy`, o CLI já
drena um lote ao fim de cada verbo; `--drain` esvazia o resto. Ver
[Embeddings](15-embeddings.md).

## Referência de flags

### `map`

| Flag | Efeito |
|---|---|
| `--axis <EIXO>` | `anchor`/`type`/`classification`/`scope` |
| `--scope <ESCOPO>` | Restringe aos membros de um épico |
| `--semantic` | Fase 2 semântica dentro dos clusters |
| `--members` | Inclui os membros de cada cluster |
| `--write` | Materializa `notas/MAP.md` + hubs |
| `--type`/`--class`/`--tag`/`--anchor` | Filtros de corpus (repetíveis) |
| `--around <ID>` `--depth <N>` | Vizinhança de uma nota |
| `--universe` | Varredura explícita do projeto inteiro |

### `rank` / `tags` / `digest`

`rank`: mesmos filtros de corpus + `--universe` + `--limit`.
`tags`: `--limit`.
`digest`: `--status` (default) / `--drain`.

## Resultados

- `map` — `{docs, clusters[], semantic[]}`; pipe `<axis>|<key>|<count>`.
- `rank` — `{ranked[]}` com `id`/`statement`/`confidence`/`why`/`channels`.
- `tags` — `{tags[]}`.
- `digest` — `{pending}` ou `{enabled, indexed, pending, stale, cache_hits}`.

## Quando (não) usar

- **Use** para panorama (`map`), priorização (`rank`), consistência de tags (`tags`) e fila de
  embeddings (`digest`).
- **Não use** para achar **uma** nota (use [`kd ask`](04-ask.md)) nem para listar trabalho
  ([`kd task list`](06-task.md)).

## Próximo passo

➡️ [`kd rewind`](08-rewind.md) · [Embeddings](15-embeddings.md)
