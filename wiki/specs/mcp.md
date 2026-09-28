# MCP — protocolo e gatilhos

O `knudge-mcp` é um servidor MCP **local e reativo a comportamento** (D68), sem servidor de rede:
motor de gatilhos (E12-T03) + transporte **JSON-RPC 2.0 sobre stdio** (E14). Os hints são
**ponteiros** (`id + statement + score`), nunca o corpo.

- Código: `crates/knudge-mcp/src/`
- Decisões: D68, E12-T03, E14

## Transporte

- **Framing:** uma **linha JSON por mensagem** (sem `Content-Length`); stdout só tem protocolo.
- `serve`/`respond` (`transport.rs`): loop stdio; `EPIPE`/EOF → **exit 0** (D71/D73).
- `jsonrpc.rs`: codec JSON-RPC 2.0 (parse/emit, códigos canônicos), sem I/O.

## Handshake e versões

- `initialize` → `{protocolVersion, capabilities.tools, serverInfo}`.
- `PROTOCOL_VERSION = "2025-06-18"`; `SUPPORTED_VERSIONS = ["2025-06-18", "2025-03-26",
  "2024-11-05"]`; desconhecida cai na atual (`protocol.rs`).
- Notificações (`notifications/*`) não geram resposta; parse inválido responde com `id: null`.
- `tools/call` **nunca** derruba o servidor: argumento inválido vira `isError: true`.

## Gatilhos e tools

Três gatilhos + uma tool de status (4 tools). `HintEngine` é **puro** (sem terminal/FS/relógio).

| `Trigger` | Tool | Dispara | Fonte |
|---|---|---|---|
| `PreWrite` | `knudge_pre_write` | antes de gravar (quase-duplicados) | candidatos do dedup |
| `PreEdit` | `knudge_pre_edit` | antes de editar arquivo (working set) | manifest do `rewind --files` |
| `SessionEnd` | `knudge_session_end` | fim de sessão (`learn`) | write-gap + propostas |
| — | `knudge_status` | estado do motor | `HintEngine` |

`HintKind` ∈ `duplicate`, `context`, `write_gap`, `missing_link`, `merge`.

## Motor (`HintEngine`)

- `new(cap, observation_sessions)`: cap de hints (default 3) e modo observação.
- Dedup **por sessão**; `is_observing` decide se sugere ou só observa.
- `pre_write`, `pre_edit`, `session_end` produzem `Vec<Hint>` (ponteiros).

## Configuração

`McpConfig` lê de `.knudge/config.toml`:

| Chave | Default | Efeito |
|---|---|---|
| `mcp.hints_cap` | 3 | máximo de hints por gatilho |
| `mcp.observation_mode` | `true` | só observa nas primeiras sessões |
| `mcp.observation_sessions` | — | quantas sessões ficam em observação |

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Entrypoint | `main.rs` |
| Motor de gatilhos | `triggers.rs` |
| Tools | `tools.rs` |
| Dispatcher | `server.rs` |
| Codec JSON-RPC | `jsonrpc.rs` |
| Versões/handshake | `protocol.rs` |
| Transporte stdio | `transport.rs` |
| Config | `config.rs` |

## Testes

`crates/knudge-mcp/tests/` (jsonrpc, server, tools, transport, triggers).
