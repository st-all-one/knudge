# Documentação do knudge

**knudge** (`kd`) é uma memória por projeto para agentes de IA: notas Markdown versionadas, um
índice derivado reconstruível e uma CLI otimizada para consumir pouco contexto. Estas páginas são
para **usar** o knudge no dia a dia; a referência interna do código fica em
[`plan/`](../plan/), [`ARCHITECTURE.md`](../ARCHITECTURE.md) e [`AGENTS.md`](../AGENTS.md).

## Comece aqui

1. **[Quickstart](00-quickstart.md)** — instalação, desinstalação, primeiros passos e o ciclo.
2. **[Conceitos e arquitetura](01-conceitos.md)** — modelo de dados, layout, decisões de projeto.

## Comandos (um guia por comando)

| Comando | Para quê |
|---|---|
| [`kd prime`](02-prime.md) | O protocolo estático ("help da IA") — byte-idêntico por versão |
| [`kd init`](03-init.md) | Funda `.knudge/` e emite o bloco do `AGENTS.md` |
| [`kd ask`](04-ask.md) | Toda a **pesquisa**: recall, get e expand |
| [`kd write`](05-write.md) | Toda a **escrita**: create, update e arestas |
| [`kd task`](06-task.md) | Planejar e executar trabalho (épico → issue → tarefa) |
| [`kd knowledge`](07-knowledge.md) | Mapa, ranking e tags |
| [`kd rewind`](08-rewind.md) | Estado/handoff ponto-no-tempo |
| [`kd doctor`](09-maintenance.md) | Saúde: 13 checks + auditoria (`--fix`, `--explain`) |
| [`kd drain`](09-maintenance.md) | Fila de embeddings: `--status` / `--digest [--force]` |
| [`kd maintenance`](09-maintenance.md) | Propostas (`compact`/`learn`/`prune`) e worker de embeddings |
| [`kd config`](10-config.md) | Configuração em dois níveis |
| [`kd forget`](11-forget.md) | Soft-delete e purga |
| [`kd sync`](12-sync.md) | Commit de `notas/` + `eventos/` |
| [`kd self`](13-self.md) | Setup de cliente, completions, upgrade, versão |

Guias transversais:

| Guia | Para quê |
|---|---|
| [MCP](14-mcp.md) | Usar de dentro de um agente (Claude/Cursor/Codex/pi) |
| [Embeddings](15-embeddings.md) | Ligar a busca semântica (opcional) |
| [Troubleshooting](troubleshooting.md) | Problemas comuns e como resolver |

## O ciclo central

```
kd ask → kd write → kd task → kd sync
(buscar)  (gravar)   (executar) (commit)
```

**Sempre busque antes de gravar.** O `write` faz dedup contra o que já existe: score `< 0.75`
cria, `0.75–0.92` faz merge, `≥ 0.92` rejeita.

## Convenções da CLI (válidas em todos os comandos)

- **`stdout` = dados** (pipe/`--json`); **`stderr` = logs**. Nunca se misturam.
- **`--json`** devolve `{success, command, data?, error?, warnings?}` — contrato de máquina.
- **Exit codes:** `2` invalid, `3` not_found, `4` conflict, `5` io, `6` timeout, `7` config,
  `8` schema, `70` internal (`101` é reservado a panic).
- **`--brief`** encurta a saída (gasta menos contexto do modelo).
- **Posicional = conteúdo:** em `write`/`task new` é o **corpo**; em `ask` é a **consulta**.
  `-` lê de stdin; sem posicional com stdin não-TTY, também lê de stdin (heredoc/pipe).
- **`--params '<json>'`** envia o objeto completo de uma vez (`-` lê de stdin).
- **`kd` sozinho = `kd help`; protocolo: `kd prime`.**
