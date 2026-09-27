# knudge-cli — binário `kd`

**Binário puro:** o pacote expõe só `[[bin]]` (`kd`), **sem `[lib]`**. A única biblioteca é
`knudge-core` (interna, compartilhada com o `knudge-mcp`).

Adaptador de linha de comando. Monta as implementações reais das portas e traduz o resultado
do núcleo para o **contrato de saída** (D71):

- **stdout = dados** (pipe/`--json`); **stderr = logs** (R20).
- Envelope `{success, command, data?, error?, warnings?}` em `--json` (R31).
- Mapa `ErrorKind` → exit code (R35); `EPIPE` → exit 0 (D73).

## Estrutura

| Arquivo | Papel |
|---|---|
| `cli/` | Definição dos argumentos/subcomandos (`clap`) — superfície v3 (D209): `mod.rs` (verbos de topo) + `ask.rs`/`write.rs`/`health.rs`/`knowledge.rs`/`maintenance.rs`/`rewind.rs`/`task.rs`. |
| `session.rs` | Resolve o projeto, carrega a config efetiva, monta store/eventos/índice/grafo e varre resíduos na inicialização (D160); `Session::corpus()` lê notas+índice+grafo numa só passada (E15-T02/O1). |
| `commands/` | Um módulo por verbo (`prime`, `init`, `rewind`, `ask`, `write_cmd` (create/update/link/outcome + data contract soft — D191), `task`, `map` (via `knowledge/map`), `maintenance`, `doctor`, `drain`, `config_cmd`, `forget_sync`, `self_cmd/{upgrade}`) + `hooks`/`validators`/`gate`/`embedder`/`idle`/`input`/`corpus`/`script` (wrapper fino de scripts acionáveis + checksum, `run_body` via stdin para ações read-only, cross-platform — D184/D185/E18); `knowledge/{map,rank,tags,suggest,promote}` implementa `kd map` + os modos `kd ask --rank|--tags|--suggest` + `kd config promote` (mapa + comunidades `GraphRAG` — D193; sugestões semânticas D158 e regras governadas D157 — superfície D209), `maintenance/{extra,proposals}` (saída/portão D156), `drain/{service}` (worker de auto-drain sob `drain`; supply-chain D183, `--uninstall` preserva o GGUF — D186/E17), `doctor/{render,explain}` (texto/JSON + achados esperado×encontrado×ação; D163) e `task/{query,show,render,batch}` separam os subcomandos. |
| `envelope.rs` | Serialização do envelope JSON. |
| `output.rs` | Escrita em stdout/stderr com tratamento de `EPIPE`. |
| `logging.rs` | Adaptador `tracing` do port `Logger` com redação. |

## Fluxo

1. `run()` parseia (`clap`), inicializa o logging e chama `commands::run`.
2. Verbos com estado abrem uma `Session` (borda única) e chamam o domínio.
3. `strict` (config de projeto) promove `warnings[]` a erro (D94).
4. Sucesso → texto/JSON em stdout; erro → `erro: …` em stderr ou envelope JSON.
5. Em `embeddings.mode=lazy`, `commands::idle::maybe_drain` drena **um lote** depois de emitir a
   saída (best-effort; D131).
