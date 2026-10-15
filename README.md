<div align="center">

# knudge

Memória durável por projeto para agentes de IA — a nota é a verdade, o índice é derivado.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-2b3a42?style=for-the-badge)](./LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.99%2B-DEA584?style=for-the-badge&logo=rust&logoColor=000)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-JSON--RPC%202.0-6E56CF?style=for-the-badge)](wiki/usage/17_mcp.md)
[![Version](https://img.shields.io/badge/version-0.6.1-009739?style=for-the-badge)](wiki/CHANGELOG.md)

</div>

O knudge é uma CLI em Rust (**`kd`**) que dá memória persistente a agentes de IA por projeto:
notas em Markdown como fonte da verdade, índice derivado e reconstruível e busca híbrida de BM25,
âncoras e embeddings opcionais fundida por RRF. Não requer servidor, banco nem daemon, e o binário
**`knudge-mcp`** expõe os gatilhos de memória via MCP (JSON-RPC 2.0 sobre `stdio`).

## Instalação

Release pré-compilado, verificado por SHA-256, que instala `kd` e `knudge-mcp` em `~/.local/bin`:

```bash
curl --proto '=https' \
  --tlsv1.2 \
  --show-error \
  --fail \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh \
  | sh
```

Versão fixa: `... | VERSION=v0.6.1 bash`. Outro destino: `... | INSTALL_DIR=/usr/local/bin bash`.
A partir do código-fonte: `make install`.

Embeddings são opcionais e recomendados; um comando baixa o `llama.cpp` e o modelo GGUF e deixa o
servidor persistente e o worker de auto-drain em execução:

```bash
kd drain service --install
```

Passo a passo por sistema operacional em [`wiki/usage/18_embeddings.md`](wiki/usage/18_embeddings.md).

## Uso

```bash
# init — funda .knudge/, o bloco do AGENTS.md e a skill .agents/skills/kd/SKILL.md
kd init
kd init --force                        # recopia a config global
kd init --git-excluded                 # memória local-only (fora do Git)

# prime — protocolo de uso estático e byte-idêntico por versão (o "help para a IA")
kd prime                               # compacto; --long inclui a gramática TOON + schema
kd prime --long

# ask — busca híbrida; consulte antes de gravar
kd ask "como o gateway limita requisições" --brief          # pipe enxuto: id|statement
kd ask --anchor src/gateway.rs         # por arquivo, sem query textual
kd ask --id <ID>                       # corpos por id (já completos)
kd ask "gateway" --full-content        # corpo completo de todos os hits
# filtros: --type/--class/--tag/--status/--scope/--anchor/--since/--until/--limit

# task — hierarquia epic ⊃ { issue ⊃ task | task }; feche sempre com evidência
kd task new --summary "Sync offline-first" --scope epic
kd task list --ready --sort impact
kd task close --id <ID> --outcome success --note "validado em produção"

# write — registra conhecimento; dedup: score <0.75 cria, 0.75–0.92 faz merge, >=0.92 rejeita
kd write --summary "Rate limit é 100 rps por chave" --type decision \
  --tag gateway --anchor src/gateway.rs
kd write --update <ID> --summary "..." # versiona; `--type task` é rejeitado (use kd task)

# doctor — saúde do corpus (13 verificações); --fix corrige, --explain detalha
kd doctor
kd doctor --fix --explain

# rewind — reconstrói o contexto/handoff no início da sessão (orçamento de tokens)
kd rewind
kd rewind --scope <ID> --budget 4000
```

## Documentação

| Tema | Referência |
|---|---|
| Guias de uso (um por comando) | [`wiki/usage/README.md`](wiki/usage/README.md) |
| Quickstart e filosofia | [`wiki/usage/00_quickstart.md`](wiki/usage/00_quickstart.md) · [`wiki/usage/01_filosofia.md`](wiki/usage/01_filosofia.md) |
| Guia para agentes | [`SKILL.md`](./SKILL.md) |
| Índice para LLM | [`llms.txt`](./llms.txt) |
| Superfície da CLI (contrato por verbo) | [`plan/implementation/16_cli_surface.md`](plan/implementation/16_cli_surface.md) |
| Matriz de aceitação (pipe, `--json`, exit) | [`plan/implementation/17_matriz_aceitacao.md`](plan/implementation/17_matriz_aceitacao.md) |
| Contrato de bytes (TOON) | [`wiki/specs/TOON.md`](wiki/specs/TOON.md) |
| Arquitetura | [`wiki/specs/ARCHITECTURE.md`](wiki/specs/ARCHITECTURE.md) |
| Decisões de projeto | [`plan/03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md) |
| Bordas e divergências | [`wiki/specs/DIVERGENCES.md`](wiki/specs/DIVERGENCES.md) |
| Servidor MCP | [`wiki/usage/17_mcp.md`](wiki/usage/17_mcp.md) |
| Histórico de mudanças | [`wiki/CHANGELOG.md`](wiki/CHANGELOG.md) |
| Contribuição | [`AGENTS.md`](./AGENTS.md) |

## Licença

Licenciado sob **MIT OR Apache-2.0** — uso livre, inclusive comercial, com atribuição.
