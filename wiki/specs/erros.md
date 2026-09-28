# Erros e exit codes

O núcleo usa **erros como valores**: `enum Error` fechado (`Send + Sync + 'static`), sem
`Box<dyn Error>` na API pública (R30). `ErrorKind` é o **contrato de máquina** (nunca traduzido).

- Código: `crates/knudge-core/src/error.rs`
- Decisões: R30–R35, D71, D73

## Taxonomia (`ErrorKind`)

`NotFound`, `InvalidInput`, `Conflict`, `Io`, `Timeout`, `Config`, `Schema`, `UnsafeBlocked`,
`Internal`.

| `ErrorKind` | `code()` (máquina) | `exit_code()` |
|---|---|---|
| `NotFound` | `not_found` | **3** |
| `InvalidInput` | `invalid_input` | **2** |
| `Conflict` | `conflict` | **4** |
| `Io` | `io` | **5** |
| `Timeout` | `timeout` | **6** |
| `Config` | `config` | **7** |
| `Schema` | `schema` | **8** |
| `UnsafeBlocked` | `unsafe_blocked` | **9** |
| `Internal` | `internal` | **70** |

`0` = sucesso; `101` é reservado a **panic** (não é um `ErrorKind`). O `code()` é o contrato de
máquina (usado no envelope `--json`); `exit_code()` é a tradução para o processo.

## Regras (R30–R35)

- `enum Error` `#[non_exhaustive]`; **nunca** `Box<dyn Error>` na API do core.
- **Todo erro de I/O carrega o `path`** (`Error::io(path, source)` — R34).
- Construtores nomeados: `Error::schema`, `Error::invalid_input`, `Error::conflict`,
  `Error::not_found`, `Error::timeout`, `Error::config`, `Error::internal`.
- `retryable()` = verdadeiro **só** para `Timeout`.
- `lock_or_recover` resolve **poison** de mutex via `PoisonError::into_inner` + `warn`; nunca
  `.expect("poisoned")`.
- `knudge-cli`/`mcp` usam `anyhow` + `.context(...)` nas bordas e convertem para `Error`/exit
  code antes de sair.

## Degradação graciosa (R33)

Canal/recurso opcional que falha retorna **resultado parcial + `warnings[]`**; `strict` (config de
projeto) promove warning a erro (D94). Exemplos: canal de busca ausente, gate inválido, drift
ausente (`drift = 0`), embeddings fora.

## EPIPE (D73)

Pipe fechado → **exit 0**. Implementado em `output.rs`: a escrita detecta `BrokenPipe` e encerra
com sucesso, sem panic.

## Onde vive

| Aspecto | Arquivo |
|---|---|
| `Error`/`ErrorKind` | `crates/knudge-core/src/error.rs` |
| Envelope/exit na borda | `crates/knudge-cli/src/{envelope,output,main}.rs` |

## Testes

Golden de mensagens (D72), exit codes por verbo (`crates/knudge-cli/tests/cli.rs`) e EPIPE → 0.
