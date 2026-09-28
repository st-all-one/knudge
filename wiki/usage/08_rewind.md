# 08 · `kd rewind` — retomar o contexto

## Para que serve

Reconstrói **onde você estava** no início (ou no fim) de uma sessão, dentro de um orçamento de
contexto. Em vez de buscar um termo, o `rewind` responde: quantas notas há, o que é recente, quais
tarefas estão prontas, o que está desatualizado e o que está pendente de indexação.

É o comando de **retomada**, não de busca dirigida.

## Quando usar

- **Use** no começo da sessão ("o que eu estava fazendo?") e no fim (para deixar o próximo
  contexto pronto).
- **Use `--files`** para saber o que já se sabe sobre os arquivos que você vai abrir.
- **Não use** como busca por assunto — para isso existe [`kd ask`](05_ask.md).

## Sintaxe

```
kd rewind [--scope C] [--files PATH]... [--budget N]
          [--since TS] [--until TS] [--resume CONTEXT_ID]
          [--tag T]... [--anchor P]... [--type T]... [--class C]...
          [--around ID] [--depth N]
```

## Exemplos

### 1. O manifest do projeto

```bash
kd rewind --budget 2000
```

Saída:

```
notes=526 ready=304 blocked=0 containers=59 clean
recent: fact_01ibc4s4 decision_0022xuwr task_01pog2v9
next: task_00aqet3d|B2.1: PHPStan nivel 0 task_005y362k|B2.2: ...
fresh: stale=0 expiring=0 pending=467
```

- `notes`/`ready`/`blocked` — panorama;
- `recent` — as notas mais novas;
- `next` — as próximas tarefas prontas, por impacto;
- `fresh` — o que está desatualizado e a fila de indexação.

### 2. Working set por arquivo

```bash
# 1. um arquivo
kd rewind --files src/gateway.rs

# 2. vários arquivos
kd rewind --files src/gateway.rs --files src/queue.rs

# 3. só as notas ligadas a esses arquivos
kd rewind --files src/gateway.rs --budget 1000
```

### 3. Escopar a retomada

```bash
# 1. um épico
kd rewind --scope epic_01abc

# 2. por tag + âncora
kd rewind --tag retry --anchor src/gateway.rs

# 3. vizinhança de uma nota
kd rewind --around fact_01abc --depth 2
```

### 4. Janela temporal e retomada 1:1

```bash
# 1. intervalo
kd rewind --since 2026-01-01 --until 2026-06-01

# 2. retomar exatamente um contexto anterior
kd rewind --resume <context_id>

# 3. JSON com o id para reusar
kd --json rewind --budget 2000 | jq -r '.data.context_id'
```

## Flags

| Flag | Efeito |
|---|---|
| `--scope <C>` | Restringe a um épico |
| `--files <PATH>...` | Working set por arquivos (repetível; aceita vírgula) |
| `--budget <N>` | Orçamento de contexto |
| `--since <TS>` / `--until <TS>` | Janela de criação |
| `--resume <CONTEXT_ID>` | Retoma um contexto 1:1 |
| `--type`/`--class`/`--tag`/`--anchor` | Filtros de corpus: tipo (`fact`/`decision`/`question`/`task`/`def`/`error`/`snippet`/`link`/`meta`/`risk`), classificação (`foundational`/`tactical`/`observational`), tag, âncora (repetíveis; aceitam vírgula) |
| `--around <ID>` / `--depth <N>` | Vizinhança de uma nota |

## Resultado esperado

- **Texto:** o manifest + `next:` + `fresh:`.
- **`--json`:** `{context_id, items[], dropped, embeddings_pending, flow}`.
- O que excede o orçamento aparece como `dropped`.
- Notas fundamentais e decisões **anexam o corpo** à linha — o "porquê" chega junto do resumo.

## Quando não usar

- Para buscar um assunto → [`kd ask`](05_ask.md).
- Para listar trabalho → [`kd task list`](07_task.md).

## Veja também

➡️ [`kd task`](07_task.md) · [`kd sync`](15_sync.md) · [`kd maintenance`](11_maintenance.md)
