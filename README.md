<div align="center">

# knudge

**Memória por projeto otimizada para LLM — a nota é a verdade, o índice é derivado.**

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-2b3a42?style=for-the-badge)](./LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.97%2B-DEA584?style=for-the-badge&logo=rust&logoColor=000)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-JSON--RPC%202.0-6E56CF?style=for-the-badge)](wiki/usage/17_mcp.md)
[![Version](https://img.shields.io/badge/version-0.5.0-009739?style=for-the-badge)](wiki/CHANGELOG.md)

</div>

O **knudge** é uma CLI Rust (binário **`kd`**) que dá **memória durável a agentes de IA por
projeto**: notas em **Markdown como fonte da verdade**, um **índice derivado reconstruível** e
**busca híbrida** (BM25 + âncoras + embeddings opcional via RRF). Sem servidor, sem banco, sem
daemon. O binário **`knudge-mcp`** expõe os gatilhos de memória por **MCP** (JSON-RPC 2.0 sobre
stdio).

O ciclo é **`ask → write → task → sync`**: buscar antes de gravar, registrar conhecimento,
planejar/executar trabalho e versionar.

## 🚀 Quick-start

### 1. Instalação

```bash
curl --proto '=https' --tlsv1.2 --show-error --fail \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash
```

Instala **`kd`** + **`knudge-mcp`** em `~/.local/bin` — release pré-compilado, **verificado por
SHA-256**, que nunca apaga nada (move o antigo para um lixo recuperável). Versão fixa:
`... | VERSION=v0.5.0 bash`; outro destino: `... | INSTALL_DIR=/usr/local/bin bash`. Do source:
`make install`.

**Embeddings (opcional, recomendado)** — um comando baixa o `llama.cpp` e o modelo GGUF e deixa o
servidor persistente + worker de auto-drain de pé:

```bash
kd drain service --install
```

Passo a passo por SO (Windows, macOS, Ubuntu, Fedora, Arch) em
[`wiki/usage/18_embeddings.md`](wiki/usage/18_embeddings.md).

### 2. Primeira sessão

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

## 📖 Documentação

| | |
|---|---|
| **[Guias de uso](wiki/usage/README.md)** | um guia por comando + [quickstart](wiki/usage/00_quickstart.md) e [filosofia](wiki/usage/01_filosofia.md) |
| **[Guia para agentes](./SKILL.md)** | o que usar, quando e quando **não** usar |
| **[Índice para LLM](./llms.txt)** | a descrição canônica, em texto |
| **[Superfície da CLI](./plan/implementation/16_cli_surface.md)** | o contrato de cada verbo |
| **[Matriz de aceite](./plan/implementation/17_matriz_aceitacao.md)** | pipe, `--json`, erro e exit por tool |
| **[Contrato de bytes (TOON)](wiki/specs/TOON.md)** | ordem canônica, hashes e ids |
| **[Arquitetura](wiki/specs/ARCHITECTURE.md)** · **[Decisões](./plan/03_decisoes-fechadas.md)** · **[Bordas](wiki/specs/DIVERGENCES.md)** | como e por quê |

## 🧠 Como funciona

1. **A nota é a verdade.** Cada nota é um arquivo Markdown com frontmatter **TOON** (ordem
   canônica; opcionais omitidos, nunca `null`) e um `id` **derivado do conteúdo**
   (`<tipo>_<base36(8)>` = `hash(type + U+001F + normalize(statement))`). Reclassificar o tipo
   **não** reescreve o id. Aceita **claims SPO**, **proveniência** e arestas de **ontologia leve**
   (`same_as`/`broader`/`narrower`/`related`).
2. **O índice é derivado e descartável.** BM25, âncoras, grafo e embeddings vivem em `.idx/` e são
   reconstruídos a partir de `notas/`; apagar `.idx/` nunca perde conhecimento.
3. **Busca híbrida e determinística.** Filtros → BM25 → âncoras → RRF (mais os canais vetorial e
   de autoridade quando há índice), com desempate `(score desc, id asc)`. Sem provedor de
   embeddings, o `ask` degrada para BM25 com `warnings`.

**Âncoras (`--anchor PATH`) ligam a nota ao código** — repetível, aceitam vírgula e glob
(`src/**`); `kd ask --anchor PATH` busca sem query textual e `kd doctor` lista âncoras quebradas.

**Tarefas têm hierarquia fechada:** `epic ⊃ { issue ⊃ task | task }` (o épico é a raiz; a issue é
opcional). Fechar uma tarefa exige **evidência** (`kd task close --outcome success --note "..."`).

## 🔌 MCP

`knudge-mcp` serve 4 gatilhos por JSON-RPC sobre stdio: **`knudge_pre_write`**,
**`knudge_pre_edit`**, **`knudge_session_end`** e **`knudge_status`**. Configure com
`kd self setup <claude|cursor|codex|pi>`. Os hints são **ponteiros** (`id + statement + score`) —
o corpo fica no `kd`, nunca no contexto do modelo. Detalhes em
[`wiki/usage/17_mcp.md`](wiki/usage/17_mcp.md).

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
de SO. Veja [`ARCHITECTURE.md`](wiki/specs/ARCHITECTURE.md).

## 🛠️ Desenvolvimento

```bash
make check     # fmt --check + clippy -D warnings + test + gate de 300 linhas
make build     # cargo build --workspace
make dist      # release otimizado + pacote da plataforma atual em dist/
```

**883 testes** + `proptest` nas funções puras e **goldens** para o contrato de bytes. Regras
duras: **sem `unwrap`/`expect`/`panic`/`unsafe`**, arquivos de produção com **≤ 300 linhas** e
iteração determinística (`BTreeMap`/`IndexMap`). Contribuir: [`AGENTS.md`](./AGENTS.md).
Toolchain: Rust **1.97+** (edição 2024).

---

<div align="center">

**A memória que o seu agente consulta antes de decidir — e que você consegue auditar depois.**

--- Licenciado sob **MIT OR Apache-2.0** — uso livre ---

</div>
