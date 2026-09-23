# knudge-cli — binário `kd`

Adaptador de linha de comando. Monta as implementações reais das portas e traduz o resultado
do núcleo para o **contrato de saída** (D71):

- **stdout = dados** (pipe/`--json`); **stderr = logs** (R20).
- Envelope `{success, command, data?, error?, warnings?}` em `--json` (R31).
- Mapa `ErrorKind` → exit code (R35); `EPIPE` → exit 0 (D73).

## Estrutura

| Arquivo | Papel |
|---|---|
| `cli/` | Definição dos argumentos/subcomandos (`clap`) — superfície v2. |
| `session.rs` | Resolve o projeto, carrega a config efetiva e monta store/eventos/índice/grafo. |
| `commands/` | Um módulo por verbo (`prime`, `init`, `rewind`, `ask`, `write_cmd`, `task`, `knowledge`, `maintenance`, `config_cmd`, `forget_sync`, `self_cmd`) + `hooks`/`validators`/`embedder`; `knowledge/map` e `maintenance/extra` separam os subcomandos. |
| `envelope.rs` | Serialização do envelope JSON. |
| `output.rs` | Escrita em stdout/stderr com tratamento de `EPIPE`. |
| `logging.rs` | Adaptador `tracing` do port `Logger` com redação. |

## Fluxo

1. `run()` parseia (`clap`), inicializa o logging e chama `commands::run`.
2. Verbos com estado abrem uma `Session` (borda única) e chamam o domínio.
3. `strict` (config de projeto) promove `warnings[]` a erro (D94).
4. Sucesso → texto/JSON em stdout; erro → `erro: …` em stderr ou envelope JSON.
