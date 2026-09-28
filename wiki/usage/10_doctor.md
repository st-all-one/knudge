# 10 · `kd doctor` — a saúde do corpus

## Para que serve

Diagnostica o corpus e repara o que for reversível. Ele roda uma bateria de verificações
(integridade, âncoras, ciclos, duplicatas, configuração, índice derivado, embeddings…) e, com
`--explain`, mostra o detalhe de cada achado.

O `doctor` **nunca apaga notas**: ele corrige o que pode e reporta o resto.

## Quando usar

- **Use** periodicamente, depois de um `git pull`, ou quando algo parecer estranho.
- **Use `--fix`** para reparar o que é reversível (layout legado, índice divergente, âncoras
  quebradas, locks velhos).
- **Use `--explain`** para entender exatamente o que está errado antes de agir.

## Sintaxe

```
kd doctor [--fix] [--explain] [--json]
```

## Exemplos

### 1. Ver o relatório

```bash
kd doctor
```

Saída (uma linha por verificação):

```
ok schema todas as notas parseiam
ok integrity grafo íntegro
fail derived índice derivado ausente/divergente; `--fix` reconstrói
```

Cada linha é `ok|warn|fail <verificação> <mensagem>`.

### 2. Reparar o reversível

```bash
# 1. reparar
kd doctor --fix

# 2. ver o que foi corrigido
kd --json doctor --fix | jq '.data.fixed'

# 3. reconferir depois
kd doctor
```

O `--fix` migra layout antigo, normaliza escopos obsoletos, recalcula hashes, remove âncoras
quebradas e locks velhos e reconstrói o índice. É **idempotente** (rodar de novo não muda nada).

### 3. Entender cada achado

```bash
# 1. detalhado
kd doctor --explain

# 2. só os detalhes em JSON
kd --json doctor --explain | jq '.data.audit'

# 3. pares duplicados
kd --json doctor | jq '.data.duplicate_pairs'
```

Com `--explain`, cada achado mostra **esperado × encontrado × ação**.

## Flags

| Flag | Efeito |
|---|---|
| `--fix` | Corrige o que for reversível (idempotente) |
| `--explain` | Detalha cada achado |
| `--json` | Envelope de máquina com os detalhes |

## Resultado esperado

- **Texto:** linhas `ok|warn|fail`, o bloco de auditoria e sugestões.
- **`--json`:** `{checks[], healthy, degraded, status, fixed[], audit{...}, suggestions[]}`.
- Alguns achados são **avisos** (ex.: épico sem âncora de programa): aparecem no relatório mas não
  tornam o corpus "não saudável".

## Quando não usar

- Não use o `doctor` para apagar ou fundir notas: ele só **repara** e **reporta**. Para aposentar
  conhecimento, use [`kd forget`](14_forget.md) ou [`kd maintenance prune`](11_maintenance.md).
- Não edite arquivos em `.knudge/notas/` à mão para "consertar": rode `kd doctor --fix`.

## Veja também

➡️ [`kd maintenance`](11_maintenance.md) · [Troubleshooting](19_troubleshooting.md) ·
[`kd sync`](15_sync.md)
