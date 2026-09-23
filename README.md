# knudge

**knudge** é uma CLI Rust (`kd`) de **memória por projeto otimizada para LLM**: notas em
**Markdown como verdade**, índice derivado reconstruível e busca híbrida (BM25 + âncoras +
embeddings opcional via RRF). Sem servidor, sem banco, sem daemon — o binário `knudge-mcp` expõe
os gatilhos de memória por MCP (JSON-RPC 2.0 sobre stdio).

O ciclo é **`ask` → `write` → `task` → `sync`**: buscar antes de gravar, registrar conhecimento,
planejar/executar trabalho e versionar.

## Instalação (com embeddings)

**1. Binário** — instala `kd` + `knudge-mcp` em `~/.local/bin`:

```bash
curl --proto '=https' --tlsv1.2 --show-error --fail \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash
```

Requer `git` no projeto. Versão fixa: `... | VERSION=v0.2.3 bash`. Do source: `make install`.

**2. Embeddings (opcional, mas recomendado)** — um comando baixa o `llama.cpp`, o modelo GGUF e
deixa o servidor **persistente** + worker de auto-drain de pé:

```bash
kd maintenance watch-service --install
```

Ele **pergunta antes** de agir (`--no-deps` não baixa dependências; sem `systemd`/`launchd` ele
imprime a linha de cron). O passo a passo manual por SO — **Windows, macOS, Ubuntu, Fedora,
Arch** — está em [`docs/06-embeddings.md`](docs/06-embeddings.md).

## Quickstart

```bash
kd init                                        # funda .knudge/ e o AGENTS.md

kd ask "como o gateway limita requisições" --brief

kd write --type decision "Rate limit é 100 rps por chave" \
  --tag gateway --anchor src/gateway.rs

kd task new "Sync offline-first" --scope epic
kd task new "Resolver conflito de merge" --scope task --parent <epic>
kd task list --ready --sort impact

kd knowledge map --axis container --semantic
kd maintenance watch-service --status
```

**Sempre busque antes de gravar** — o `write` faz dedup (`< 0.75` cria, `0.75–0.92` merge,
`≥ 0.92` rejeita). `kd` sozinho = `kd prime`, o protocolo estático (o "help da IA").

## Comandos principais

| Comando | Para quê |
|---|---|
| **`kd maintenance watch-service`** | Sobe/checa o servidor de embeddings persistente e o worker de auto-drain (`--install`/`--status`/`--uninstall`). |
| **`kd write`** | Grava conhecimento (fato/decisão/erro/risco): `--type`, `--tag`, `--anchor`, `--update`, `--link`, `--outcome`. |
| **`kd task`** | Planeja/executa trabalho (`plan ⊃ epic ⊃ issue ⊃ task`): `new`/`list`/`show`/`close`/`claim`/`graph`. |
| **`kd ask`** | Busca: filtros (`--type/--tag/--status/--anchor`), `--id`, `--around`, `--rank`, `--brief`. |
| **`kd knowledge map`** | Mapa de clusters estrutural e `--semantic`. |
| `kd rewind` | Handoff de contexto com orçamento de tokens. |
| `kd forget` · `kd sync` | Soft-delete e commit de `notas/` + `eventos/`. |
| `kd maintenance doctor --audit` | Saúde, integridade e âncoras quebradas. |

## MCP

`knudge-mcp` serve 4 gatilhos por JSON-RPC sobre stdio: `knudge_pre_write`, `knudge_pre_edit`,
`knudge_session_end` e `knudge_status`. Configure com `kd self setup <claude|cursor|codex|pi>`.
Os hints são **ponteiros** (`id + statement + score`) — o corpo fica no `kd`, nunca no contexto
do modelo.

## Documentação

Guias de uso: [`docs/`](docs/README.md) · Para agentes: [`SKILL.md`](SKILL.md) · Índice para LLM:
[`llms.txt`](llms.txt) · CLI completa: [`16_cli_surface.md`](plan/implementation/16_cli_surface.md) ·
Contrato de bytes: [`TOON.md`](TOON.md) · Arquitetura: [`ARCHITECTURE.md`](ARCHITECTURE.md) ·
Decisões: [`plan/03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md).

## Desenvolvimento

```bash
make check     # fmt --check + clippy -D warnings + test + gate de 300 linhas
make dist      # release otimizado + pacote da plataforma atual em dist/
```

Contribuir: [`AGENTS.md`](AGENTS.md).

## Licença

[MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE) — uso livre.
