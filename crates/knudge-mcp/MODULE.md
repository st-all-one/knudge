# knudge-mcp — servidor MCP

Servidor MCP **reativo a comportamento** (E12-T03), sem proatividade por iniciativa própria.
Três gatilhos: pré-`write` (quase-duplicados), pré-edição de arquivo (`kd rewind --files`) e
fim de sessão (`kd maintenance learn`). O hint é sempre **ponteiro** (`id + statement + score`),
com cap 3 e modo observação.

Nesta fase (E01) o crate é apenas um esqueleto.
