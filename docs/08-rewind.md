# `kd rewind` — estado/handoff ponto-no-tempo

## O que faz

Reconstrói o **contexto** no início de uma sessão, dentro de um **orçamento de tokens**. Em vez de
buscar um termo, o `rewind` responde "onde eu estava?": quantas notas há, o que é recente, quais
tarefas estão prontas e o que está desatualizado. É o comando de **retomada**, não de busca
dirigida.

## Em 30 segundos

```bash
kd rewind                     # manifest + tarefas prontas (next:)
kd rewind --budget 2000       # orçamento de tokens
kd rewind --files src/gateway.rs
```

## Uso

```
kd rewind [--scope C] [--files PATH]... [--budget N]
          [--since TS] [--until TS] [--resume CONTEXT_ID]
          [--tag T]... [--anchor P]... [--type T]... [--class C]...
          [--around ID] [--depth N]
```

## Exemplos

### Nível 1 — o manifest

```bash
kd rewind --budget 2000
```

```
notes=526 ready=304 blocked=0 containers=59 clean
recent: fact_01ibc4s4 decision_0022xuwr task_01pog2v9
next: task_00aqet3d|B2.1: PHPStan nivel 0 em V2/Core task_005y362k|B2.2: ...
fresh: stale=0 expiring=0 pending=467
```

- **`notes`** — total de notas (no escopo);
- **`ready`/`blocked`** — views de dependência;
- **`containers`** — épicos;
- **`clean`/`dirty`** — se o working set mudou desde a última sessão;
- **`recent:`** — até 3 notas mais recentes;
- **`next:`** — até 5 tarefas `ready` e **abertas** por impacto;
- **`fresh:`** — notas `stale`/`expiring` e fila `pending` de embeddings.

### Nível 2 — working set por arquivos

```bash
kd rewind --files src/gateway.rs
kd rewind --files src/gateway.rs --files src/queue.rs
```

Ranqueia as notas que tocam esses arquivos — ideal para "o que já sei sobre este arquivo?" antes
de editá-lo. Notas confirmadas por tarefa viram `star` na saída.

### Nível 3 — escopar o handoff (D143)

```bash
kd rewind --scope epic_01abc
kd rewind --tag retry --anchor src/gateway.rs
kd rewind --around fact_01abc --depth 2
```

Os filtros de corpus restringem o que entra no manifest, além do `--scope` (épico).

### Nível 4 — janela temporal e retomada

```bash
kd rewind --since 2026-01-01 --until 2026-06-01
kd rewind --resume <context_id>          # retoma 1:1
```

Cada execução gera um `context_id` retomável; `--resume` reabre **exatamente** aquele contexto.

## Referência de flags

| Flag | Efeito |
|---|---|
| `--scope <C>` | Restringe a um épico |
| `--files <PATH>...` | Working set por arquivos |
| `--budget <N>` | Orçamento de tokens (default 4000; `ceil(len/4)`) |
| `--since <TS>` / `--until <TS>` | Janela de criação |
| `--resume <CONTEXT_ID>` | Retoma um contexto 1:1 |
| `--type`/`--class`/`--tag`/`--anchor` | Filtros de corpus (D143) |
| `--around <ID>` `--depth <N>` | Vizinhança de uma nota |

## Resultados

- Texto: manifest + `next:`/`fresh:`.
- `--json`: `{context_id, items[], dropped, embeddings_pending}`.
- O excedente do orçamento vira `dropped`; `K` de `next:` deriva do orçamento.
- Escreve `.idx/contexts/<context_id>.json` (derivado, fora do git).

## Quando (não) usar

- **Use** no começo e no fim de sessão, para situar/retomar.
- **Não use** como busca dirigida — para isso há [`kd ask`](04-ask.md).

## Próximo passo

➡️ [`kd maintenance`](09-maintenance.md) · [`kd sync`](12-sync.md)
