# knudge-mcp — servidor MCP

Servidor MCP **reativo a comportamento** (E12-T03), sem proatividade por iniciativa própria.
Três gatilhos: pré-`write` (quase-duplicados), pré-edição de arquivo (`kd rewind --files`) e
fim de sessão (`kd maintenance learn`). O hint é sempre **ponteiro** (`id + statement + score`),
com cap 3 e modo observação.

## Estrutura

| Arquivo | Papel |
|---|---|
| `triggers.rs` | `HintEngine` puro: os 3 gatilhos, cap, dedup por sessão e modo observação. |
| `lib.rs` | `SERVER_NAME` e re-exports. |

O **transporte JSON-RPC** (stdio) fica na E13; por ora o crate entrega o motor de gatilhos
testável, sem I/O.
