# Quickstart

Esta página leva você de zero a uma memória de projeto funcionando. Em cinco minutos você terá
instalado o `kd`, fundado a memória, gravado e buscado a primeira nota, e saberá retomar o
contexto na próxima sessão.

## 1. Instalar

O jeito mais simples (Linux e macOS):

```bash
curl --proto '=https' --tlsv1.2 --show-error --fail \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash
```

Isso instala **dois binários** em `~/.local/bin` (release pré-compilado, verificado por SHA-256):

- **`kd`** — a CLI;
- **`knudge-mcp`** — o servidor MCP (JSON-RPC 2.0 sobre stdio).

Opções úteis:

```bash
# Fixar uma versão
curl ... | VERSION=v0.3.1 bash

# Outro diretório de instalação
curl ... | INSTALL_DIR=/usr/local/bin bash

# Compilar do source (precisa de Rust 1.97+)
./install.sh --from-source
make install
```

**Requisitos:** `git` no projeto onde você vai usar. Embeddings (busca semântica) são
**opcionais** — veja [Embeddings](15-embeddings.md).

O instalador adiciona `~/.local/bin` ao PATH (em `~/.profile`, `~/.bashrc`, `~/.zshrc`) e instala
completions de **bash**, **zsh** e **fish**. Se ele avisar, reinicie o shell ou rode:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Verificar a instalação

```bash
kd self version        # versão do binário
kd prime               # o protocolo ("help da IA")
kd --help              # ajuda geral
```

## 2. Desinstalar

O instalador aceita `--uninstall`, que invalida os binários e as completions **sem tocar** no seu
corpus (`.knudge/`) nem na config global:

```bash
./install.sh --uninstall
```

Se você instalou via `make install`, use `make uninstall` (move os binários para o lixo
recuperável). Para remover o sistema de embeddings (agendador + servidor), use
`kd maintenance watch-service --uninstall` — veja [Manutenção](09-maintenance.md).

> O corpus vive **dentro de cada projeto** (`.knudge/`). Desinstalar o binário não apaga nota
> nenhuma; para esquecer notas, use [`kd forget`](11-forget.md).

## 3. Fundar a memória do projeto

Dentro do seu projeto (na raiz do git):

```bash
kd init
```

O `kd init` cria `.knudge/` (notas, eventos, índice derivado) e escreve um bloco no `AGENTS.md`
do projeto, ensinando o agente a usar o `kd`. A **verdade** são os arquivos Markdown em
`.knudge/notas/<tipo>/`; todo o resto é **derivado** e reconstruível — veja
[Conceitos](01-conceitos.md).

## 4. Buscar antes de gravar

```bash
kd ask "como o gateway limita requisições" --brief
```

A saída no terminal (ou no pipe) é `id|statement|score|why`, um hit por linha. Use `--brief`
para gastar menos contexto.

## 5. Gravar conhecimento

```bash
kd write --summary "Rate limit é 100 rps por chave" --type fact \
  --tag gateway --anchor src/gateway.rs
```

O `write` faz **dedup** contra o que já existe:

| Score do `ask` | Ação do `write` |
|---|---|
| `< 0.75` | Cria nota nova |
| `0.75 – 0.92` | Faz **merge** na nota existente |
| `≥ 0.92` | **Rejeita** (duplicata) |

Tipos de conhecimento: `fact`, `decision`, `error`, `risk`, `question`, `def`, `snippet`,
`link`, `meta`. Para **trabalho**, use [`kd task`](06-task.md) — o `write` rejeita `--type task`.

## 6. Planejar e executar

```bash
kd task new --summary "Sync offline-first" --scope epic
kd task new --summary "Resolver conflito de merge" --scope task --parent <epic>
kd task list --ready --sort impact
kd task close --id <task> --outcome success --note "testes verdes"
```

A hierarquia é fechada: **`epic ⊃ { issue ⊃ task | task }`** (`epic` é a raiz). Fechar exige
**evidência** (`--outcome`). Detalhes em [Tarefas](06-task.md).

## 7. Retomar contexto entre sessões

```bash
kd rewind --budget 2000          # manifest + tarefas prontas (next:)
kd rewind --files src/gateway.rs
kd rewind --resume <context_id>  # retoma 1:1
```

## 8. Commitar

```bash
kd sync --message "notas: decisão do rate limit"
```

Versiona `notas/` + `eventos/`. O derivado (`.idx/`, `cache/`) fica fora do git (o `kd init`
cuida disso via `.git/info/exclude`).

## O ciclo, de novo

```
kd ask → kd write → kd task → kd sync
```

**Regra de ouro:** busque antes de gravar. O `kd ask "<rascunho>"` evita duplicata e aponta a
nota que talvez você só precise atualizar (`kd write --update <ID> --summary "..."`).

## Troubleshooting rápido

| Sintoma | Causa provável | Ação |
|---|---|---|
| `kd: command not found` | PATH não atualizado | `export PATH="$HOME/.local/bin:$PATH"` |
| `nota ausente`/`schema` | corpus legado ou nota malformada | `kd maintenance doctor --fix` |
| `ask` sem resultados | termo não casa | veja o sentinela `[no_results]`; ajuste a query |
| Embeddings fora do ar | servidor não sobe | `kd maintenance watch-service --status` |
| Conflito de merge | dois devs editaram a mesma nota | `kd maintenance doctor` aponta; resolva/supersede |

A lista completa está em [Troubleshooting](troubleshooting.md).

## Próximo passo

➡️ [Conceitos e arquitetura](01-conceitos.md) · [Comandos](README.md#comandos-um-guia-por-comando)
