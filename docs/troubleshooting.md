# Troubleshooting

Problemas comuns e como resolver. Se nada aqui ajudar, rode `kd maintenance doctor --audit` e leia
o [`plan/`](../plan/) — a referência interna é detalhada.

## Instalação e PATH

| Sintoma | Causa | Ação |
|---|---|---|
| `kd: command not found` | PATH não atualizado | `export PATH="$HOME/.local/bin:$PATH"` e reinicie o shell |
| `permission denied` no install | destino sem permissão | use `INSTALL_DIR=~/.local/bin` |
| `rustc` antigo | MSRV | Rust **1.97+** (edição 2024) |
| completions não funcionam | shell não recarregado | reabra o shell ou `source` o arquivo de completion |

```bash
kd self version          # confirma que o binário responde
which kd knudge-mcp      # confirma o PATH
```

## Corpus e integridade

| Sintoma | Causa | Ação |
|---|---|---|
| `nota ausente: <id>` | layout plano legado / id errado | `kd maintenance doctor --fix` |
| `tipo desconhecido` | corpus legado (`type: container`) | `kd maintenance doctor --fix` |
| `schema`/`body_hash` no doctor | nota editada à mão | `kd maintenance doctor --fix` |
| Índice divergente | `.idx/` corrompido | `kd maintenance doctor --fix` (reconstrói) |
| Conflito de merge | mesma `statement`, corpos divergentes | `kd maintenance doctor` aponta; resolva/supersede |

**Notas são a verdade.** Nunca edite à mão um arquivo em `.knudge/notas/` — use `kd write`/
`kd task`. O `doctor --fix` é reversível e **não apaga** notas.

```bash
kd maintenance doctor          # o que está errado?
kd maintenance doctor --fix    # repara o reversível
kd maintenance doctor --audit  # integridade de grafo/arestas
```

## Busca (`kd ask`)

| Sintoma | Causa | Ação |
|---|---|---|
| `[no_results]` | nada casou | ajuste a query; tente `--anchor`; confirme `--with-task` |
| Tarefa não aparece | `ask` é só conhecimento | use `--with-task` ou [`kd task list`](06-task.md) |
| Poucos/ muitos hits | `--limit` | `--limit N` ou `kd config set --key recall.default_limit --value N` |
| `why=semantic` ausente | embeddings desligados | veja [Embeddings](15-embeddings.md) |

## Escrita (`kd write`)

| Sintoma | Causa | Ação |
|---|---|---|
| `rejected` | duplicata (`score ≥ 0.92`) | atualize a existente com `--update <ID>` |
| `merged` | quase-duplicata (`0.75–0.92`) | revise a nota resultante |
| `--type task` rejeitado | trabalho não é conhecimento | use [`kd task new`](06-task.md) |
| chave de rascunho desconhecida | `--params`/`--batch` usam chaves canônicas | use `statement`, não `summary` |

## Tarefas (`kd task`)

| Sintoma | Causa | Ação |
|---|---|---|
| `task list` sem escopo | D144 | passe um filtro (`--ready`, `--tag`, …) ou `--universe` |
| `close` recusa | falta evidência | `--outcome success` (+ `--note`) |
| `graph --program` vazio | épico sem âncora | ancore o épico em `plan/*.md` |
| hierarquia inválida | pai de rank ≥ | o pai precisa ter rank estritamente menor |

## Embeddings

Ver a seção de [solução de problemas](15-embeddings.md#solução-de-problemas). Em resumo:

```bash
kd maintenance watch-service --status   # agendador/servidor/fila
kd knowledge digest --status            # fila pending
kd knowledge digest --drain             # tenta de novo
kd config set --key recall.semantic --value false   # desliga o canal
```

## MCP

| Sintoma | Causa | Ação |
|---|---|---|
| Cliente não vê as tools | recipe não apontada | `kd self setup <cliente>` e aponte o arquivo |
| Hints demais/poucos | `mcp.hints_cap` | `kd config set --key mcp.hints_cap --value N` |
| Sem hints no começo | modo observação | aguarde as sessões ou `mcp.observation_mode=false` |

## Exit codes e `--json`

| Código | Significado |
|---|---|
| `2` | `invalid_input` — argumento/uso inválido |
| `3` | `not_found` — id/chave ausente |
| `4` | `conflict` — id inexistente/`forgotten` no update |
| `5` | `io` — falha de arquivo |
| `6` | `timeout` — provedor/hook demorou |
| `7` | `config` — chave/valor inválido |
| `8` | `schema` — nota/frontmatter inválido |
| `70` | `internal` — bug/estado inesperado |
| `101` | reservado a panic |

- **Pipe fechado (EPIPE)** → exit **0** (não é erro).
- Em `--json`, o stdout é **só** o envelope; nenhum log vaza. Teste:
  `kd … --json 2>/dev/null | jq .`
- `warnings[]` indica degradação graciosa (canal opcional falhou). Com
  `behavior.strict=true`, vira erro.

## Ainda travado?

```bash
kd prime                       # releia o protocolo
kd maintenance doctor --audit  # integridade
kd rewind --budget 2000        # onde eu estava?
```

A referência completa da superfície está em
[`plan/implementation/16_cli_surface.md`](../plan/implementation/16_cli_surface.md).
