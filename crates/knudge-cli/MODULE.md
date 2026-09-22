# knudge-cli — binário `kd`

Adaptador de linha de comando. Monta as implementações reais das portas e traduz o resultado
do núcleo para o **contrato de saída** (D71):

- **stdout = dados** (pipe/`--json`); **stderr = logs** (R20).
- Envelope `{success, command, data?, error?, warnings?}` em `--json` (R31).
- Mapa `ErrorKind` → exit code (R35); `EPIPE` → exit 0 (D73).

## Estrutura

| Arquivo | Papel |
|---|---|
| `cli.rs` | Definição dos argumentos/subcomandos (`clap`). |
| `envelope.rs` | Serialização do envelope JSON. |
| `output.rs` | Escrita em stdout/stderr com tratamento de `EPIPE`. |
| `logging.rs` | Adaptador `tracing` do port `Logger` com redação. |
