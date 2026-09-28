# knudge-cli — binário `kd`

Adaptador de linha de comando do knudge. Monta as implementações reais das portas (relógio, FS,
git, embedder, logger), abre uma `Session` e traduz o resultado do núcleo para o **contrato de
saída** (D71). Toda a lógica de domínio vive em `knudge-core`; aqui só há **borda**: parsing
(`clap`), sessão, renderização e o mapa de erro→exit.

**Binário puro:** o pacote expõe só `[[bin]]` (`kd`), **sem `[lib]`**. A única biblioteca é
`knudge-core` (interna, compartilhada com o `knudge-mcp`).

## Contrato de saída (D71/R20–R23)

- **stdout = dados** (pipe/`--json`); **stderr = logs**. Nunca `println!`/`eprintln!` — só
  `output::{emit_stdout, emit_stderr}` (R20).
- Envelope JSON em `--json`: `{success, command, data?, error?, warnings?}` (R31). Nenhum log
  pode vazar no stdout.
- `ErrorKind` → exit code (R35): `2` entrada inválida, `3` não encontrado, `4` conflito, `5` I/O,
  `6` timeout, `7` config, `8` schema, `9` unsafe, `70` interno; `101` reservado a panic.
- **EPIPE** (pipe fechado) → **exit 0** (D73).
- `strict` (config de projeto) promove `warnings[]` a erro (D94).
- `kd` sozinho = `kd help` (exit 0); com `--json` não há envelope sem verbo (D171).

## Dependências

`knudge-core`, `clap`, `serde`, `serde_json`, `tracing`, `tracing-subscriber` e `sha2` (verificação
de supply-chain do `--url` remoto, D184). Nada de `tokio`/`reqwest` (R43).

## Superfície de verbos (v3, D209)

Fonte da verdade: [`plan/implementation/16_cli_surface.md`](../../plan/implementation/16_cli_surface.md).

| Verbo | Papel | Implementação |
|---|---|---|
| `init` | funda `.knudge/` e emite o prompt inicial (D57/D60/D165). | `commands/init.rs` |
| `prime` | protocolo estático, byte-idêntico por versão (D57). | `commands/prime.rs` |
| `rewind` | estado/handoff ponto-no-tempo (D57/D88). | `commands/rewind.rs` |
| `ask` | **toda** pesquisa: recall, `--id`, `--around`, `--rank`, `--tags`, `--suggest`. | `commands/ask/{mod,query,render}.rs` |
| `write` | **toda** escrita: create, `--update`, `--link`, `--batch`. | `commands/write_cmd/`, `write_batch.rs` |
| `task` | hierarquia `epic ⊃ {issue ⊃ task \| task}`. | `commands/task/` |
| `map` | mapa de conhecimento (clusters + comunidades GraphRAG, D193). | `commands/knowledge/map.rs` |
| `maintenance` | `compact`, `learn`, `prune`. | `commands/maintenance/` |
| `doctor` | diagnóstico + `--fix`/`--explain` (D163). | `commands/doctor/` |
| `drain` | fila de embeddings + worker de auto-drain (D170/D186). | `commands/drain/` |
| `config` | get/set/unset/list + `promote` (D61/D157). | `commands/config_cmd.rs`, `knowledge/promote.rs` |
| `forget` | soft/restore/purge (D48/D84). | `commands/forget_sync.rs` |
| `sync` | commit de `notas/`+`eventos/` (D32). | `commands/forget_sync.rs` |
| `self` | version, completions, setup, upgrade (D69/D187). | `commands/self_cmd/` |

**Domínio (8):** `ask`, `write`, `task`, `rewind`, `map`, `doctor`, `drain`, `forget`.
**Fundação/meta (5):** `init`, `prime`, `sync`, `config`, `self`. O verbo `knowledge` não existe
mais (D209): `ask --rank|--tags|--suggest`, `map`, `config promote`.

## Estrutura

### Borda (infra)

| Arquivo | Papel |
|---|---|
| `main.rs` | Ponto de entrada `run()`: parseia, inicializa logging, despacha, emite e drena ocioso. |
| `session.rs` | `Session::open` resolve o projeto, carrega a config efetiva e monta store/eventos/índice/grafo; `corpus()` lê numa passada (E15-T02); `write_context`, `changed_paths`, `sweep_residues`. |
| `envelope.rs` | `Envelope::{success, failure, to_json_line}` — envelope JSON de máquina. |
| `output.rs` | `Output` + `emit_stdout`/`emit_stderr` com tratamento de EPIPE. |
| `logging.rs` | `TracingLogger` — adaptador `tracing` da porta `Logger`, com redação (R21/R22) em stderr. |
| `commands/mod.rs` | `run(cli)` → `commands::run` (dispatch) e `run_session` (verbos com estado). |

