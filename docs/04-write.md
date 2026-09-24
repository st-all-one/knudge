# 04 — Escrita: `kd write`

O `kd write` cobre **toda** a escrita de conhecimento: criar, versionar, criar arestas e anexar
evidência.

```
kd write --type <T> "<statement>" [opções]
kd write --update <ID> "<statement>"
kd write --link <FROM:ARESTA:TO>
kd write --outcome <success|partial|failure|abandoned> <ID> [--note TXT]
kd write --batch - [--dry-run]
```

## Criar uma nota

```bash
kd write --type decision "Rate limit é 100 rps por chave" \
  --tag gateway --anchor src/gateway.rs --class tactical
```

Antes de criar, **busque**: `kd ask "<rascunho>"`. O `write` também faz dedup automático
(`< 0.75` cria, `0.75–0.92` merge, `≥ 0.92` rejeita).

| Opção | Para quê |
|---|---|
| `--type T` | Tipo (default `fact`); `task` é **rejeitado** — use `kd task` |
| `--body -\|TXT` | Corpo (`-` lê stdin) |
| `--tag T...` | Tags |
| `--anchor PATH...` | Âncora a arquivo/glob (repetível; aceita vírgula) |
| `--class C` | Classificação (`foundational`/`tactical`/`observational`) |
| `--status S` | Status inicial |
| `--edge ARESTA:ID` | Aresta a partir da nota |

## Ancorar (o que liga a memória ao código)

Toda nota que fala de um arquivo ou módulo **deve** ser ancorada:

```bash
kd write --type error "O parser quebra com NBSP" --anchor src/toon/parse.rs
kd write --type decision "Cache usa LRU" --anchor src/cache.rs --anchor src/cache/**
```

- **Repetível** e aceita vírgula: `--anchor a.rs --anchor b.rs` ou `--anchor a.rs,b.rs`.
- **Glob** (`src/cache/**`) casa subárvores.
- Depois, `kd ask --anchor src/cache.rs` acha tudo que toca aquele arquivo.
- `kd maintenance doctor --audit` lista **âncoras quebradas** (arquivo removido).

## Versionar (update)

```bash
kd write --update decision_01abc123 "Rate limit é 200 rps por chave"
```

Mudar o **statement** (ou o `type`) cria um **novo id** e marca o antigo como `superseded`;
mudar só campos complementares mantém o id. **Nunca invente id** — copie da saída do `ask`.

## Arestas explícitas (link)

```bash
kd write --link "decision_01abc:refines:fact_01xyz"
```

Aresta com **via única** (`FROM:ARESTA:TO`); inclui `depends_on` entre tarefas. Para criar
aresta ao gravar, use `--edge`.

## Evidência (outcome)

```bash
kd write --outcome success decision_01abc --note "testes verdes em CI"
```

Anexa um resultado (`success|partial|failure|abandoned`) — é assim que uma tarefa concluída
**confirma** as notas que compartilham âncoras (não escreva "confirmado" à mão).

## Lote (batch)

```bash
kd write --batch rascunhos.jsonl --dry-run   # simula
kd write --batch rascunhos.jsonl             # aplica
cat rascunhos.jsonl | kd write --batch -
```

Aceita JSONL de rascunhos; `--dry-run` mostra o que faria sem gravar.

## Onde gravar o quê

| Você quer… | Tipo |
|---|---|
| Um fato durável | `fact` |
| Uma decisão e o porquê | `decision` |
| Um erro e como reproduzir | `error` |
| Um risco ou armadilha | `risk` |
| Uma pergunta em aberto | `question` |
| Um trecho de código/link | `snippet`/`link` |
| Trabalho a fazer | **`kd task`** (não `write`) |

## Quando **não** usar

- **Trabalho/tarefa** — use [`kd task`](05-task.md).
- **Sem buscar antes** — risco de duplicata; rode `kd ask "<rascunho>"` primeiro.

## Próximo passo

➡️ [Tarefas — `kd task`](05-task.md)
