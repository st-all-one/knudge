# 08 — MCP (usar de dentro de um agente)

O `knudge-mcp` expõe os gatilhos de memória por **MCP** (Model Context Protocol), via
**JSON-RPC 2.0 sobre stdio** — sem servidor de rede. É o que permite um agente (Claude, Cursor,
Codex, pi) consultar o knudge automaticamente.

## Configurar no cliente

```bash
kd self setup claude    # Claude Desktop / Claude Code
kd self setup cursor
kd self setup codex
kd self setup pi
```

O comando instala a *recipe* do cliente apontando para o binário `knudge-mcp`. Você também pode
rodar direto:

```bash
knudge-mcp --stdio
```

## Tools expostas

| Tool | Quando dispara | O que devolve |
|---|---|---|
| `knudge_pre_write` | Antes de gravar conhecimento | Quase-duplicatas (evita duplicata) |
| `knudge_pre_edit` | Antes de editar um arquivo | Working set das notas que tocam o arquivo |
| `knudge_session_end` | Fim de sessão | Sugestões do que registrar |
| `knudge_status` | Sob demanda | Estado da memória/fila |

Os *hints* são **ponteiros** (`id + statement + score`) — o corpo da nota fica no `kd`, nunca no
contexto do modelo. Isso mantém o contexto enxuto.

## Configuração relacionada

```bash
kd config set mcp.hints_cap 3        # quantos hints por gatilho
```

## Fluxo típico

1. O agente vai **gravar** algo → `knudge_pre_write` avisa se já existe (dedup).
2. O agente vai **editar** `src/x.rs` → `knudge_pre_edit` devolve o que já se sabe daquele arquivo.
3. Ao **encerrar a sessão** → `knudge_session_end` sugere o que virar nota.
4. O agente pode **consultar** o estado com `knudge_status`.

## Próximo passo

➡️ Volte ao [índice](README.md) ou veja a [referência completa da CLI](../plan/implementation/16_cli_surface.md).
