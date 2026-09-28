# 17 · MCP — usar de dentro de um agente

## O que é

O binário **`knudge-mcp`** expõe a memória do knudge por **MCP** (Model Context Protocol) sobre
**stdio** — sem servidor de rede. É o que permite um agente (Claude, Cursor, Codex, pi) consultar o
knudge **automaticamente**, sem você colar comandos.

O MCP não inventa uma API nova: ele dispara os mesmos gatilhos de memória que a CLI oferece.

## Quando usar

- **Use** quando o agente suporta MCP e você quer lembretes automáticos de dedup e contexto.
- **Não use** como substituto da CLI: o MCP cobre poucos gatilhos; a superfície completa está no
  `kd` (e o protocolo, no [`kd prime`](04_prime.md)).

## Rodar o servidor

```bash
# 1. transporte stdio (padrão)
knudge-mcp

# 2. explícito
knudge-mcp --stdio

# 3. ajuda/versão
knudge-mcp --help
knudge-mcp --version
```

O transporte é **uma linha JSON por mensagem** em stdin/stdout. O servidor lê a configuração do
projeto a partir do diretório atual.

## Configurar no cliente

```bash
# 1. gravar a recipe do cliente
kd self setup claude

# 2. conferir o arquivo gerado
cat .knudge/setup/claude.json

# 3. aponte o cliente para `knudge-mcp` (ou registre o comando manualmente)
```

Outros clientes: `kd self setup cursor`, `kd self setup codex`, `kd self setup pi`.

## Tools expostas

| Tool | Quando dispara | O que devolve |
|---|---|---|
| `knudge_pre_write` | Antes de gravar conhecimento | Quase-duplicatas (ponteiros) |
| `knudge_pre_edit` | Antes de editar arquivos | O que já se sabe dos arquivos |
| `knudge_session_end` | Fim de sessão sem writes | Sugestões do que registrar |
| `knudge_status` | Sob demanda | Estado do motor de hints |

### Exemplos de uso (conceitual)

```jsonc
// 1. antes de gravar
{ "candidates": [{ "id": "x", "statement": "rate limit", "score": 0.8 }] }

// 2. antes de editar
{ "items": [{ "id": "y", "statement": "cache LRU", "score": 0.6 }] }

// 3. fim de sessão
{ "writes": 0, "proposals": [{ "kind": "create_note", "ids": ["z"] }] }
```

## Hints são ponteiros, não corpos

Os hints devolvidos pelo MCP são **ponteiros** (`id` + afirmação + score). O corpo da nota fica no
`kd`, nunca no contexto do modelo — isso mantém o contexto enxuto. Quando o agente precisa do
corpo, ele pede [`kd ask --id <ID> --full-content`](05_ask.md).

## Configuração relacionada

```bash
# 1. quantos hints por vez
kd config set --key mcp.hints_cap --value 3

# 2. desligar o modo observação (passa a emitir hints já)
kd config set --key mcp.observation_mode --value false

# 3. número de sessões de observação
kd config set --key mcp.observation_sessions --value 3
```

No **modo observação** (default), o MCP aprende por algumas sessões sem interferir; depois passa a
emitir hints.

## Fluxo típico

1. O agente vai **gravar** algo → `knudge_pre_write` avisa se já existe.
2. O agente vai **editar** `src/x.rs` → `knudge_pre_edit` devolve o que já se sabe do arquivo.
3. Ao **encerrar a sessão** → `knudge_session_end` sugere o que virar nota.
4. O agente pode **consultar** o estado com `knudge_status`.

## Resultado esperado

- O servidor fica ativo enquanto o cliente o mantém aberto.
- Sem MCP, você continua usando a CLI normalmente — nada muda no corpus.

## Veja também

➡️ [`kd prime`](04_prime.md) · [`kd self`](16_self.md) · [Troubleshooting](19_troubleshooting.md)
