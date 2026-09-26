# `kd knowledge` — mapa, ranking e tags

## O que faz

Reúne as visões **agregadas** do corpus de conhecimento:

| Subcomando | O que dá |
|---|---|
| `map` | Clusters por eixo (âncora, tipo, classificação, escopo) e, opcionalmente, semânticos |
| `rank` | As notas mais **confiáveis**, sem pergunta textual (ex-`ask --rank`, D146) |
| `tags` | O vocabulário de tags (`tag\|count`) |

`map` e `rank` **exigem escopo** (`--tag`/`--anchor`/`--type`/`--class`/`--around`) ou
`--universe` (D143). `tags` não é varredura de corpus; a **fila de embeddings** é `kd drain`
(top-level, D170).

## Em 30 segundos

```bash
kd knowledge map --universe --axis type     # panorama por tipo
kd knowledge rank --universe --limit 10     # mais confiáveis
kd knowledge tags                           # vocabulário de tags
kd drain --status                # fila de embeddings
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
embeddings ([`kd drain`](15-embeddings.md)); sem índice, degrada com `warnings[]`.

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

## Fila de embeddings — `kd drain`

O estado/digestão da fila de embeddings é verbo de topo (D170):

```bash
kd drain --status              # estado rico (provider/mode/dimensions, pending/stale)
kd drain --digest             # digere lotes até esvaziar/estagnar
kd drain --digest --force     # apaga `.idx/` (derivado) e redigeri tudo
```

Com `embeddings.mode=lazy`, o CLI já drena um lote ao fim de cada verbo; `--digest` esvazia o
resto. Ver [Embeddings](15-embeddings.md).

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

### `rank` / `tags`

`rank`: mesmos filtros de corpus + `--universe` + `--limit`.
`tags`: `--limit`.

## Resultados

- `map` — `{docs, clusters[], semantic[]}`; pipe `<axis>|<key>|<count>`.
- `rank` — `{ranked[]}` com `id`/`statement`/`confidence`/`why`/`channels`.
- `tags` — `{tags[]}`.

## Quando (não) usar

- **Use** para panorama (`map`), priorização (`rank`) e consistência de tags (`tags`); a fila de
  embeddings é [`kd drain`](15-embeddings.md).
- **Não use** para achar **uma** nota (use [`kd ask`](04-ask.md)) nem para listar trabalho
  ([`kd task list`](06-task.md)).

## Próximo passo

➡️ [`kd rewind`](08-rewind.md) · [Embeddings](15-embeddings.md)

## `knowledge suggest` (D158)

Sugestões semânticas **advisory** a partir do índice vetorial: classifica pares em
`duplicate` (quase-duplicata → merge), `contradiction` (mesmo tópico, banda
`suggestions.contradiction_low..high`) e `link` (relacionadas, sem aresta). Nunca vira aresta
sozinha (D49) — é entrada para `kd write --link` ou `kd maintenance compact`.

```
kd knowledge suggest
kd knowledge suggest --relation contradiction --limit 10
```

Pipe: `relação|from|to|score`; sem índice vetorial → `[no_results]`.

## `knowledge promote` (D157)

Promove conhecimento a **regras governadas** no bloco `knudge:rules` do `AGENTS.md` (irmão do
bloco de protocolo, intocado pelo `init`). Desligado por default (`rules.enabled=false`).

```
kd knowledge promote recommend --universe   # read-only
kd knowledge promote approve <ID> --universe
kd knowledge promote edit <ID> --summary "regra revisada"
kd knowledge promote remove <ID> --universe
kd knowledge promote list
```

Elegíveis: `type=meta|decision`, `classification=foundational`, confiança derivada (D87) ≥
`rules.min_confidence` e sem `contradicts` aberto. Teto rígido `rules.max_promoted` (recusa e
nomeia quem sai). A nota de origem permanece.
