# 09 · `kd map` — o mapa do conhecimento

## Para que serve

Dá uma **visão agregada** do corpus: como o conhecimento se distribui por arquivo, tipo,
classificação ou escopo. Opcionalmente, agrupa por semelhança semântica e detecta **comunidades**
(grupos densos de notas relacionadas). Com `--write`, materializa um índice legível dentro do
próprio corpus.

É o comando de **panorama**, não de busca pontual.

## Quando usar

- **Use** para entender a forma do corpus, achar áreas carentes ou gerar um ponto de entrada humano.
- **Use `--semantic`** quando você já tem embeddings e quer clusters por assunto.
- **Não use** para achar **uma** nota ([`kd ask`](05_ask.md)) nem para listar trabalho
  ([`kd task list`](07_task.md)).

> Ranking por confiança, vocabulário de tags e sugestões de vínculo vivem em [`kd ask`](05_ask.md):
> `kd ask --rank`, `kd ask --tags`, `kd ask --suggest`.

## Sintaxe

```
kd map [--axis anchor|type|classification|scope] [--scope ESCOPO]
       [--semantic] [--communities] [--members] [--write]
       [--type T]... [--class C]... [--tag T]... [--anchor P]...
       [--around ID] [--depth N] [--universe]
```

`kd map` exige um filtro ou `--universe`.

## Exemplos

### 1. Panorama por tipo

```bash
kd map --universe --axis type
```

Saída:

```
docs=467 clusters=7
type|fact|80
type|decision|12
type|task|262
```

### 2. Escopar o mapa

```bash
# 1. só notas com uma tag, agrupadas por arquivo
kd map --tag retry --axis anchor

# 2. vizinhança de uma nota
kd map --around fact_01abc --depth 2

# 3. membros de um épico
kd map --scope epic_01abc
```

### 3. Incluir os membros de cada cluster

```bash
# 1. com membros
kd map --universe --axis anchor --members

# 2. por classificação
kd map --universe --axis classification --members

# 3. limitar via pipe
kd map --universe --axis type --members | head -40
```

### 4. Clusters semânticos (precisa de embeddings)

```bash
# 1. agrupar por assunto
kd map --universe --semantic

# 2. semântico dentro de um recorte
kd map --tag gateway --semantic

# 3. JSON
kd --json map --universe --semantic | jq '.data.semantic'
```

### 5. Comunidades (grupos densos)

```bash
# 1. detectar comunidades
kd map --universe --communities

# 2. comunidades com membros
kd map --universe --communities --members

# 3. JSON
kd --json map --universe --communities | jq '.data.communities'
```

Cada comunidade traz um resumo local (os termos mais frequentes do grupo).

### 6. Materializar o mapa no corpus

```bash
# 1. gerar o índice legível e as notas-hub
kd map --universe --write

# 2. por tipo
kd map --universe --axis type --write

# 3. conferir o resultado
kd ask --anchor notas/MAP.md
```

Cria `notas/MAP.md` (árvore de grupos + clusters) e uma nota-hub por cluster, tudo versionado e
buscável.

## Flags

| Flag | Efeito |
|---|---|
| `--axis <EIXO>` | `anchor`/`type`/`classification`/`scope` |
| `--scope <ESCOPO>` | Restringe aos membros de um épico |
| `--semantic` | Agrupa por semelhança semântica (precisa de embeddings) |
| `--communities` | Detecta comunidades + resumo local |
| `--members` | Inclui os membros de cada cluster |
| `--write` | Materializa `notas/MAP.md` + notas-hub |
| `--type`/`--class`/`--tag`/`--anchor` | Filtros de corpus: tipo, classificação (`foundational`/`tactical`/`observational`), tag, âncora (repetíveis; aceitam vírgula) |
| `--around <ID>` / `--depth <N>` | Vizinhança de uma nota |
| `--universe` | Varredura explícita do projeto inteiro |

## Resultado esperado

- **Texto:** `docs=<n> clusters=<n>` e linhas `<eixo>|<chave>|<contagem>` (membros indentados com
  `--members`).
- **`--json`:** `{docs, clusters[], semantic[]?, communities[]?}`.
- Sem filtro nem `--universe` → exit 2. Sem embeddings, `--semantic` degrada com aviso (não é erro).

## Quando não usar

- Para buscar um termo → [`kd ask`](05_ask.md).
- Para listar/priorizar trabalho → [`kd task list`](07_task.md).
- Para ver a saúde do corpus → [`kd doctor`](10_doctor.md).

## Veja também

➡️ [`kd ask`](05_ask.md) · [`kd doctor`](10_doctor.md) · [Embeddings](18_embeddings.md)
