<div align="center">

# knudge

**Memória por projeto otimizada para LLM — a nota é a verdade, o índice é derivado.**

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-2b3a42?style=for-the-badge)](./LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.97%2B-DEA584?style=for-the-badge&logo=rust&logoColor=000)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-JSON--RPC%202.0-6E56CF?style=for-the-badge)](./docs/14-mcp.md)
[![Version](https://img.shields.io/badge/version-0.4.0-009739?style=for-the-badge)](./CHANGELOG.md)
[![Made in Brazil](https://img.shields.io/badge/Made_in-Brazil-009739?style=for-the-badge)](https://github.com/topics/brazil)

</div>

O **knudge** é uma CLI Rust (binário **`kd`**) que dá **memória durável a agentes de IA por
projeto**: notas em **Markdown como fonte da verdade**, um **índice derivado reconstruível** e
**busca híbrida** (BM25 + âncoras + embeddings opcional via RRF). Sem servidor, sem banco, sem
daemon. O binário **`knudge-mcp`** expõe os gatilhos de memória por **MCP** (JSON-RPC 2.0 sobre
stdio).

O ciclo é **`ask → write → task → sync`**: buscar antes de gravar, registrar conhecimento,
planejar/executar trabalho e versionar.

---

## 📖 Documentação

| | |
|---|---|
| **[Guias de uso](./docs/README.md)** | um guia por comando + [quickstart](./docs/00-quickstart.md) e [conceitos](./docs/01-conceitos.md) |
| **[Guia para agentes](./SKILL.md)** | o que usar, quando e quando **não** usar |
| **[Índice para LLM](./llms.txt)** | a descrição canônica, em texto |
| **[Superfície da CLI](./plan/implementation/16_cli_surface.md)** | o contrato de cada verbo |
| **[Matriz de aceite](./plan/implementation/17_matriz_aceitacao.md)** | pipe, `--json`, erro e exit por tool |
| **[Contrato de bytes (TOON)](./TOON.md)** | ordem canônica, hashes e ids |
| **[Arquitetura](./ARCHITECTURE.md)** · **[Decisões](./plan/03_decisoes-fechadas.md)** · **[Bordas](./DIVERGENCES.md)** | como e por quê |

## 🚀 Quick-start

### Instalação

```bash
curl --proto '=https' --tlsv1.2 --show-error --fail \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash
```

Instala **`kd`** + **`knudge-mcp`** em `~/.local/bin` — release pré-compilado, **verificado por
SHA-256**, que nunca apaga nada (move o antigo para um lixo recuperável). Versão fixa:
`... | VERSION=v0.4.0 bash`; outro destino: `... | INSTALL_DIR=/usr/local/bin bash`. Do source:
`make install`.

**Embeddings (opcional, recomendado)** — um comando baixa o `llama.cpp`, o modelo GGUF e deixa o
servidor **persistente** + worker de auto-drain de pé:

```bash
kd maintenance watch-service --install
```

O passo a passo manual por SO — **Windows, macOS, Ubuntu, Fedora, Arch** — está em
[`docs/15-embeddings.md`](./docs/15-embeddings.md).

### Primeira sessão

```bash
kd init                                        # funda .knudge/ e escreve o AGENTS.md

kd ask "como o gateway limita requisições" --brief
kd write --summary "Rate limit é 100 rps por chave" --type decision \
  --tag gateway --anchor src/gateway.rs

kd task new --summary "Sync offline-first" --scope epic
kd task list --ready --sort impact
kd sync --message "notas: rate limit"
```

**Sempre busque antes de gravar.** O `write` faz dedup contra o que já existe: score `< 0.75`
cria, `0.75–0.92` faz **merge**, `≥ 0.92` **rejeita**. `kd` sozinho = `kd help`; o protocolo
estático (o "help da IA") é `kd prime`.

## 🧠 Como funciona

Três decisões sustentam tudo:

1. **A nota é a verdade.** Cada nota é um arquivo Markdown com frontmatter **TOON** (ordem
   canônica; opcionais omitidos, nunca `null`) e um `id` **derivado do conteúdo**:
   `<tipo>_<base36(8)>` = `hash(type + U+001F + normalize(statement))`. Reclassificar o tipo
   **não** reescreve o id.
2. **O índice é derivado e descartável.** BM25, âncoras, grafo e embeddings vivem em `.idx/` e
   são reconstruídos a partir de `notas/`. Apagar `.idx/` nunca perde conhecimento.
3. **Busca híbrida e determinística.** Filtros determinísticos → BM25 → âncoras → RRF (e o canal
   vetorial quando há índice), com desempate `(score desc, id asc)`. Sem provedor de embeddings,
   o `ask` degrada para BM25 com `warnings`.

**Âncoras (`--anchor PATH`) ligam a nota ao código** — repetível, aceitam vírgula e glob
(`src/**`); `kd ask --anchor PATH` busca sem query textual e `kd doctor` lista âncoras quebradas.

**Tarefas têm hierarquia fechada:** `epic ⊃ { issue ⊃ task | task }` (o épico é a raiz; a issue é
opcional). Fechar uma tarefa exige **evidência** (`kd task close --outcome success --note "..."`).

## 🌐 Possibilidades de uso

O knudge foi desenhado para **agentes de código** e para **times** que precisam que a decisão de
ontem sobreviva à sessão de hoje — com prova, não com "achismo".

- **🤖 Agentes de IA (MCP)** — Claude, Cursor, Codex e `pi` consultam e gravam memória sem sair do
  fluxo; os hints são **ponteiros** (`id + statement + score`), nunca o corpo da nota.
- **👥 Times com vários devs** — as notas são arquivos endereçados por conteúdo; o git faz o
  merge natural e o índice é reconstruído.
- **🏛️ Projetos de longa duração** — decisões, riscos, erros e perguntas ficam versionados e
  buscáveis, em vez de espalhados por issues e conversas.
- **🗺️ Planejamento e execução** — `kd task` cobre `plan`/`epic`/`issue`/`task`, com `next:`,
  `--sort impact` e grafo do programa.
- **🧭 Onboarding e handoff** — `kd rewind` reconstrói o contexto do projeto dentro de um
  orçamento de tokens.

## 🎯 Qual problema o knudge resolve?

Agentes de IA esquecem. Contexto de projeto vive em conversas efêmeras, issues fechadas e
cabeças de pessoas — e o "o que já decidimos sobre isto?" vira uma busca manual que ninguém faz.
O knudge transforma a memória do projeto em **artefato durável, versionado e determinístico**, que
o próprio agente consulta e atualiza.

O que torna isso possível são três pilares:

### 1. Verdade durável, não um banco opaco

- **Markdown versionado** — a verdade são arquivos em `notas/` (endereçados por tipo), legíveis
  por humanos e versionados pelo git; `eventos/` guarda o log auditável.
- **Contrato de bytes (TOON)** — frontmatter na ordem canônica das chaves, hashes e ids
  determinísticos; nada de `null` acidental nem ordem instável.
- **Reconstruível** — todo derivado (`.idx/`, `cache/`) pode ser apagado e regerado; nada se
  perde, e `kd doctor` detecta divergência.

### 2. Busca híbrida, sem infraestrutura

- **BM25 + âncoras + RRF** — recuperação lexical com IDF por campo, canal de âncoras por
  caminho/glob e fusão RRF determinística; **embeddings são opcionais** (provedor HTTP local
  OpenAI-compatible, default `granite-embedding-97m-multilingual-r2`).
- **Filtros determinísticos** — `--type`, `--status`, `--tag`, `--anchor`, `--scope`; o canal
  vetorial é intersectado com o conjunto permitido.
- **Zero servidor** — um binário e um diretório `.knudge/` na raiz do git.

### 3. Determinismo e contrato de máquina

- **stdout = dados, stderr = logs** — em `--json`, stdout é **só** o envelope
  `{success, command, data?, error?, warnings?}`; nenhum log vaza.
- **Exit codes congelados** — `2` invalid, `3` not_found, `4` conflict, `5` io, `6` timeout,
  `7` config, `8` schema, `70` internal; `101` reservado a panic; EPIPE → `0`.
- **Núcleo puro** — o domínio não conhece terminal, `argv`, relógio ou FS; todo acesso ao mundo
  externo atravessa uma **porta**, e os testes do core usam fakes reproduzíveis byte a byte.

## ⚡ Performance

A versão **0.4.0** reescreveu os caminhos quentes do núcleo — sem mudar **um byte** de saída
(goldens e proptest idênticos). A bancada está em [`bench/`](./bench/RELATORIO.md).

| Operação (N=1167) | v0.3.3 | v0.4.0 | Ganho |
|---|---:|---:|---:|
| `kd prime` / `kd self version` | 40 / 48 ms | **2,3 / 3,9 ms** | **−94 % / −92 %** |
| `kd rewind` | 362 ms | **92 ms** | **−75 %** |
| `kd rewind --files` | 164 ms | **91 ms** | −45 % |
| `kd task graph` | 122 ms | **82 ms** | −33 % |
| `kd ask` (query comum) | 113 ms | **80 ms** | −29 % |
| `normalize` / `body_hash` (micro) | 3,27 / 4,98 µs | **0,19 / 0,43 µs** | −94 % / −91 % |

O maior gargalo (`doctor`/`compact`, O(N²)) foi atacado com uma peneira de postings; o índice
invertido e a leitura paralela do corpus completam o quadro. Detalhes por tarefa em
[`bench/RELATORIO.md`](./bench/RELATORIO.md).

## 🔍 Qualidade

**734 testes** (563 no núcleo, 132 na CLI, 39 no MCP) + `proptest` nas funções puras e **goldens**
para o contrato de bytes.

```bash
make check     # fmt --check + clippy -D warnings + test + gate de 300 linhas
make ci        # check + nextest + deny + audit + machete + typos
make miri      # verificação dinâmica de UB no núcleo puro
make fuzz      # alvos de fuzz (TOON, JSONL)
make bench     # bancada de performance (fora do workspace)
```

O projeto se impõe regras duras: **sem `unwrap`/`expect`/`panic`/`unsafe`**, arquivos de produção
com **≤ 300 linhas**, `BTreeMap`/`IndexMap` (iteração determinística) e aritmética com
`checked_*`/`saturating_*`. Cada borda (Unicode, ordem, lock, atomicidade…) tem um teste que a
trava em [`DIVERGENCES.md`](./DIVERGENCES.md).

## 🏗️ Arquitetura

```
knudge-mcp ──┐
             ├── knudge-core (núcleo puro + portas + adaptadores std)
knudge-cli ──┘
```

| Crate | Papel |
|---|---|
| **`knudge-core`** | Modelo, schema, retrieval, ciclo de vida e **portas** — lógica pura; `adapters` (std) isolado |
| **`knudge-cli`** | Binário **`kd`**: monta adaptadores e escreve a saída |
| **`knudge-mcp`** | Servidor **MCP**: motor de gatilhos + transporte JSON-RPC sobre stdio |

O domínio depende de **portas** (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`), não
de SO. Veja [`ARCHITECTURE.md`](./ARCHITECTURE.md).

## 🔌 MCP

`knudge-mcp` serve 4 gatilhos por JSON-RPC sobre stdio: **`knudge_pre_write`**,
**`knudge_pre_edit`**, **`knudge_session_end`** e **`knudge_status`**. Configure com
`kd self setup <claude|cursor|codex|pi>`.

Os hints são **ponteiros** (`id + statement + score`) — o corpo fica no `kd`, nunca no contexto
do modelo. Detalhes em [`docs/14-mcp.md`](./docs/14-mcp.md).

## 🛠️ Desenvolvimento

```bash
make check     # fmt --check + clippy -D warnings + test + gate de 300 linhas
make build     # cargo build --workspace
make dist      # release otimizado + pacote da plataforma atual em dist/
```

Contribuir: [`AGENTS.md`](./AGENTS.md). Toolchain: Rust **1.97+** (edição 2024).

---

<div align="center">

**O knudge é a memória que o seu agente consulta antes de decidir — e que você consegue auditar depois.**

--- Licenciado sob **MIT OR Apache-2.0** — uso livre ---

</div>
