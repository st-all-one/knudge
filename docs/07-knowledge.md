# `kd map` / `kd ask --rank` — mapa, ranking e tags

## O que faz

Reúne as visões **agregadas** do corpus de conhecimento:

| Subcomando | O que dá |
|---|---|
| `kd map` | Clusters por eixo (âncora, tipo, classificação, escopo) e, opcionalmente, semânticos |
| `kd ask --rank` | As notas mais **confiáveis**, sem pergunta textual (ex-`ask --rank`, D146) |
| `kd ask --tags` | O vocabulário de tags (`tag\|count`) |

`kd map` e `kd ask --rank` **exigem escopo** (`--tag`/`--anchor`/`--type`/`--class`/`--around`) ou
`--universe` (D143). `kd ask --tags` não é varredura de corpus; a **fila de embeddings** é `kd drain`
(top-level, D170).

## Em 30 segundos

```bash
kd map --universe --axis type     # panorama por tipo
kd ask --rank --universe --limit 10     # mais confiáveis
kd ask --tags                           # vocabulário de tags
kd drain --status                # fila de embeddings
```

## `kd map`

### Nível 1 — visão por eixo

```bash
kd map --universe --axis type
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
kd map --tag retry --axis anchor
kd map --around fact_01abc --depth 2
kd map --scope epic_01abc
```

Os filtros são aplicados **antes** de clusterizar. Sem filtro nem `--universe`, o comando é exit 2
(D143).

### Nível 3 — fase semântica

```bash
kd map --universe --semantic
```

Roda o agrupamento semântico (complete-link) **dentro** de cada cluster estrutural. Requer
embeddings ([`kd drain`](15-embeddings.md)); sem índice, degrada com `warnings[]`.

### Nível 3.5 — comunidades (`GraphRAG`)

```bash
kd map --universe --communities
```

Detecta **comunidades** por Louvain determinístico (D193) sobre as arestas explícitas + âncoras
compartilhadas. Cada comunidade traz um **resumo local** (termos mais frequentes). É off-path
(só roda no `map`) e aditivo no `--json` (`data.communities`). Com `--write`, cada comunidade
vira uma nota-hub (`## Comunidades` no `MAP.md`).

### Nível 3.6 — ontologia leve e claims (D207)

O frontmatter aceita arestas de **ontologia** (`same_as`, `broader`/`narrower`, `related`) e
**claims** SPO (`kd write --claim "sujeito:relação:objeto"`). O `ask --around --via broader`
percorre a hierarquia; a inferência derivada (classes de equivalência por `same_as`, clausura
transitiva de `broader`/`narrower`) é calculada em `graph/ontology.rs`, sem gravar nada. Duas
notas com a mesma `(sujeito, relação)` e objetos divergentes são uma **contradição precisa** e
aparecem no check `integrity` do [`kd doctor`](09-maintenance.md).

### Nível 4 — materializar o mapa (D150)

```bash
kd map --universe --write
```

Materializa `notas/MAP.md` (árvore de grupos + clusters) e uma **nota-hub** (`meta` +
`references`) por cluster — tudo versionado e buscável pelo `ask`. Dá ponto de entrada humano ao
corpus.

## `kd ask --rank`

```bash
kd ask --rank --universe --limit 10
kd ask --rank --tag retry --limit 5
```

Ranqueia por **confiança derivada** (evidência, uso, idade, confirmação por tarefas) — bom para
revisar o que merece atenção. A evidência é **bayesiana** (D189): `outcomes` viram ensaios de
Bernoulli e a confiança usa o **limite inferior** de 95 % do posterior `Beta(1+s, 1+f)` — uma
nota com 1 sucesso não empata com uma com 20 (o canal `stars` usa a média). Saída
`id|statement|score|why`. Exige escopo ou `--universe`.

Quando a evidência empata, a **recência** desempata a favor da nota mais nova (D175): sem
similaridade textual, a idade entra de forma aditiva (`AGE_WEIGHT = 0,05`) — pequena o bastante
para nunca superar a evidência.

Âncoras quebradas também **descontam** a confiança (D203): o `drift` de cada nota (fração de
âncoras quebradas, derivado `.idx/drift.jsonl` e atualizado pelo `maintenance prune`) multiplica
o score por `drift_factor` (`0,5` no pior caso). Assim, uma nota cuja proveniência se perdeu cai
mesmo sem query.

## `kd ask --tags`

```bash
kd ask --tags --limit 20
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
| `--communities` | Detecta comunidades (`GraphRAG`) + resumo local (D193) |
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

## `kd ask --suggest` (D158)

Sugestões semânticas **advisory** a partir do índice vetorial: classifica pares em
`duplicate` (quase-duplicata → merge), `contradiction` (mesmo tópico, banda
`suggestions.contradiction_low..high`) e `link` (relacionadas, sem aresta). Nunca vira aresta
sozinha (D49) — é entrada para `kd write --link` ou `kd maintenance compact`.

```
kd ask --suggest
kd ask --suggest --relation contradiction --limit 10
```

Pipe: `relação|from|to|score`; sem índice vetorial → `[no_results]`.

## `kd config promote` (D157)

Promove conhecimento a **regras governadas** no bloco `knudge:rules` do `AGENTS.md` (irmão do
bloco de protocolo, intocado pelo `init`). Desligado por default (`rules.enabled=false`).

```
kd config promote recommend --universe   # read-only
kd config promote approve <ID> --universe
kd config promote edit <ID> --summary "regra revisada"
kd config promote remove <ID> --universe
kd config promote list
```

Elegíveis: `type=meta|decision`, `classification=foundational`, confiança derivada (D87) ≥
`rules.min_confidence` e sem `contradicts` aberto. Teto rígido `rules.max_promoted` (recusa e
nomeia quem sai). A nota de origem permanece.
