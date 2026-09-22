# knudge

Memória **por projeto**, otimizada para LLM: arquivos Markdown como verdade, um índice derivado
reconstruível e um binário único (`kd`). Sem servidor, sem banco, sem daemon obrigatório.

## Construir e testar

```sh
make check     # fmt --check + clippy -D warnings + test + gate de 300 linhas
make build     # cargo build --workspace
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
kd maintenance  # doctor, compact, eval, index, learn
kd config       # .knudge/config.toml
kd forget       # soft-delete / restore
kd sync         # commit de notas/ + eventos/
kd self         # setup, completions, upgrade, version
```

Contrato congelado: [`plan/implementation/16_cli_surface.md`](plan/implementation/16_cli_surface.md).

## Documentação

- Visão geral: [`plan/00_panorama.md`](plan/00_panorama.md)
- Guia de contribuição (agentes): [`AGENTS.md`](AGENTS.md)
- Arquitetura: [`ARCHITECTURE.md`](ARCHITECTURE.md)
- Contrato de bytes: [`TOON.md`](TOON.md)
- Decisões: [`plan/03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md)
- Plano de implementação: [`plan/implementation/README.md`](plan/implementation/README.md)
