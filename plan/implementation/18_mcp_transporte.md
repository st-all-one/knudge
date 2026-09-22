# E14 — MCP: transporte JSON-RPC (stdio) e superfície de tools

> **Fase 4 (continuação).** Fecha o que ficou adiado em E12-T03: o **motor de gatilhos** já
> existe e é testado, mas falta o **transporte** que o cliente MCP consome. Aqui entra o
> JSON-RPC 2.0 sobre stdio e a exposição dos 3 gatilhos como **tools** — sempre **ponteiros**,
> cap 3, dedup por sessão e modo observação.
>
> **Decisões:** D68, D71, D72, D73.
> **Políticas:** R12, R20, R21, R22, R30, R33 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Um servidor MCP **local, reativo e sem servidor de rede**: fala JSON-RPC 2.0 delimitado por
linha em stdin/stdout, negocia a versão do protocolo, lista e executa as tools do knudge.
Nenhum hint carrega corpo de nota; nenhum log contamina stdout.

## Pré-requisitos

E12 (motor de gatilhos), E13 (golden/testes).

## Tarefas

### E14-T01 ☑ Codec JSON-RPC 2.0 (puro)
- **Objetivo:** decodificar requisição/notificação (`{jsonrpc,id?,method,params?}`) e emitir
  resposta/erro (`result` | `error{code,message,data?}`) sem I/O, com os códigos canônicos
  (`-32700` parse, `-32600` invalid request, `-32601` method not found, `-32602` invalid
  params, `-32603` internal).
- **Entregáveis:** `src/jsonrpc.rs` (`Id`, `Request`, `RpcError`, `parse`, `result`, `error`).
- **Decisões:** D68, D72. **Políticas:** R30.
- **Aceite:** proptest/round-trip de `Id`; id ausente vira notificação; `jsonrpc` errado e
  `method` ausente rejeitados com o código certo.

### E14-T02 ☑ Handshake e negociação de protocolo
- **Objetivo:** `initialize` (negocia `protocolVersion` entre as suportadas; devolve
  `capabilities.tools` e `serverInfo`), `notifications/initialized`, `ping` e `notifications/*`
  sem resposta.
- **Entregáveis:** `src/protocol.rs` (versões, nome) + ramo de handshake em `src/server.rs`.
- **Decisões:** D68.
- **Aceite:** versão pedida suportada é ecoada; versão desconhecida cai na mais nova; `ping`
  responde `{}`; notificação não gera resposta.

### E14-T03 ☑ Tools dos 3 gatilhos
- **Objetivo:** `tools/list` (com `inputSchema`) e `tools/call` para `knudge_pre_write`,
  `knudge_pre_edit`, `knudge_session_end` e `knudge_status`, delegando ao `HintEngine`.
- **Entregáveis:** `src/tools.rs` (definições + parsing de `arguments` → tipos do core).
- **Decisões:** D68. **Políticas:** R33.
- **Aceite:** hint é **ponteiro** (`id`+`statement`+`score`), cap respeitado, argumentos
  inválidos viram `isError` (não derrubam o servidor); `knudge_session_end` fecha a sessão.

### E14-T04 ☑ Transporte stdio
- **Objetivo:** loop bloqueante lendo **uma linha JSON por mensagem** de stdin e escrevendo a
  resposta em stdout; `EPIPE`/EOF termina com exit 0; linha vazia ignorada; nenhuma escrita de
  log em stdout.
- **Entregáveis:** `src/main.rs` + `src/transport.rs`.
- **Decisões:** D71, D73. **Políticas:** R20, R21.
- **Aceite:** `printf '%s\n' '{...}' | knudge-mcp` responde uma linha JSON; pipe fechado sai 0;
  `2>/dev/null` continua JSON válido.

### E14-T05 ☑ Binário, config e recipe
- **Objetivo:** binário `knudge-mcp` lê `mcp.hints_cap` e `mcp.observation_mode`/`mcp.observation_sessions`
  do `.knudge/config.toml` (defaults quando ausente) e `kd self setup` passa a apontar o
  comando MCP no recipe.
- **Entregáveis:** `[[bin]]`; leitura de config via portas; recipe com bloco `mcp`.
- **Decisões:** D68, D69. **Políticas:** R43 (sem dep nova fora do workspace).
- **Aceite:** cap do config reflete em `tools/call`; fora de projeto usa defaults; recipe contém
  `knudge-mcp --stdio`.

### E14-T06 ☑ Testes, docs e matriz
- **Objetivo:** unidade do codec/protocolo/tools + integração do binário (spawn, handshake,
  tools/list, tools/call) e docs.
- **Entregáveis:** `src/tests/{mod,jsonrpc,server,tools}.rs`, `tests/stdio.rs`, `MODULE.md`,
  linha em [`17_matriz_aceitacao.md`](17_matriz_aceitacao.md).
- **Decisões:** D76.
- **Aceite:** `make check` verde; matriz atualizada; `DIVERGENCES.md` ganha a linha de framing
  (linha a linha, sem `Content-Length`).

## Definition of Done

- [x] Codec JSON-RPC puro com códigos canônicos.
- [x] Handshake, `ping` e notificações corretos.
- [x] Tools dos 3 gatilhos (+ `status`) expostas e testadas.
- [x] Transporte stdio com EPIPE→0 e stdout limpo.
- [x] Binário configurável e recipe atualizado.
- [x] `make check` verde; matriz e docs atualizados.

## Não-objetivos

- Servidor de rede (HTTP/SSE/WebSocket) — recusado em `05`, seção 3 (D68: MCP local).
- `resources`/`prompts` do MCP: os hints são tools; conteúdo fica no `kd`.
- FFI/WASM (D68).

## Entregue (E14)

- **T01** — `src/jsonrpc.rs`: `Request`/`Id`/`RpcError`, `parse` (JSON inválido, versão errada,
  `method` ausente e `id` de tipo inválido) e emissores `result`/`error`; `RpcError: Error`
  para uso com `?` nos testes.
- **T02** — `src/protocol.rs` (`SERVER_NAME`, `PROTOCOL_VERSION`, `negotiate`) + handshake em
  `src/server.rs`; `notifications/initialized` liga o flag sem responder.
- **T03** — `src/tools.rs`: `knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end` e
  `knudge_status`, com `inputSchema`; argumentos inválidos viram `isError` (o servidor não cai).
- **T04** — `src/transport.rs`: uma linha JSON por mensagem; linha vazia ignorada; `EPIPE`/EOF
  → `true` (exit 0); parse inválido responde com `id: null`.
- **T05** — binário `knudge-mcp` (`--stdio`, `--help`, `--version`); `src/config.rs` lê
  `mcp.hints_cap`, `mcp.observation_mode` e `mcp.observation_sessions` (nova chave, default 3);
  `kd self setup` passou a incluir o bloco `mcp`.
- **T06** — 37 testes de unidade + 2 de integração (`tests/stdio.rs`, handshake e `--help`);
  `MODULE.md` e a linha MCP em [`17_matriz_aceitacao.md`](17_matriz_aceitacao.md); framing
  registrado em [`DIVERGENCES.md`](../../DIVERGENCES.md).
