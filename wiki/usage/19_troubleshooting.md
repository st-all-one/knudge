# 19 · Troubleshooting

Problemas comuns e como resolver. Se nada aqui ajudar, rode [`kd doctor`](10_doctor.md) e leia o
[`plan/`](../../plan) — a referência interna é detalhada.

---

## Instalação e PATH

| Sintoma | Causa | Ação |
|---|---|---|
| `kd: command not found` | PATH não atualizado | `export PATH="$HOME/.local/bin:$PATH"` e abra um shell novo |
| `permission denied` no install | destino sem permissão | `INSTALL_DIR=~/.local/bin` |
| `checksum mismatch` | download corrompido | rode de novo; confira `VERSION` e rede |
| `rustc` antigo | MSRV | Rust **1.97+** (`rustup update`) |
| completions não funcionam | shell não recarregado | abra um shell novo ou `source` o arquivo |

```bash
# 1. o binário responde?
kd self version

# 2. os dois binários estão no PATH?
which kd knudge-mcp

# 3. qual versão instalada?
kd --version
```

---

## Corpus e integridade

| Sintoma | Causa | Ação |
|---|---|---|
| `nota ausente: <id>` | layout antigo / id errado | `kd doctor --fix` |
| `tipo desconhecido` | corpus legado | `kd doctor --fix` |
| erro de schema/`body_hash` | nota editada à mão | `kd doctor --fix` |
| índice divergente | `.idx/` corrompido | `kd doctor --fix` |
| conflito de merge | mesma afirmação, corpos divergentes | `kd doctor` aponta; resolva/substitua |

```bash
# 1. o que está errado?
kd doctor

# 2. reparar o reversível
kd doctor --fix

# 3. entender cada achado
kd doctor --explain
```

> **As notas são a verdade.** Nunca edite à mão um arquivo em `.knudge/notas/` — use
> [`kd write`](06_write.md) / [`kd task`](07_task.md). O `doctor --fix` é reversível e **não apaga**
> notas.

---

## Busca (`kd ask`)

| Sintoma | Causa | Ação |
|---|---|---|
| `[no_results]` | nada casou | ajuste a consulta; tente `--anchor`; confirme `--with-task` |
| Tarefa não aparece | o `ask` é só conhecimento | use `--with-task` ou [`kd task list`](07_task.md) |
| Poucos/muitos resultados | `--limit` | `--limit N` ou `kd config set --key recall.default_limit --value N` |
| Sem `why=semantic` | embeddings desligados | veja [Embeddings](18_embeddings.md) |

```bash
# 1. sem resultado? tente por arquivo
kd ask --anchor src/gateway.rs

# 2. incluir trabalho
kd ask "cache" --with-task

# 3. confirmar o canal semântico
kd config get --key recall.semantic
```

---

## Escrita (`kd write`)

| Sintoma | Causa | Ação |
|---|---|---|
| `rejected` | duplicata | atualize a existente com `--update <ID>` |
| `merged` | quase-duplicata | revise a nota resultante |
| `--type task` rejeitado | trabalho não é conhecimento | use [`kd task new`](07_task.md) |
| chave de rascunho desconhecida | `--params`/`--batch` usam chaves canônicas | use `statement`, não `summary` |

```bash
# 1. revisar antes de gravar
kd ask "rascunho da nota" --brief

# 2. atualizar em vez de duplicar
kd write --update <ID> --summary "texto corrigido"

# 3. validar um lote sem gravar
kd write --batch lote.jsonl --dry-run
```

---

## Tarefas (`kd task`)

| Sintoma | Causa | Ação |
|---|---|---|
| `task list` sem filtro | exige escopo | passe um filtro (`--ready`, `--tag`, …) ou `--universe` |
| `close` recusa | falta evidência | `--outcome success` (+ `--note`) |
| `graph --program` vazio | épico sem âncora | ancore o épico em `plan/*.md` |
| hierarquia inválida | pai de nível errado | o pai precisa ser um nível acima |

```bash
# 1. listar o que está pronto
kd task list --ready

# 2. fechar com evidência
kd task close --id task_01abc --outcome success --note "testes verdes"

# 3. ver o contexto de um item
kd task show --id task_01abc
```

---

## Embeddings

Resumo; o guia completo está em [Embeddings](18_embeddings.md#7-solução-de-problemas).

```bash
# 1. saúde do worker
kd drain service --status

# 2. fila pendente
kd drain --status

# 3. tentar de novo
kd drain --digest

# 4. desligar o canal
kd config set --key recall.semantic --value false
```

---

## MCP

| Sintoma | Causa | Ação |
|---|---|---|
| Cliente não vê as tools | recipe não apontada | `kd self setup <cliente>` e aponte o arquivo |
| Hints demais/poucos | `mcp.hints_cap` | `kd config set --key mcp.hints_cap --value N` |
| Sem hints no começo | modo observação | aguarde as sessões ou `mcp.observation_mode=false` |

```bash
# 1. gerar a recipe
kd self setup claude

# 2. ajustar o limite de hints
kd config set --key mcp.hints_cap --value 5

# 3. sair do modo observação
kd config set --key mcp.observation_mode --value false
```

---

## Códigos de saída e `--json`

| Código | Significado |
|---|---|
| `0` | sucesso (inclui busca vazia e pipe fechado) |
| `2` | uso/argumento inválido |
| `3` | id/chave não encontrado |
| `4` | conflito |
| `5` | erro de arquivo |
| `6` | tempo esgotado |
| `7` | configuração inválida |
| `8` | nota/schema inválido |
| `70` | erro interno |

```bash
# 1. o stdout é JSON puro
kd --json ask "cache" 2>/dev/null | jq .

# 2. ver os avisos de degradação
kd --json ask "cache" | jq '.warnings'

# 3. confirmar o código de saída
kd ask ""; echo "exit=$?"
```

- **Pipe fechado (EPIPE)** → exit **0** (não é erro).
- `warnings[]` indica degradação graciosa (um recurso opcional falhou e o resultado veio parcial).

---

## Ainda travado?

```bash
# 1. releia o protocolo
kd prime

# 2. diagnóstico do corpus
kd doctor

# 3. onde eu estava?
kd rewind --budget 2000
```

A referência completa da superfície está em
[`plan/implementation/16_cli_surface.md`](../../plan/implementation/16_cli_surface.md).

---

## Veja também

➡️ [Quickstart](00_quickstart.md) · [`kd doctor`](10_doctor.md) · [Embeddings](18_embeddings.md)
