# 01 — Instalação e ambiente

## Instalar o binário

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
curl ... | VERSION=v0.2.3 bash

# Outro diretório de instalação
curl ... | INSTALL_DIR=/usr/local/bin bash

# Compilar do source (precisa de Rust 1.97+)
./install.sh --from-source
make install
```

**Requisitos:** `git` no projeto onde você vai usar. Embeddings (busca semântica) são
**opcionais** — veja o [guia de embeddings](06-embeddings.md).

## PATH e completions

O instalador adiciona `~/.local/bin` ao seu PATH (em `~/.profile`, `~/.bashrc`, `~/.zshrc`) e
instala completions de **bash**, **zsh** e **fish**. Se ele avisar, reinicie o shell ou rode:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Preparar o ambiente de embeddings (opcional)

Se você quer busca semântica, o caminho mais curto é:

```bash
kd maintenance watch-service --install
```

Esse comando baixa o **`llama.cpp`** e o **modelo GGUF** (se faltarem), sobe o servidor
persistente e cria o worker de auto-drain. Ele **pergunta antes** de agir (`--yes` pula;
`--no-deps` não baixa dependências).

O passo a passo manual (com todos os gerenciadores de pacote) está no
[guia de embeddings](06-embeddings.md).

## Verificar a instalação

```bash
kd self version        # versão do binário
kd prime               # o protocolo ("help da IA")
kd --help              # ajuda geral
```

## Desinstalar

```bash
# Move os binários para o lixo recuperável (notas e config preservadas)
kd self --help         # veja as opções
```

O instalador também aceita `--uninstall` (via `install.sh --uninstall`), que invalida os
binários e as completions, **sem tocar** no seu corpus (`.knudge/`) nem na config global.

## Próximo passo

➡️ [Primeiros passos](02-primeiros-passos.md)
