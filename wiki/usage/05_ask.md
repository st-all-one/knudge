# 05 · `kd ask` — toda a pesquisa

## Para que serve

É **a** ferramenta de consulta do knudge. Em um único comando, ela cobre cinco modos:

| Modo | Como acionar | O que devolve |
|---|---|---|
| **recall** | `kd ask "<pergunta>"` | notas relevantes, por texto |
| **get** | `kd ask --id <ID>` | o corpo de ids específicos |
| **expand** | `kd ask --around <ID>` | vizinhos de uma nota no grafo |
| **rank** | `kd ask --rank` | as notas mais confiáveis (sem pergunta) |
| **tags** | `kd ask --tags` | o vocabulário de tags |
| **suggest** | `kd ask --suggest` | pares parecidos (duplicata/contradição/link) |

Por padrão devolve **só conhecimento** (notas que não são trabalho). Itens de trabalho entram com
`--with-task`; para listar trabalho, prefira [`kd task list`](07_task.md).

A busca é **insensível a acentos** e reconhece variações comuns de palavras (singular/plural,
conjugações), então `configuracao` acha `configuração` e `consultas` acha `consulta`.

## Quando usar

- **Use** antes de gravar, antes de editar um arquivo, ou sempre que precisar lembrar algo.
- **Use `--brief`** quando um agente vai ler a saída (gasta menos contexto).
- **Não use** para criar/editar (é read-only), para listar trabalho
  ([`kd task list`](07_task.md)) nem como histórico de sessão ([`kd rewind`](08_rewind.md)).

## Sintaxe

```
kd ask <QUERY> [filtros] [--limit N] [--brief] [--full-content] [--with-task]
kd ask -                                # lê a QUERY do stdin
kd ask --params '<json>'                # consulta + filtros de uma vez
kd ask --id <ID>...
kd ask --around <ID> [--via ARESTA] [--depth N]
kd ask --rank   [filtros] [--universe]
kd ask --tags   [--limit N]
kd ask --suggest [--relation R] [--top-k N]
```

## Exemplos

### Recall — buscar por texto

```bash
# 1. pergunta simples
kd ask "rate limit do gateway"

# 2. poucos resultados, enxuto
kd ask "cache" --brief --limit 3

# 3. filtrar por tipo e status
kd ask "cache" --type decision --status active
```

Saída típica:

```
fact_01qejflt|O gateway limita 100 rps por chave|0.82|recent
decision_0022xuwr|Rate limit usa janela deslizante|0.51|semantic
```

A linha é `id|afirmação|score|motivo`. O **motivo** (`why`) explica por que aquela nota apareceu:
`file_match`, `anchor_match`, `tracker_match`, `stars`, `semantic`, `recent` ou `universal`.

### Recall — buscar por arquivo (sem texto)

Se você só sabe o arquivo que vai editar:

```bash
# 1. um arquivo
kd ask --anchor src/gateway.rs

# 2. vários arquivos
kd ask --anchor src/gateway.rs --anchor src/queue.rs

# 3. lista com vírgula
kd ask --anchor src/gateway.rs,src/queue.rs
```

### Recall — filtros determinísticos

```bash
# 1. janela de tempo
kd ask "timeout" --since 2026-01-01 --until 2026-06-01

# 2. pertence a um épico
kd ask "parser" --scope epic_01abc

# 3. combinar tipo + tag + classificação
kd ask "gateway" --type decision --tag retry --class tactical
```

Filtros se somam (interseção). Notas esquecidas/substituídas ficam **fora** por padrão; inclua com
`--status forgotten`.

### Get — recuperar corpos por id

```bash
# 1. um id
kd ask --id fact_01m81b6h

# 2. vários ids, com corpo completo (vírgula ou repetindo)
kd ask --id fact_01m81b6h,decision_01abc123 --full-content
kd ask --id fact_01m81b6h --id decision_01abc123 --full-content

# 3. id inexistente degrada com aviso (não é erro)
kd ask --id fact_inexistente
```

### Expand — navegar o grafo

```bash
# 1. vizinhos diretos
kd ask --around fact_01m81b6h

# 2. só por uma relação, mais fundo
kd ask --around fact_01m81b6h --via extends --depth 2

# 3. id inexistente é erro (exit 3)
kd ask --around fact_inexistente
```

### Rank — as mais confiáveis (sem pergunta)

```bash
# 1. panorama do projeto
kd ask --rank --universe --limit 10

# 2. restringir por tag
kd ask --rank --tag retry --limit 5

# 3. por arquivo
kd ask --rank --anchor src/gateway.rs
```

`--rank` exige um escopo (tag/tipo/âncora/…) ou `--universe`. Ele prioriza notas com mais
evidência confirmada, uso e atualidade.

### Tags — o vocabulário

```bash
# 1. todas as tags
kd ask --tags

# 2. limitar
kd ask --tags --limit 20

# 3. uso num pipe
kd ask --tags | head -10
```

Saída: `tag|count`, da mais usada para a menos usada.

### Suggest — parecidos que merecem atenção

