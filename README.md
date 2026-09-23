# knudge

Memória **por projeto**, otimizada para LLM: arquivos Markdown como verdade, um índice derivado
reconstruível e um binário único (`kd`). Sem servidor, sem banco, sem daemon obrigatório.

## Instalação

**A partir do source** (precisa de Rust 1.97+):

```sh
make install          # release build + binários + config global + completions + PATH
make uninstall        # invalida os binários (move para um lixo recuperável)
```

Instala em `~/.local/bin` (`PREFIX`/`BINDIR` mudam o destino) e cria a config global em
`~/.config/local/knudge/config.toml`.

**Pelo script** (release pré-compilado ou source, sem clonar o repositório):

```sh
# release (verifica SHA-256 e instala)
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash

# versão fixa / destino alternativo
curl ... | VERSION=v0.1.0 bash
curl ... | INSTALL_DIR=/usr/local/bin bash

# de dentro do repositório, a partir do source
./install.sh --from-source
./install.sh --uninstall
```

O script instala os dois binários (`kd` e `knudge-mcp`), cria a pasta de config global,
semeia os defaults, instala completions de bash/zsh/fish e ajusta o PATH. Nada é apagado de
forma irreversível: artefatos antigos vão para `${XDG_CACHE_HOME:-~/.cache}/knudge/trash`.

## Preparar o ambiente

| Requisito | Obrigatório? | Nota |
|---|---|---|
| Rust **1.97+** (edição 2024) | só para build do source | MSRV travada; `make install` compila release |
| `git` | sim | o `kd` ancora o `.knudge/` na raiz do repositório |
| servidor de embeddings | **não** | busca semântica é opcional; sem ele o `ask` usa BM25 |

O binário é **único e sem daemon** (`kd`), e o `knudge-mcp` é opcional para agentes via MCP. Nada
de banco, servidor HTTP próprio ou runtime de IA embutido: o embedding (quando usado) roda **fora**
num servidor local OpenAI-compatible.

## Integrar ao projeto

O `kd` ancora a memória na **raiz do git** (um `.knudge/` por projeto):

```sh
cd meu-projeto
kd init                 # funda .knudge/ + bloco no AGENTS.md (protocolo para agentes)
kd write --type fact "O parser de TOON é byte-exato" --tag toon
kd task new "Implementar sync offline-first" --scope epic
kd sync                 # commit de notas/ + eventos/
```

O que **versionar**: `.knudge/notas/` (a verdade, Markdown) e `.knudge/eventos/` (auditoria).
O que é **derivado** e reconstruível (não versionar): `.knudge/.idx/`, `.knudge/cache/` e
`.knudge/contexts/` — o `kd init` cuida do `.gitignore`/`.gitattributes`.

Para agentes via **MCP** (JSON-RPC 2.0 sobre stdio), rode `knudge-mcp`: ele expõe os gatilhos de
memória sobre o mesmo `.knudge/` do projeto (sem servidor de rede).

## Embeddings (busca semântica, opcional)

O `kd ask` funciona **sem embeddings** (BM25 + âncoras + RRF); a busca semântica é um canal
**derivado** que melhora perguntas em linguagem natural. Sem provedor, o `ask` degrada para
lexical e o aviso aparece em `warnings[]`.

**Modelo recomendado:** `ibm-granite/granite-embedding-97m-multilingual-r2` (384 dims,
Apache-2.0, multilíngue com PT explícito), escolhido na bancada PT-BR ([`bench/`](bench/), D123).
Sirva-o com `llama.cpp` usando **`--pooling mean`** — o índice é gravado com esse pooling e outro
pooling degrada o ranking.

```sh
# 1. servidor local OpenAI-compatible (GGUF Q8_0, ~115 MB)
llama serve -m models/granite-97m-r2-Q8_0.gguf --embeddings --pooling mean --port 8084

# 2. aponte o kd para o servidor (config de projeto; --global aplica a todos)
kd config set embeddings.provider http
kd config set embeddings.model ibm-granite/granite-embedding-97m-multilingual-r2
kd config set embeddings.dimensions 384
kd config set embeddings.similarity cosine
kd config set embeddings.endpoint http://127.0.0.1:8084/v1/embeddings

# 3. indexe as notas e busque
kd maintenance index --drain
kd ask "como o servidor não vê o conteúdo das notas"
```

- **Assíncrono e lazy:** o embedding nunca bloqueia `write`/`ask`; notas novas ficam `pending`
  até drenar (`kd maintenance index --drain`). Trocar de modelo invalida o índice e re-embeda tudo.
- **Para desligar:** `kd config set recall.semantic false` (ou `embeddings.enabled false`) — volta
  a BM25 puro.
- A escolha de modelo é configurável; o veredito da bancada e o A/B estão em
  [`plan/04_embeddings.md`](plan/04_embeddings.md).

## Construir e testar

```sh
make check     # fmt --check + clippy -D warnings + test + gate de 300 linhas
make build     # cargo build --workspace
make ci        # check + nextest + deny + audit + machete + typos (E13)
```

## Superfície

```
kd              # equivale a `kd prime`
kd init         # funda .knudge/ no projeto + prompt inicial
kd prime        # protocolo de uso (estático, byte-idêntico)
kd rewind       # estado/handoff ponto-no-tempo
kd ask          # toda pesquisa (recall + get + expand)
kd write        # toda escrita (create + update + arestas)
kd task         # plan / epic / issue / task
kd knowledge    # mapa de conhecimento (clusters)
kd maintenance  # doctor, compact, eval, index, learn
kd config       # .knudge/config.toml
kd forget       # soft-delete / restore
kd sync         # commit de notas/ + eventos/
kd self         # setup, completions, upgrade, version
```

Contrato congelado: [`plan/implementation/16_cli_surface.md`](plan/implementation/16_cli_surface.md).

Além do `kd`, o binário `knudge-mcp` serve o protocolo MCP (JSON-RPC 2.0 sobre stdio) com os
gatilhos de memória — detalhes em [`plan/implementation/18_mcp_transporte.md`](plan/implementation/18_mcp_transporte.md).

## Documentação

- Visão geral: [`plan/00_panorama.md`](plan/00_panorama.md)
- Guia de contribuição (agentes): [`AGENTS.md`](AGENTS.md)
- Arquitetura: [`ARCHITECTURE.md`](ARCHITECTURE.md)
- Contrato de bytes: [`TOON.md`](TOON.md)
- Decisões: [`plan/03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md)
- Plano de implementação: [`plan/implementation/README.md`](plan/implementation/README.md)
- Matriz de aceite por tool: [`plan/implementation/17_matriz_aceitacao.md`](plan/implementation/17_matriz_aceitacao.md)
- Bordas e testes que as travam: [`DIVERGENCES.md`](DIVERGENCES.md)