### `cli/` — definição dos argumentos (`clap`)

| Arquivo | Papel |
|---|---|
| `mod.rs` | `Cli`, `enum Command` (14 verbos), flags globais (`--json`, `--verbose`, `--root`). |
| `ask.rs` | `AskArgs` — recall/`--id`/`--around` + modos `--rank`/`--tags`/`--suggest`. |
| `write.rs` | `WriteArgs`/`ForgetArgs`/`SyncArgs`. |
| `task.rs` | Subcomandos de `kd task`. |
| `rewind.rs` | `RewindArgs` (modos, escopo, `--files`). |
| `knowledge.rs` | `MapArgs`, `RankArgs`, `TagsArgs`, `SuggestArgs`, `PromoteCommand`. |
| `health.rs` | `DoctorArgs` (D163) e `DrainArgs` (D170/D186). |
| `maintenance.rs` | Subcomandos de `maintenance`, `config` e `self`. |
| `help.rs` | Help embutido (resumo global + por verbo, D167/D171). |

### `commands/` — um módulo por verbo

| Área | Arquivos |
|---|---|
| Núcleo | `prime.rs`, `init.rs`, `rewind.rs` |
| Busca | `ask/{mod,query,render}.rs`, `corpus.rs` (escopo compartilhado D143/D144) |
| Escrita | `write_cmd/{mod,contract}.rs` (data contract soft D191), `write_batch.rs`, `input.rs` (posicional/stdin) |
| Tarefas | `task/{mod,create,mutate,query,show,render,graph,plan,flow,batch}.rs` |
| Conhecimento | `knowledge/{mod,map,clusters,community,hub,rank,tags,suggest,promote}.rs` |
| Manutenção | `maintenance/{mod,extra,proposals}.rs` |
| Saúde | `doctor/{mod,render,explain}.rs`, `validators.rs`, `gate.rs`, `hooks.rs` |
| Fila/embeddings | `drain/{mod,service}.rs`, `embedder.rs`, `idle.rs` |
| Config/self | `config_cmd.rs`, `self_cmd/{mod,upgrade}.rs` |
| Forks | `forget_sync.rs`, `script/mod.rs` (wrapper de scripts acionáveis, D184) |
| Suporte | `parse.rs` (CLI→domínio), `mod.rs` (dispatch) |

### Fluxo de execução

1. `main::run` parseia com `clap`; em erro de parse, mapeia para exit 2 e emite o help.
2. `logging::init` configura o layer (redação, stderr, nível por `--verbose`).
3. `commands::run` despacha; verbos com estado abrem uma `Session` (borda única) e chamam o
   domínio.
4. `emit_success`/`emit_error` escrevem stdout (dados) ou stderr (log/erro).
5. Em `embeddings.mode=lazy`, `commands::idle::maybe_drain` drena **um lote** após a saída
   (best-effort; D131). `KNUDGE_NO_IDLE` desliga.

## Testes

Integração em `crates/knudge-cli/tests/` (`cli.rs`, `golden.rs`, `invariants.rs`, `real_usage.rs`,
`legacy_migration.rs`, `body.rs`, `improvements_032.rs`, `regressions_031.rs`) usando
`env!("CARGO_BIN_EXE_kd")`. Cobrem `--help`, `kd == kd prime`, `--json` válido, exit codes e
**EPIPE → 0**. Unidade em `src/<mod>/tests.rs`. Golden de `prime --long` em
`tests/golden/prime_long.txt`.

## Onde mexer

| Quero… | Vá para |
|---|---|
| argumentos/flags de um verbo | `cli/<verbo>.rs` |
| dispatch e sessão | `commands/mod.rs`, `session.rs` |
| contrato de saída/envelope/exit | `output.rs`, `envelope.rs`, `main.rs` |
| renderização de um verbo | `commands/<verbo>/render.rs` |
| texto do `--help`/`prime` | `cli/help.rs`, `commands/prime.rs` |
| supply-chain de script | `commands/script/`, `commands/self_cmd/upgrade.rs` |
| worker de auto-drain | `commands/drain/service.rs` |

## Referências

- Superfície: [`plan/implementation/16_cli_surface.md`](../../plan/implementation/16_cli_surface.md).
- Matriz de aceite: [`plan/implementation/17_matriz_aceitacao.md`](../../plan/implementation/17_matriz_aceitacao.md).
- Contrato de bytes: [`TOON.md`](../../wiki/specs/TOON.md).
