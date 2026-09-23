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
