# CLI — superfície e contrato de saída

O binário `kd` é a borda: parseia argumentos (`clap`), monta os adaptadores, abre a `Session` e
traduz o resultado do núcleo para o **contrato de saída** (D71).

- Código: `crates/knudge-cli/src/`
- Decisões: D14, D57, D68–D73, D94, D128/D129, D143–D147, D152, D161–D171, D184–D188, D209, D210, D212
- Fonte da verdade: `plan/implementation/16_cli_surface.md`

## Superfície (D209)

**14 verbos** (+ `help`): `init`, `prime`, `rewind`, `ask`, `write`, `task`, `map`, `maintenance`,
`doctor`, `drain`, `config`, `forget`, `sync`, `self`.

- **Domínio (8):** `ask`, `write`, `task`, `rewind`, `map`, `doctor`, `drain`, `forget`.
- **Fundação/meta (5):** `init`, `prime`, `sync`, `config`, `self`.
- O verbo `knowledge` **não existe** (D209): `ask --rank|--tags|--suggest`, `map`,
  `config promote`. Sem retrocompatibilidade (D14) — o caminho antigo exit 2.

## Contrato de saída (D71/R20–R23)

- **stdout = dados** (pipe/`--json`); **stderr = logs**. Nunca `println!`/`eprintln!` (R20).
- Envelope JSON em `--json`: `{success, command, data?, error?, warnings?}` (R31); nenhum log
  vaza no stdout.
- Pipe enxuto: `id|statement|score|why` (D39). Busca vazia → `[no_results]` (D152).
- **EPIPE** (pipe fechado) → **exit 0** (D73).
- `strict` (config de projeto, D94) promove `warnings[]` a erro.
- `kd` sozinho = `kd help` (exit 0); `--json` sem verbo = `invalid_input` (2) (D171).

## Exit codes (R35)

`2` entrada inválida · `3` não encontrado · `4` conflito · `5` I/O · `6` timeout · `7` config ·
`8` schema · `9` unsafe · `70` interno · `101` panic · `0` sucesso. Detalhes em [`erros.md`](erros.md).

## Escopo obrigatório (D143/D144/D130)

Operações que varrem o corpus (`map`, `ask --rank`, `maintenance learn/compact/prune`,
`task list`) exigem escopo (`--tag`/`--anchor`/`--type`/`--class`/`--scope`/`--around`) ou
`--universe`; sem escopo → exit 2.

## Convenções de entrada (D140/D147)

- O **posicional é conteúdo**, nunca metadado: em `write`/`task new` é o corpo; em `ask` é a
  consulta. Ids viram `--id`.
- `--params '{...}'` universal; `--params -` lê de stdin. Conteúdo por stdin/heredoc (posicional
  `-` ou stdin não-TTY). `--params` e `-` são vias exclusivas.

## Listas (D210)

Toda flag de valor múltiplo com semântica de **seleção** (`--id`, `--type`, `--class`, `--tag`,
`--anchor`, `--edge`, `--claim`, `--checks`, `--files`) aceita **repetição** (`--tag a --tag b`)
e **lista com vírgula** (`--tag a,b`), equivalentes (`value_delimiter = ','`). O formato por
**espaço** não existe (`num_args = 1..` removido). **Texto livre** (`[QUERY]`/corpo, `--step`,
`--summary`, `--note`, `--message`) **não** é dividido. Modos sem query (`ask --id`/`--around`)
**conflitam** com a query textual (exit 2). Entrada estruturada/array é via `--params '<json>'`.

## Conjuntos fechados (D212)

Toda flag/campo de valor pré-definido (`--type`/`--class`/`--status`/`--scope`/`--kind`/
`--outcome`, arestas `--link`/`--edge`/`--via`, `--relation`, `--axis`, `--sort`, `--template`,
`self setup`, `self completions`, `--log-level`, `config --key`) **rejeita valor inválido** com a
**lista das possibilidades** e a **sugestão da mais provável** (Levenshtein determinística,
`knudge-core::schema::suggest`). Flag **ausente** não valida (sem erro nem lista). Os valores de
cada conjunto estão em `wiki/usage/02_ciclo.md` §5.8.

## Help e prime (D57/D166/D167/D171)

- `prime` compacto por padrão (`PRIME_COMPACT`); `--long` = protocolo completo + gramática TOON.
- `--help` de cada verbo ensina, exemplifica e explicita o escopo obrigatório (`--universe`).
- `help` embutido (`cli/help.rs`) — resumo global + por verbo.

## Borda

`main::run` → parseia → `logging::init` (redação, stderr, `--verbose`) → `commands::run` →
`emit_success`/`emit_error` → `idle::maybe_drain` (D131). Verbos com estado abrem `Session`
(`session.rs`), que resolve o projeto, carrega a config efetiva e monta store/eventos/índice/
grafo (via `Corpus` — leitura única).

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Entrypoint | `main.rs` |
| Sessão | `session.rs` |
| Envelope | `envelope.rs` |
| Saída/EPIPE | `output.rs` |
| Logging | `logging.rs` |
| Argumentos | `cli/` |
| Dispatch/verbos | `commands/` |

## Testes

`crates/knudge-cli/tests/` (cli, golden, invariants, real_usage, legacy_migration, body,
improvements_032, regressions_031, list_args) — `--help`, `kd == kd prime`, `--json` válido, exit
codes, EPIPE → 0.
