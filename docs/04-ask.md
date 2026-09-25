# `kd ask` — toda a pesquisa

## O que faz

É **a** ferramenta de pesquisa do knudge. Cobre três modos no mesmo comando:

- **recall** — busca textual por relevância (filtros → BM25 → âncoras → RRF);
- **get** — recupera os **corpos** de ids específicos (`--id`);
- **expand** — navega o grafo explícito a partir de uma nota (`--around`).

Por padrão devolve **só conhecimento** (notas sem `scope`); itens de trabalho entram com
`--with-task`. A saída é um pipe de uma linha por hit: `id|statement|score|why`.

## Em 30 segundos

```bash
kd ask "como o cache é invalidado"      # busca textual
kd ask --id fact_01m81b6h               # corpos por id
kd ask --around fact_01m81b6h           # vizinhos no grafo
```

Use `--brief` para gastar menos contexto: a linha vira `id|statement`.

## Uso

```
kd ask <QUERY> [filtros] [--limit N] [--brief] [--full-content] [--with-task]
kd ask -                                   # lê a QUERY do stdin
kd ask --params '<json>'                   # consulta + filtros de uma vez (D147)
kd ask --id <ID>...
kd ask --around <ID> [--via ARESTA] [--depth N]
```

## Exemplos

### Nível 1 — o básico

```bash
kd ask "rate limit do gateway"
```

```
fact_01qejflt|O gateway limita 100 rps por chave|0.82|recent
decision_0022xuwr|Rate limit usa janela deslizante|0.51|semantic
```

- **`id`** — endereço da nota;
- **`statement`** — a afirmação (sanitizada: sem `|` nem quebras);
- **`score`** — score fundido (RRF), ordenado desc;
- **`why`** — o motivo dominante do hit.

### Nível 2 — gastar menos contexto

```bash
kd ask "cache" --brief --limit 3
```

```
fact_01qejflt|O cache usa LRU
decision_0022xuwr|Cache expira em 30 dias
```

### Nível 3 — filtros determinísticos

```bash
kd ask "cache" --type decision --status active
kd ask "gateway" --tag retry --class tactical
kd ask "timeout" --since 2026-01-01 --until 2026-06-01
kd ask "parser" --scope epic_01abc
```

Filtros compõem: o resultado é a interseção. `forgotten`/`superseded` ficam **fora** por padrão;
inclua com `--status forgotten`.

### Nível 4 — busca por arquivo (sem query)

Ancorar é o que liga a memória ao código. Se você só sabe o arquivo:

```bash
kd ask --anchor src/gateway.rs
kd ask --anchor src/gateway.rs --anchor src/queue/**
kd ask --anchor src/gateway.rs,src/queue.rs
```

Isso usa o **canal de âncoras** e funciona mesmo sem texto — ideal para "o que já sei sobre este
arquivo?" antes de editá-lo.

### Nível 5 — corpos e grafo

```bash
kd ask --id fact_01m81b6h decision_01abc123 --full-content
kd ask --around fact_01m81b6h --via extends --depth 2
```

`--id` recupera corpos diretamente; `--around` expande o grafo explícito a partir de uma nota
(aresta opcional em `--via`, profundidade em `--depth`).

### Nível 6 — incluir trabalho e JSON

```bash
kd ask "cache" --with-task
kd --json ask "cache" --limit 5
```

Sem `--with-task`, o `ask` é **só conhecimento**. Para listar/ordenar trabalho, prefira
[`kd task list`](06-task.md).

## Referência de flags

| Flag | Efeito |
|---|---|
| `[QUERY]...` | Consulta textual (posicional; `-` ou pipe lê stdin) |
| `--params <JSON>` | Objeto com `query`/filtros/`limit`/`brief`/… (`-` lê stdin) |
| `--id <ID>...` | Recupera os corpos dos ids (modo **get**) |
| `--around <ID>` | Expande o grafo (modo **expand**) |
| `--via <ARESTA>` | Tipo de aresta do expand (default: todas) |
| `--depth <N>` | Profundidade do expand (default `1`) |
| `--brief` | Saída mínima `id\|statement` |
| `--full-content` | Inclui o corpo completo dos hits |
| `--with-task` | Inclui notas com `scope` (trabalho) |
| `--type <T>...` | Filtro por tipo |
| `--class <C>...` | Filtro por classificação |
| `--tag <T>...` | Filtro por tag |
| `--status <S>` | Filtro por status |
| `--scope <ID>` | Pertencimento a um escopo (épico) |
| `--anchor <PATH>...` | Filtro **e** canal de âncoras (repetível; aceita vírgula) |
| `--since <TS>` / `--until <TS>` | Janela de criação |
| `--limit <N>` | Limite de hits (default: `recall.default_limit`, 5) |

