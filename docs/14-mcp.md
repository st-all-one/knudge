# MCP — usar de dentro de um agente

## O que é

O binário **`knudge-mcp`** expõe a memória do knudge por **MCP** (Model Context Protocol) via
**JSON-RPC 2.0 sobre stdio** — sem servidor de rede. É o que permite um agente (Claude, Cursor,
Codex, pi) consultar o knudge **automaticamente**, sem colar comandos.

O MCP não inventa uma API nova: ele apenas dispara os mesmos gatilhos de memória que a CLI oferece.

## Em 30 segundos

```bash
kd self setup claude        # grava a recipe do cliente em .knudge/setup/claude.json
knudge-mcp --stdio          # roda o servidor direto
```

## Rodar o servidor

```bash
knudge-mcp            # transporte stdio (padrão)
knudge-mcp --stdio    # idem, explícito
knudge-mcp --help
knudge-mcp --version
```

O transporte é **uma linha JSON por mensagem** em stdin/stdout. O servidor lê a config efetiva do
projeto (`mcp.*`) a partir do diretório atual.

## Configurar no cliente

```bash
kd self setup claude    # Claude Desktop / Claude Code
kd self setup cursor
kd self setup codex
kd self setup pi
```

O comando grava `.knudge/setup/<cliente>.json` apontando para o binário `knudge-mcp`. Aponte o
cliente para esse arquivo (ou registre o comando manualmente com `command: "knudge-mcp"`).

## Tools expostas

| Tool | Quando dispara | O que devolve |
|---|---|---|
| `knudge_pre_write` | Antes de gravar conhecimento | Quase-duplicatas (ponteiros `id`/`statement`/`score`) |
| `knudge_pre_edit` | Antes de editar arquivos | Working set das notas que tocam o arquivo |
| `knudge_session_end` | Fim de sessão sem writes | Sugestões do que registrar (`learn`) |
| `knudge_status` | Sob demanda | Estado do motor de hints |

### `knudge_pre_write`

Entrada: `{ "candidates": [{ "id": string, "statement"?: string, "score": number }] }`.
Devolve os candidatos que passam do limiar de dedup — o agente evita gravar duplicata.

### `knudge_pre_edit`

Entrada: `{ "items": [{ "id": string, "statement"?: string, "score": number }] }`.
Devolve o contexto do working set (o que já se sabe dos arquivos que serão editados).

### `knudge_session_end`

Entrada: `{ "writes": integer, "proposals": [{ "kind": string, "ids": string[], "why"?: string, "score"?: number }] }`.
Devolve propostas de `learn` quando a sessão termina sem gravar.

### `knudge_status`

Entrada: `{}`. Devolve o estado do motor (modo observação, contadores).

## Hints são ponteiros, não corpos

Os hints devolvidos pelo MCP são **ponteiros** (`id + statement + score`). O corpo da nota fica no
`kd`, nunca no contexto do modelo — isso mantém o contexto enxuto. Quando o agente precisa do
corpo, ele pede `kd ask --id <ID> --full-content`.

## Configuração relacionada

```bash
kd config set --key mcp.hints_cap --value 3
kd config set --key mcp.observation_mode --value true
kd config set --key mcp.observation_sessions --value 3
```

No **modo observação** (default), o MCP aprende por algumas sessões sem interferir; depois passa a
emitir hints.

## Fluxo típico

1. O agente vai **gravar** algo → `knudge_pre_write` avisa se já existe (dedup).
2. O agente vai **editar** `src/x.rs` → `knudge_pre_edit` devolve o que já se sabe daquele arquivo.
3. Ao **encerrar a sessão** → `knudge_session_end` sugere o que virar nota.
4. O agente pode **consultar** o estado com `knudge_status`.

## Quando (não) usar

- **Use** quando o agente suporta MCP e você quer gatilhos automáticos.
- **Não use** como substituto da CLI: o MCP cobre poucos gatilhos; a superfície completa está no
  `kd` (e o protocolo, no [`kd prime`](02-prime.md)).

## Próximo passo

➡️ [Troubleshooting](troubleshooting.md) · [Conceitos](01-conceitos.md)
