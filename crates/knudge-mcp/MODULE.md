# knudge-mcp — servidor MCP

**Binário puro:** o pacote expõe só `[[bin]]` (`knudge-mcp`), **sem `[lib]`**. A única
biblioteca é `knudge-core` (interna, compartilhada com o `kd`).

Servidor MCP **local e reativo a comportamento** (D68), sem servidor de rede. O núcleo é o
**motor de gatilhos** (E12-T03); a E14 acrescentou o **transporte JSON-RPC 2.0 sobre stdio**.

Três gatilhos: pré-`write` (quase-duplicados), pré-edição de arquivo (`kd rewind --files`) e
fim de sessão (`kd maintenance learn`). O hint é sempre **ponteiro** (`id + statement + score`),
com cap 3 e modo observação.

## Estrutura

| Arquivo | Papel |
|---|---|
| `triggers.rs` | `HintEngine` puro: os 3 gatilhos, cap, dedup por sessão e modo observação. |
| `jsonrpc.rs` | Codec JSON-RPC 2.0 (parse/emit, códigos canônicos), sem I/O. |
| `protocol.rs` | Nome/versões do MCP e negociação de `protocolVersion`. |
| `tools.rs` | Tools (`knudge_pre_write`/`pre_edit`/`session_end`/`status`) e parsing de argumentos. |
| `server.rs` | Dispatcher puro: handshake, `ping`, `tools/list` e `tools/call`. |
| `transport.rs` | Loop stdio: uma linha JSON por mensagem; `EPIPE`/EOF → exit 0. |
| `config.rs` | Lê `mcp.hints_cap`/`mcp.observation_mode`/`mcp.observation_sessions` de `.knudge/config.toml`. |
| `main.rs` | Binário `knudge-mcp` (`--stdio`, `--help`, `--version`). |

## Contrato

- Framing: **uma linha JSON por mensagem** (sem `Content-Length`); stdout só tem protocolo.
- `initialize` → `{protocolVersion, capabilities.tools, serverInfo}`; versões suportadas
  `2025-06-18`, `2025-03-26`, `2024-11-05`.
- Notificações (`notifications/*`) não geram resposta; parse inválido responde com `id: null`.
- `tools/call` nunca derruba o servidor: argumento inválido vira `isError: true`.