### Os 7 valores de `why`

| `why` | Significado |
|---|---|
| `file_match` | Casou por um arquivo do working set |
| `anchor_match` | Casou pelo id do working set |
| `tracker_match` | Pertence ao `--scope` pedido |
| `stars` | Tem confirmação derivada (`outcomes` ou tarefas com sucesso) |
| `semantic` | Casou pelo vetor (paráfrase; precisa de embeddings) |
| `recent` | É recente |
| `universal` | Entrou sem sinal específico (fallback) |

## Resultados

- **Busca sem hit** → `stdout` é o sentinela **`[no_results]`** (exit 0). No `--json`,
  `data.hits` é `[]`. Isso distingue "busca vazia" de erro.
- **`--json`** — cada hit traz `id`, `statement`, `score`, `confidence`, `why` e o objeto
  **`channels`** com as parcelas de cada sinal:

```json
{ "lexical": 0.016, "anchor": 0.0, "semantic": 0.032, "recent": 1.0, "stars": 0.0 }
```

`lexical`/`anchor`/`semantic` são as **parcelas RRF** (somam o `score`); `recent`/`stars` são
boosts informativos em `[0,1]`. O pipe `id|statement|score|why` **não muda**.
- **`--id` de nota ausente** degrada para `warnings[]` (exit 0); **`--around` de nota ausente** é
  `not_found` (exit 3).
- Sem nenhum modo (query e âncora vazias, sem `--id`/`--around`) → imprime o **uso** com exit 2.

## Dedup antes de gravar

O uso canônico do `ask` é **antes** do `write`:

```bash
kd ask "<rascunho>"          # score < 0.75 cria | 0.75–0.92 merge | >= 0.92 rejeita
```

Isso evita duplicata e mostra a nota que talvez você só precise atualizar
(`kd write --update <ID> --summary "..."`).

## Quando (não) usar

- **Use** para achar o que já se sabe, por texto, arquivo ou grafo.
- **Não use** para criar/editar (é read-only) nem como histórico de sessão (use
  [`kd rewind`](08-rewind.md)).
- **Não use** para listar trabalho ([`kd task list`](06-task.md)) nem para ranking por confiança
  ([`kd knowledge rank`](07-knowledge.md)).

## Próximo passo

➡️ [`kd write`](05-write.md) · [Embeddings](15-embeddings.md)

## `--as-of <TS>` — consulta temporal (D155)

Reconstrói o corpus **ativo em `T`** a partir dos eventos (`forget`/`restore` e a cadeia de
supersessão) e roda o mesmo pipeline determinístico — o ranking de `T` é reproduzível.

```
kd ask "postgres" --as-of 2026-07-01T00:00:00.000Z
```

O pipe ganha um banner `as_of=<TS>`; o `--json` traz `as_of` e `historical: true` por hit.
`T` no futuro → exit 2. Nota cujo conteúdo já foi purgado vira `warnings[]` (não é
reconstruível).

## Corpo na resposta — revelação progressiva (D161)

Por padrão o `ask` mostra o corpo da nota de forma progressiva:

- **1º hit** — corpo **completo**.
- **2º–5º** — corpo **truncado** a `recall.preview_chars` (default 280).
- **6º+** — só o padrão `id|statement|score|why`.

`--brief` desliga (só `id|statement`); `--full-content` mostra o corpo completo de todos.
O `--json` traz, por hit, `body_match` (o termo casou no corpo) e `body_snippet` (trecho).

```
kd ask "postgres"
kd ask "postgres" --limit 10        # só o top-5 mostra corpo
kd ask "postgres" --brief           # enxuto
```
