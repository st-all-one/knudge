# 03 — Busca: `kd ask`

O `kd ask` é **a** ferramenta de pesquisa: recall (busca), get (corpo por id) e expand (grafo),
tudo no mesmo comando.

```
kd ask <QUERY> [filtros] [--limit N] [--brief] [--with-body]
kd ask --id <ID>...
kd ask --around <ID> [--via ARESTA] [--depth N]
kd ask --rank
kd ask --tags
```

## Busca textual

```bash
kd ask "como o cache é invalidado"
kd ask "retry do gateway" --brief --limit 3
```

A saída é `id|statement|score|why`, um hit por linha:

- **`why = lexical`** — casou por BM25 (termos).
- **`why = semantic`** — casou pelo vetor (paráfrase; precisa de embeddings).
- **`why = anchor`/`file_match`** — casou pela âncora de arquivo.

## Filtros determinísticos

```bash
kd ask "cache" --type decision --status active
kd ask "gateway" --tag retry --class tactical
kd ask "timeout" --since 2026-01-01 --until 2026-06-01
kd ask "parser" --container plan_01abc
```

| Flag | Filtra por |
|---|---|
| `--type T...` | Tipo da nota (`fact`, `decision`, …) |
| `--class C...` | Classificação (`foundational`, `tactical`, `observational`) |
| `--tag T...` | Tag (repetível) |
| `--status S` | Status (`active`, `in_progress`, `blocked`, `closed`, `superseded`, `forgotten`) |
| `--container ID` | Nota sob um container |
| `--anchor PATH...` | Arquivo/glob ancorado (repetível; aceita vírgula) |
| `--since TS` / `--until TS` | Janela de criação (RFC3339/epoch) |

> `forgotten`/`superseded` ficam **fora** do `ask` por padrão; inclua com `--status`.

## Busca por âncora (sem query)

Ancorar é o que liga a memória ao código. Se você só sabe o arquivo:

```bash
kd ask --anchor src/gateway.rs
kd ask --anchor src/gateway.rs --anchor src/queue/**
kd ask --anchor src/gateway.rs,src/queue.rs
```

Isso usa o **canal de âncoras** e funciona mesmo sem texto — ideal para "o que já sei sobre
este arquivo?" antes de editá-lo.

## Corpo de ids

```bash
kd ask --id fact_01m81b6h
kd ask --id fact_01m81b6h decision_01abc123 --with-body
```

O `--with-body` inclui o corpo dos hits da busca; `--id` recupera corpos diretamente.

## Expandir o grafo

```bash
kd ask --around fact_01m81b6h                    # vizinhos diretos
kd ask --around fact_01m81b6h --via refines --depth 2
kd ask --around fact_01m81b6h --depth 3
```

## Mais confiáveis, sem query

```bash
kd ask --rank --limit 10
```

Ranqueia por **confiança derivada** (evidência, uso, idade) — bom para revisar o que merece
atenção (D107).

## Vocabulário de tags

```bash
kd ask --tags
```

Lista `tag|count` (count decrescente) — ajuda a escolher tags consistentes.

## Para agentes / `--json`

```bash
kd ask "cache" --brief --json
```

Em `--json`, o stdout é **só** o envelope `{success, command, data, warnings?}`; nenhum log
vaza. Use `--brief` e `--limit` para gastar menos contexto.

## Quando **não** usar

- **Criar/editar** — o `ask` é read-only; use [`kd write`](04-write.md).
- **Histórico de sessão** — use [`kd rewind`](07-manutencao.md).
- **Listar trabalho** — use [`kd task list`](05-task.md).

## Próximo passo

➡️ [Escrita — `kd write`](04-write.md)
