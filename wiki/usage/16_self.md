# 16 · `kd self` — utilitários do binário

## Para que serve

Instalação e utilidades do próprio `kd`: preparar a integração com um cliente de agente, gerar
completions de shell, atualizar o binário e mostrar a versão.

| Subcomando | O que faz |
|---|---|
| `setup <CLIENTE>` | Grava a recipe de integração (`claude`/`cursor`/`codex`/`pi`) |
| `completions <SHELL>` | Gera completions (`bash`/`zsh`/`fish`) |
| `upgrade` | Atualiza o binário pelo instalador oficial (release + checksum) |
| `version` | Mostra a versão |

## Quando usar

- **Use `setup`** ao integrar o knudge a um agente (e `completions` para o seu shell).
- **Use `upgrade`** para atualizar sem reinstalar à mão.
- **Use `version`** em scripts que precisam da versão.

## Sintaxe

```
kd self setup <CLIENTE>
kd self completions <SHELL>
kd self upgrade [--version TAG] [--dry-run] [--script PATH | --url URL --sha256 HEX]
kd self version
```

## Exemplos

### 1. Versão

```bash
kd self version
```

Saída:

```
kd 0.5.0
```

Em `--json`: `{name: "knudge", version: "0.5.0"}`.

### 2. Completions

```bash
# 1. bash
kd self completions bash > ~/.local/share/bash-completion/completions/kd

# 2. zsh
kd self completions zsh > ~/.zfunc/_kd

# 3. fish
kd self completions fish > ~/.config/fish/completions/kd.fish
```

### 3. Integração com um cliente

```bash
# 1. Claude
kd self setup claude

# 2. Cursor
kd self setup cursor

# 3. pi
kd self setup pi
```

Grava `.knudge/setup/<cliente>.json` com a recipe (binário, verbos, transporte MCP). O `setup`
**não** edita a configuração do cliente: ele deixa o arquivo pronto para você apontar. Veja
[MCP](17_mcp.md).

### 4. Atualizar o binário

```bash
# 1. atualizar para a última versão
kd self upgrade

# 2. fixar uma versão
kd self upgrade --version v0.5.0

# 3. só ver o plano
kd self upgrade --dry-run
```

O `upgrade` usa o instalador oficial e verifica o checksum do release antes de instalar. No Windows
nativo, rode via Git Bash/WSL ou baixe o release manualmente.

## Flags

| Comando | Flags |
|---|---|
| `setup` | `<CLIENTE>`: `claude`/`cursor`/`codex`/`pi` |
| `completions` | `<SHELL>`: `bash`/`zsh`/`fish` |
| `upgrade` | `--version <TAG>`, `--dry-run`, `--script <PATH>`, `--url <URL>`, `--sha256 <HEX>` |
| `version` | — |

## Resultado esperado

- **`version`** — `kd <versão>`.
- **`completions`** — o script no stdout.
- **`setup`** — `recipe <cliente> gravada em <path>`.
- **`upgrade`** — confirmação (ou o plano, com `--dry-run`).
- Cliente/shell desconhecido → exit 2.

## Quando não usar

- Para a versão do `clap` (`-V`) prefira `kd self version` (estável para scripts).

## Veja também

➡️ [MCP](17_mcp.md) · [Troubleshooting](19_troubleshooting.md) · [Quickstart](00_quickstart.md)
