# E01 — Fundação: workspace, núcleo puro e ports

> **Fase 0.** Cria o esqueleto do projeto: um workspace Cargo com núcleo puro (sem terminal,
> `argv`, relógio ou RNG globais) e adaptadores finos, mais o binário `kd`. Tudo o que vem
> depois assume esta separação.
>
> **Decisões:** D65, D66, D67, D68, D92.
> **Políticas:** R01–R05, R11–R12, R14, R16, R20–R23, R30–R35, R40–R43
> (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Ter `cargo build` verde, lints de produção limpos, o binário `kd` respondendo `--help`/`--json`,
e os **ports determinísticos** definidos — a base para testes sem tocar o sistema operacional.

## Pré-requisitos

Nenhum.

## Tarefas

### E01-T01 ☐ Workspace e escopos temáticos
- **Objetivo:** workspace `knudge` com crates `knudge-core`, `knudge-cli`, `knudge-mcp` e
  módulos por escopo temático (`core`, `cli`, `mcp`, `jsonl`, `toon`, `git`, `retrieval`,
  `embeddings`, `lifecycle`).
- **Entregáveis:** `Cargo.toml` do workspace; `src/` de cada crate; `MODULE.md` por escopo.
- **Decisões:** D65, D66.
- **Aceite:** `cargo build`; cada crate compila isolado; nenhum adaptador é dependência do core.

### E01-T02 ☐ Ports determinísticos
- **Objetivo:** traits `Clock`, `Rng`, `Git`, `Fs`, `Env`, `HookRunner`, `Logger` no core;
  implementações reais nos adaptadores; fakes no core para teste.
- **Entregáveis:** traits + impls; fakes (`FixedClock`, `SeqRng`, `MemFs`, …).
- **Decisões:** D65.
- **Aceite:** o core compila sem dependências de SO/terminal; um teste do core usa só fakes e
  é reprodutível byte a byte.

### E01-T03 ☐ Gate de qualidade, perfil e supply chain
- **Objetivo:** travar estilo, disciplina e cadeia de dependências antes de crescer o código.
- **Entregáveis:** `rustfmt.toml`; **`clippy.toml`** (ver [`clippy.toml`](clippy.toml) e
  [`15_clippy_config.md`](15_clippy_config.md)); `edition = "2024"` + **`rust-version = "1.97"`
  (MSRV mínimo obrigatório)** + `resolver = "2"`; `[workspace.lints]` com `unsafe_code=deny` (e
  `#![forbid(unsafe_code)]`
  nas crates puras), `unwrap_used=deny`, `expect_used=deny`, `panic=deny`, `disallowed_types`
  (`Rc`/`RefCell`), `print_stdout`/`print_stderr=deny`; `[profile.release]` (`lto="fat"`,
  `codegen-units=1`, `strip="symbols"`, `overflow-checks=true`, `panic="abort"`); `Cargo.lock`
  commitado; `publish=false`; `scripts/check_file_length.sh` (≤300 linhas de produção); alvo
  `make check`; política de dependências (features mínimas, `default-features=false`).
- **Decisões:** D92. **Políticas:** R02, R40, R41, R43, R44.
- **Aceite:** `make check` roda `fmt --check` + `clippy --all-targets -D warnings` + `test` +
  gate de linhas; `clippy.toml` aplicado; `cargo tree` sem runtime pesado (R16/R43).

### E01-T04 ☐ Esqueleto do binário `kd`
- **Objetivo:** `main.rs` mínimo com parsing de subcomandos (stub), envelope `--json`
  (`{success, command, error}`), exit codes e EPIPE → exit 0.
- **Entregáveis:** `knudge-cli` com `clap`; tratamento de pipe fechado.
- **Decisões:** D67, D68, D71, D73.
- **Aceite:** `kd --help`; `kd --json` devolve envelope; `kd ... | head -1` sai com 0.

### E01-T05 ☐ Documento de arquitetura
- **Objetivo:** registrar a separação core/adapter e os escopos temáticos.
- **Entregáveis:** `ARCHITECTURE.md` (core puro + ports + adaptadores + mapa de módulos).
- **Decisões:** D65, D68.
- **Aceite:** documento referencia D65 e o grafo de dependências do README.

### E01-T06 ☐ Modelo de erro e envelope de máquina
- **Objetivo:** definir a taxonomia de erro antes de espalhá-la pelo código.
- **Entregáveis:** `enum Error` no core com `thiserror` (`#[from]`, `#[source]`),
  `#[non_exhaustive]` e `Send + Sync + 'static`; `ErrorKind` estável (`not_found`,
  `invalid_input`, `conflict`, `io`, `timeout`, `config`, `schema`, `unsafe_blocked`, `internal`);
  helper de poison (`PoisonError::into_inner` + `warn`); `anyhow` + `with_context` na CLI/MCP;
  mapa **código → exit code**.
- **Decisões:** D71. **Políticas:** R30, R31, R32, R34, R35.
- **Aceite:** `source()` encadeia; nada de `Box<dyn Error>` na API do core; envenenamento não
  derruba o processo; todo erro de I/O carrega `path`/`id`.

### E01-T07 ☐ Logging, observabilidade e redação
- **Objetivo:** logs úteis que **nunca** quebram o pipe nem vazam segredo.
- **Entregáveis:** impl do port `Logger` com `tracing` + `tracing-subscriber` (`EnvFilter`);
  regra **stdout=dados / stderr=logs**; níveis documentados; `#[instrument]` nas operações;
  campos estruturados (candidatos, tempo, orçamento, `pending`, `rrf_k`, cache hit/miss);
  redação por allowlist (corpos, `[secrets]`, `Authorization`, tokens); `context_id`/`session_id`
  como span raiz; log em arquivo opcional com rotação e teto.
- **Decisões:** D91. **Políticas:** R20, R21, R22, R23.
- **Aceite:** `kd … --json 2>/dev/null` é JSON válido; segredo plantado nunca aparece no log.

### E01-T08 ☐ Política de memória e `unsafe`
- **Objetivo:** manter a segurança de memória por construção.
- **Entregáveis:** `#![forbid(unsafe_code)]` em core/cli/mcp; `unsafe` só no adaptador de
  embedding com `#[allow(unsafe_code)]` + comentário `// SAFETY:`; proibir `Rc`/`RefCell` no
  core; `try_reserve`/`Cow<'_, str>` onde couber; `O_NOFOLLOW`/canonicalização ao abrir arquivos
  de `.knudge/`; `Drop` determinístico (sem `mem::forget`).
- **Decisões:** D92. **Políticas:** R01, R03, R04, R05.
- **Aceite:** compila com `forbid(unsafe_code)`; Miri verde (E13-T08); zero `unsafe` fora de
  `embeddings`; symlink rejeitado.

### E01-T09 ☐ Política de recursos e runtime mínimo
- **Objetivo:** teto de memória/disco/tempo, sem runtime pesado.
- **Entregáveis:** canal bounded + `max_pending` como backpressure; pool limitado a
  `available_parallelism()`; timeouts tipados e retry/backoff só em operação idempotente; cap de
  corpo e de cache; decisão de runtime (**worker bloqueante por padrão**; `tokio` mínimo só se
  necessário); HTTP/ORT isolados no adaptador.
- **Decisões:** D79, D80 (contexto). **Políticas:** R11, R12, R14, R16, R43.
- **Aceite:** `cargo tree` sem `tokio full`; rajada acima de `max_pending` não estoura memória;
  I/O lento não trava o comando.

## Definition of Done

- [ ] E01-T01…T09 concluídas e seus aceites verdes.
- [ ] `make check` verde do zero.
- [ ] `ARCHITECTURE.md` e `MODULE.md` presentes.
- [ ] Modelo de erro, política de log e de memória publicados (E01-T06/T07/T08).

## Não-objetivos

- Nada de I/O real de notas, config ou retrieval (vêm em E02–E06).
- Nada de FFI/WASM (D68).
