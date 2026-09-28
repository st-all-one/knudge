# knudge-mcp — servidor MCP

Servidor MCP **local e reativo a comportamento** (D68), sem servidor de rede. O núcleo é o
**motor de gatilhos** (E12-T03); a E14 acrescentou o **transporte JSON-RPC 2.0 sobre stdio**.
Expõe o knudge a editores/agentes como *tools* que devolvem **ponteiros** — nunca o corpo de uma
nota — para manter o contexto curto.

**Binário puro:** o pacote expõe só `[[bin]]` (`knudge-mcp`), **sem `[lib]`**. A única biblioteca é
`knudge-core` (interna, compartilhada com o `kd`). Dependência externa: só `serde_json`.

## Gatilhos e tools

Três **gatilhos** (comportamento) + uma tool de **status** (4 tools ao todo). O hint é sempre
ponteiro (`id + statement + score`), com cap configurável (default 3) e modo observação.

| Gatilho | Tool | Quando dispara | Fonte |
|---|---|---|---|
| `pre_write` | `knudge_pre_write` | antes de gravar (quase-duplicados). | candidatos do dedup |
| `pre_edit` | `knudge_pre_edit` | antes de editar arquivo (working set). | manifest do `rewind --files` |
| `session_end` | `knudge_session_end` | fim de sessão (`learn`). | write-gap + propostas |
| — | `knudge_status` | consulta o estado do motor. | `HintEngine` |

`HintKind`: `duplicate`, `context`, `write_gap`, `missing_link`, `merge`.

## Estrutura

| Arquivo | Papel |
|---|---|
| `main.rs` | Binário `knudge-mcp` (`--stdio` padrão, `--help`, `--version`); exit codes. |
| `triggers.rs` | `HintEngine` **puro**: os 3 gatilhos, `Trigger`/`HintKind`/`Hint`, cap, dedup por sessão e modo observação. |
| `tools.rs` | Catálogo das tools (`PRE_WRITE`/`PRE_EDIT`/`SESSION_END`/`STATUS`), `list()` e `call()`. |
| `server.rs` | `McpServer` — dispatcher puro: handshake, `ping`, `tools/list`, `tools/call`. |
| `jsonrpc.rs` | Codec JSON-RPC 2.0 (parse/emit, códigos canônicos), sem I/O. |
| `protocol.rs` | `PROTOCOL_VERSION`/`SUPPORTED_VERSIONS` e negociação de `protocolVersion`. |
| `transport.rs` | `serve`/`respond` — loop stdio, uma linha JSON por mensagem; `EPIPE`/EOF → exit 0. |
| `config.rs` | `McpConfig` — lê `mcp.hints_cap`/`mcp.observation_mode`/`mcp.observation_sessions`. |

## Contrato

- **Framing:** uma **linha JSON por mensagem** (sem `Content-Length`); stdout só tem protocolo.
- `initialize` → `{protocolVersion, capabilities.tools, serverInfo}`. Versões suportadas:
  `2025-06-18` (atual), `2025-03-26`, `2024-11-05`; desconhecida cai na atual.
- Notificações (`notifications/*`) não geram resposta; parse inválido responde com `id: null`.
- `tools/call` **nunca** derruba o servidor: argumento inválido vira `isError: true`.
- `EPIPE`/EOF no transporte → exit 0 (D73/D71).

## Configuração (`.knudge/config.toml`)

| Chave | Default | Efeito |
|---|---|---|
| `mcp.hints_cap` | 3 | máximo de hints por gatilho. |
| `mcp.observation_mode` | `true` | só observa (não sugere) nas primeiras sessões. |
| `mcp.observation_sessions` | — | quantas sessões ficam em observação. |

## Testes

Em `crates/knudge-mcp/tests/`: `jsonrpc.rs` (codec), `server.rs` (dispatcher), `tools.rs`
(tools), `transport.rs` (stdio/EPIPE), `triggers.rs` (motor). `HintEngine` é puro — sem
terminal/FS/relógio —, então os testes são determinísticos.

## Onde mexer

| Quero… | Vá para |
|---|---|
| adicionar/alterar um gatilho | `triggers.rs` |
| adicionar uma tool | `tools.rs` + `server.rs` |
| handshake/versões MCP | `protocol.rs` |
| framing/transporte | `transport.rs` |
| chaves de config do MCP | `config.rs` |