```bash
# 1. todas as sugestões
kd ask --suggest

# 2. só contradições
kd ask --suggest --relation contradiction --limit 10

# 3. só possíveis duplicatas
kd ask --suggest --relation duplicate
```

Saída: `relação|from|to|score`. As relações são um conjunto **fechado de três** valores (um valor
fora deles é erro, exit 2):

| `--relation` | Quando aparece (default) | Significado | Ação típica |
|---|---|---|---|
| `duplicate` | similaridade ≥ `dedup.merge_below` (0,92) | quase-duplicata | [`kd write --update`](06_write.md) ou merge |
| `link` | similaridade ≥ `suggestions.contradiction_high` (0,75), **ou** na banda `contradiction_low..high` (0,40–0,75) **com** âncora em comum | relacionadas, sem aresta | [`kd write --link`](06_write.md) |
| `contradiction` | banda `contradiction_low..high` (0,40–0,75) **sem** âncora em comum | mesmo tópico, possível contradição | revisar; talvez `--link ...:contradicts:...` |

Pares que **já têm** aresta nunca aparecem. É **advisory**: nada vira aresta sozinho.

### Consulta em um objeto (automação)

```bash
# 1. query + filtros de uma vez
kd ask --params '{"query":"cache","limit":3,"brief":true}'

# 2. ler o objeto do stdin
echo '{"query":"gateway","type":["decision"]}' | kd ask --params -

# 3. JSON de máquina
kd --json ask "cache" --limit 5 | jq '.data.hits[].id'
```

### Consulta temporal — `--as-of`

```bash
# 1. como o corpus estava numa data
kd ask "postgres" --as-of 2026-07-01T00:00:00.000Z

# 2. JSON com a flag histórica
kd --json ask "postgres" --as-of 2026-07-01 | jq '.data.historical'

# 3. data no futuro é erro (exit 2)
kd ask "postgres" --as-of 2099-01-01
```

## Flags

| Flag | Efeito |
|---|---|
| `[QUERY]...` | Consulta textual (posicional; `-` ou pipe lê stdin) |
| `--params <JSON>` | Objeto com consulta e filtros (`-` lê stdin) |
| `--id <ID>...` | Recupera os corpos dos ids (modo **get**; repetível; aceita vírgula) |
| `--around <ID>` | Expande o grafo a partir de uma nota |
| `--via <ARESTA>` | Restringe o expand a uma relação (as 12 arestas de [`kd write`](06_write.md#ligar-notas-arestas)) |
| `--depth <N>` | Profundidade do expand (default `1`) |
| `--brief` | Saída mínima `id\|afirmação` |
| `--full-content` | Inclui o corpo completo dos resultados |
| `--with-task` | Inclui itens de trabalho (notas com escopo) |
| `--type <T>...` | Filtro por tipo: `fact`, `decision`, `question`, `task`, `def`, `error`, `snippet`, `link`, `meta`, `risk` (repetível; aceita vírgula) |
| `--class <C>...` | Filtro por classificação: `foundational`, `tactical`, `observational` (repetível; aceita vírgula) |
| `--tag <T>...` | Filtro por tag (repetível; aceita vírgula) |
| `--status <S>` | Filtro por status: `active`, `in_progress`, `blocked`, `closed`, `superseded`, `forgotten` |
| `--scope <ID>` | Pertencimento a um épico |
| `--anchor <PATH>...` | Filtro **e** canal de âncoras (repetível; aceita vírgula) |
| `--since <TS>` / `--until <TS>` | Janela de criação |
| `--as-of <TS>` | Como o corpus estava naquele instante |
| `--limit <N>` | Limite de resultados |
| `--rank` | Modo ranking por confiança |
| `--tags` | Modo vocabulário de tags |
| `--suggest` | Modo sugestões semânticas |
| `--universe` | Varredura explícita do projeto inteiro |
| `--top-k <N>` | Vizinhos por nota no `--suggest` (default `5`) |
| `--relation <R>` | `duplicate`/`contradiction`/`link` no `--suggest` |

## Resultado esperado

- **Com resultados:** `id|afirmação|score|motivo`, um por linha.
- **Sem resultados:** o sentinela **`[no_results]`** (exit 0). No `--json`, `data.hits` é `[]`.
- **Corpo progressivo:** o 1º resultado mostra o corpo completo; do 2º ao 5º, um trecho; do 6º em
  diante, só a linha. `--brief` desliga; `--full-content` mostra tudo.
- **`--json`:** cada resultado traz `id`, `statement`, `score`, `confidence`, `why` e as parcelas de
  cada canal de busca.
- **Sem nenhum modo** (sem query/âncora/id/around): imprime o uso e sai com exit 2.

## Quando não usar

- Para criar/editar → [`kd write`](06_write.md) / [`kd task update`](07_task.md).
- Para histórico de sessão → [`kd rewind`](08_rewind.md).
- Para listar trabalho → [`kd task list`](07_task.md).

## Veja também

➡️ [`kd write`](06_write.md) · [`kd map`](09_map.md) · [Embeddings](18_embeddings.md)
