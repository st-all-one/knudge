# Guias do knudge

Bem-vindo! Estes guias são para **usar** o knudge no dia a dia. A referência interna do código
fica em [`plan/`](../plan/), [`AGENTS.md`](../AGENTS.md) e [`ARCHITECTURE.md`](../ARCHITECTURE.md).

Comece por:

1. [Instalação e ambiente](01-instalacao.md)
2. [Primeiros passos](02-primeiros-passos.md)

Depois, por grupo de comandos:

| Guia | Para quê |
|---|---|
| [Busca — `kd ask`](03-ask.md) | Achar o que já se sabe (a busca é o coração do knudge) |
| [Escrita — `kd write`](04-write.md) | Registrar fatos, decisões, erros, riscos |
| [Tarefas — `kd task`](05-task.md) | Planejar e executar trabalho (plano → épico → issue → tarefa) |
| [Embeddings](06-embeddings.md) | Ligar a busca semântica (opcional) |
| [Manutenção e handoff](07-manutencao.md) | Saúde, limpeza, `rewind`, `forget`, `config`, `sync` |
| [MCP](08-mcp.md) | Usar de dentro de um agente (Claude/Cursor/Codex/pi) |

## O ciclo central

```
kd ask → kd write → kd task → kd sync
(buscar)  (gravar)   (executar) (commit)
```

**Sempre busque antes de gravar** — o `write` faz dedup (score `< 0.75` cria, `0.75–0.92` faz
merge, `≥ 0.92` rejeita).

## Convenções da CLI

- **`stdout` = dados** (pipe/`--json`); **`stderr` = logs**. Nunca se misturam.
- **`--json`** devolve `{success, command, data?, error?, warnings?}` — contrato de máquina.
- **Exit codes:** `2` invalid, `3` not_found, `4` conflict, `5` io, `6` timeout, `7` config,
  `8` schema, `70` internal (`101` é reservado a panic).
- **`--brief`** encurta a saída (ótimo para gastar menos contexto do modelo).
- **`kd`** sem argumentos = **`kd prime`**: o protocolo estático ("help da IA"), byte-idêntico
  por versão.

## Precisa de ajuda rápida?

```bash
kd prime            # protocolo completo (o "help da IA")
kd prime --long     # + gramática TOON e schema
kd ask --help       # ajuda do clap para um verbo
```
