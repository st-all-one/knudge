# `kd self` — utilitários do binário

## O que faz

Instalação e utilidades do binário: recipe de integração para clientes de agente, shell
completions, atualização e versão.

| Subcomando | O que dá |
|---|---|
| `setup <CLIENTE>` | Grava a recipe de integração do cliente (`claude`/`cursor`/`codex`/`pi`) |
| `completions <SHELL>` | Gera completions (`bash`/`zsh`/`fish`) |
| `upgrade` | Atualização automática (não disponível; instale pelo canal de origem) |
| `version` | Versão do binário |

## Em 30 segundos

```bash
kd self version
kd self completions bash > ~/.local/share/bash-completion/completions/kd
kd self setup claude
```

## Uso

```
kd self setup <CLIENTE>
kd self completions <SHELL>
kd self upgrade
kd self version
```

## Exemplos

### Nível 1 — versão

```bash
kd self version
```

```
kd 0.3.1
```

Em `--json`: `{name: "knudge", version: "0.3.1"}`.

### Nível 2 — completions

```bash
kd self completions bash   # ou zsh, fish
```

Saída é o script de completion (stdout), pronto para salvar no local certo do seu shell.

### Nível 3 — recipe para um agente

```bash
kd self setup claude
```

Grava `.knudge/setup/claude.json` com a recipe:

```json
{
  "knudge": {
    "version": "0.3.1",
    "client": "claude",
    "binary": "kd",
    "verbs": ["prime", "rewind", "ask", "write", "task", "maintenance"],
    "protocol": "AGENTS.md",
    "mcp": { "command": "knudge-mcp", "transport": "stdio" }
  }
}
```

O `setup` **não** edita a config do cliente: ele deixa a recipe pronta para você apontar. Para o
servidor MCP, veja [MCP](14-mcp.md).

### Nível 4 — atualização

```bash
kd self upgrade
```

Retorna `invalid_input` (exit 2): a atualização automática não está disponível — instale a nova
versão pelo canal de origem (script/`make install`).

## Resultados

- `version` — `kd <versão>`; `{name, version}`.
- `completions` — script no stdout; `{shell, script}`.
- `setup` — `recipe <cliente> gravada em <path>`; `{client, path}`.
- `upgrade` — exit 2.
- Cliente/shell desconhecido → exit 2.

## Quando (não) usar

- **Use** para integrar o `kd` a um agente e instalar completions.
- **Não use** como `--version` do clap: para a versão, prefira `kd self version` (o `-V` existe,
  mas o subcomando é estável para scripts).

## Próximo passo

➡️ [MCP](14-mcp.md) · [Troubleshooting](troubleshooting.md)
