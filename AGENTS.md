# AGENTS.md — guia de contribuição do knudge

> Guia para agentes/contribuidores que trabalham **no código deste repositório** (Rust, binário
> `kd`). **Não** é o `AGENTS.md` de projeto-alvo que o `kd init` emitirá (esse é o protocolo de
> escrita de notas do usuário). Documentos irmãos: [`ARCHITECTURE.md`](ARCHITECTURE.md) (camadas),
> [`TOON.md`](TOON.md) (contrato de bytes), [`plan/`](plan/) (decisões `Dxx` e políticas `Rn`).

## 0. Regras de ouro

1. **Idioma:** documentação, comentários, commits e `CHANGELOG` em **português**; identificadores
   de código em **inglês**.
2. **Vermelho = não terminou.** Rode `make check` e deixe verde antes de concluir qualquer tarefa.
3. **Simplicidade e localidade** são metas globais. Recuse infraestrutura que o caso local não
   paga (`tokio full`, servidor, DB, ORM). Ver `14_revisao_tecnica.md` §8.
4. **Domínio puro:** `knudge-core` **nunca** acessa terminal, `argv`, relógio/RNG global ou
   sistema de arquivos direto — só via **portas** (`ports`). Adaptadores ficam na borda.
5. **Sem `unwrap`/`expect`/`panic`** em `src/`; **sem `unsafe`** fora do adaptador de embedding
   (hoje: nenhum). Arquivo de produção **≤ 300 linhas**.
6. **Byte-sensitivity:** ordem, hashes, IDs e serialização são **contrato** — mudanças exigem
   golden/proptest e, se for decisão, um `Dxx` novo.

## 1. Fluxo de trabalho

```sh
make check     # fmt --check + clippy -D warnings + test + gate de 300 linhas  (portão único)
make fmt       # cargo fmt --all
make clippy    # cargo clippy --workspace --all-targets -- -D warnings
make test      # cargo test --workspace
make build     # cargo build --workspace
make install   # release + binários + config global + completions + PATH (source)
make uninstall # invalida os binários (move para o lixo)
make update-version VERSION=v0.2.1  # bump em Cargo.toml/lock/goldens/install.sh/README/CHANGELOG

# alvos extras (E13) — pulam se a ferramenta não estiver instalada
make ci        # check + nextest + deny + audit + machete + typos
make miri      # verificação dinâmica de UB no core puro
make fuzz      # build dos alvos de fuzz (TOON, JSONL)

# alvos pontuais enquanto trabalha
cargo test -p knudge-core toon::tests::round_trip_is_byte_exact
cargo clippy -p knudge-core --all-targets -- -D warnings
```

- Toolchain: Rust **1.97+** (edição 2024). MSRV **obrigatória** = 1.97; não use API mais nova.
- `rustfmt`: largura 100, indent 4, newline Unix (ver `rustfmt.toml`).
- Nunca `cargo fmt` só num arquivo isolado de forma que o `check` global reclame.

## 2. Mapa do repositório

```
crates/knudge-core/   # núcleo puro: modelo, schema, toon, retrieval, lifecycle + portas
crates/knudge-cli/    # binário `kd`: clap, envelope --json, logging, montagem de adaptadores
crates/knudge-mcp/    # servidor MCP: gatilhos + transporte JSON-RPC stdio (binário `knudge-mcp`)
plan/                 # visão (00), decisões (Dxx), implementação por épico (E0x-T0y) e políticas (Rn)
refs/                 # projetos de referência (mulch-rs, seeds-rs, arags) — leitura, não editar
fuzz/                 # alvos de fuzz (TOON, JSONL) — fora do workspace
scripts/              # utilitários: bump-version.sh, knudge-idle.sh, package.sh, check_file_length.sh
.agents/skill/rust/   # skill de Rust (SKILL.md + capítulos)
DIVERGENCES.md        # bordas (Unicode, ordem, lock, atomicidade…) + teste que trava cada uma
SKILL.md              # guia de uso ativo do knudge para agentes (dos projetos que o adotam)
docs/                 # guias de uso (didáticos) por grupo de comandos: ask, write, task, embeddings…
llms.txt              # índice do projeto para modelos de linguagem
```

Cada crate tem um `MODULE.md` com o papel e os módulos. Mantenha-o em sincronia ao criar/remover
módulos. Onde mexer:

| Quero… | Vá para |
|---|---|
| schema/tipos/IDs/hash | `knudge-core/src/schema/` |
| parser/emissor TOON | `knudge-core/src/toon/` |
| notas, eventos, lock, rebuild | `knudge-core/src/store/` |
| JSONL/JSON | `knudge-core/src/jsonl/` |
| config TOML em dois níveis | `knudge-core/src/config/` |
| worktree, exclude, `AGENTS.md`, `sync` | `knudge-core/src/git/` |
| arestas, integridade, ciclos | `knudge-core/src/graph/` |
| BM25, âncoras, filtros, views, RRF | `knudge-core/src/retrieval/` |
| escrita, dedup, update/supersede, forget | `knudge-core/src/write/` |
| rewind, manifest, orçamento, context_id | `knudge-core/src/handoff/` |
| diff, learn, compact | `knudge-core/src/maintenance/` |
| tarefas plan/epic/issue/task | `knudge-core/src/task/` |
| validação, audit, doctor, validators | `knudge-core/src/health/` |
| shelf-life, decay, purga, confiança derivada, clusters | `knudge-core/src/lifecycle/` |
| erros e exit codes | `knudge-core/src/error.rs` |
| tempo determinístico | `knudge-core/src/time.rs` |
| redação de segredos | `knudge-core/src/logging.rs` |
| portas e fakes | `knudge-core/src/ports/` |
| acesso real a SO/rede | `knudge-core/src/adapters/` |
| embeddings (provedor, cache, fila, eval) | `knudge-core/src/embeddings/` |
| superfície de CLI / dispatch | `knudge-cli/src/cli/` + `knudge-cli/src/commands/` + `plan/implementation/16_cli_surface.md` |
| hooks de ciclo de vida | `knudge-core/src/adapters/hook.rs` + `knudge-cli/src/commands/hooks.rs` |
| MCP (gatilhos, JSON-RPC, tools) | `knudge-mcp/src/` (binário `knudge-mcp`) |

## 3. Padrões de desenvolvimento

- **Portas:** dependa de `ports::Clock/Rng/Env/Fs/Git/HookRunner/Logger`. Testes do core usam os
  fakes (`ports::fakes`), nunca o relógio/FS reais.
- **Erros como valores:** funções do core retornam `crate::Result<T>` = `Result<T, Error>`.
- **Tipos proibidos** (`clippy.toml`): `Rc`/`Weak`/`RefCell`/`Cell`, `LinkedList`,
  `HashMap`/`HashSet` (use `BTreeMap`/`IndexMap`; adaptadores abrem exceção com `reason`).
  Determinismo de iteração é requisito.
- **Métodos proibidos:** `transmute`/`forget`/`zeroed`, `ptr::*`, `env::var`/`set_var`,
  `SystemTime::now`/`Instant::now`, `process::exit`. Use portas e propague `Result`.
- **Macros proibidas:** `dbg!`/`todo!`/`unimplemented!`/`unreachable!`/`panic!`.
- **Aritmética:** `overflow-checks` ligado em dev e release; prefira `checked_*`/`saturating_*`.
  Com `-D warnings`, `arithmetic_side_effects` vira erro; se o domínio for limitado, use
  `#[allow(clippy::arithmetic_side_effects, reason = "...")]` no item.
- **Indexação/slicing** (`[]`) é negada: use `.get()`, `.first()`, `first_chunk`, iteradores.
- **Todo `#[allow]` exige `reason`** (`allow_attributes_without_reason = deny`).
- **Caminhos absolutos** de 3+ segmentos (`a::b::c`) são erro: importe com `use`.
- **Dependências:** declare em `[workspace.dependencies]` e consuma com `.workspace = true`;
  justifique no PR/issue (orçamento R43). `default-features = false` quando possível.
- **Limite de arquivo:** ao passar de ~280 linhas, fatie em `mod.rs` + subarquivos por
  responsabilidade (ex.: `toon/{lex,flow,parse,emit}.rs`). O gate mede só `*/src/*`.

## 4. Erros (R30–R35)

- `knudge-core` usa `enum Error` (`thiserror`), `#[non_exhaustive]`, `Send + Sync + 'static`.
  **Nunca** `Box<dyn Error>` na API pública do core.
- `knudge-cli`/`mcp` usam `anyhow` + `.context(...)` nas bordas; converta para `Error`/exit code
  antes de sair.
- Sempre prefira o construtor nomeado: `Error::schema`, `Error::invalid_input`, `Error::conflict`,
  `Error::not_found`, `Error::timeout`, `Error::config`, `Error::internal`; I/O **sempre** com
  `Error::io(path, source)` (contexto de path é obrigatório, R34).
- `ErrorKind::code()` é o **contrato de máquina** (nunca traduzir); `exit_code()` mapeia:
  `not_found=3, invalid_input=2, conflict=4, io=5, timeout=6, config=7, schema=8,
  unsafe_blocked=9, internal=70`; **101 é reservado a panic**.
- `retryable()` = verdadeiro só para `Timeout`.
- Poison de `Mutex`: use `lock_or_recover` (via `PoisonError::into_inner` + `warn`). Nunca
  `.expect("poisoned")`.
- **Degradação graciosa** (R33): canal/recurso opcional que falha retorna resultado parcial +
  `warnings[]`; `strict` (config de projeto) promove warning a erro.

## 5. Logs (R20–R23)

- **stdout = dados** (pipe/`--json`); **stderr = logs**. Nunca `println!`/`eprintln!`
  (`print_stdout`/`print_stderr` negados): use a porta `Logger` / `tracing`.
- Em `--json`, stdout é **só** o envelope; nenhum log pode vazar. Teste:
  `kd … --json 2>/dev/null` deve ser JSON válido.
- **Redação no layer** (`logging::Redactor` + `TracingLogger`): nunca logue corpo de nota,
  segredos, `Authorization`/tokens. Logue ids, contagens e métricas estruturadas.
- EPIPE (pipe fechado) → **exit 0** (D73).

## 6. Testes (D76, E13)

- **Onde:** unidade em `src/<mod>/tests.rs` com `#[cfg(test)] mod tests;`; integração em
  `crates/*/tests/*.rs`. Nome do teste descreve o comportamento, não o método.
- **Sem `unwrap`/`expect`/`panic` em testes** (`allow-*-in-tests = false`). Use:
  - `?` em testes que retornam `crate::Result<()>`;
  - `let Some(x) = ... else { return; }` / `assert!(matches!(...))`;
  - `.ok() == Some(valor)` para comparar `Result` sem `unwrap`;
  - `unwrap_or`/`unwrap_or_default` quando houver valor neutro.
- `let _ = expr;` é proibido (`let_underscore_must_use`): use `let _ignored = expr;` ou nomeie.
- **Determinismo:** teste de core usa fakes (`FixedClock`, `SeqRng`, `MemFs`, `FakeGit`,
  `RecordingLogger`). Nada de tempo/ambiente/FS reais.
- **Proptest** para funções puras (normalize, IDs, TOON, RRF, decay): invariantes e idempotência.
- **Golden** para contrato de bytes (TOON, hash, exit codes, mensagens de erro).
- **Poison** de mutex em teste: `catch_unwind(AssertUnwindSafe(...))` +
  `resume_unwind(Box::new("..."))` — **não** use `panic!`.
- **Integração do binário:** cubra `--help`, `kd == kd prime`, `--json` válido, exit codes e
  **EPIPE → 0**. Use `env!("CARGO_BIN_EXE_kd")`.
- Bug encontrado ⇒ adicione o **teste de regressão** que o pega (ex.: whitespace Unicode no TOON)
  antes de fechar.

## 7. Contrato de bytes (TOON / schema)

- Gramática e regras em [`TOON.md`](TOON.md). Ordem canônica das 28 chaves em
  `schema::CANONICAL_KEYS`; opcionais **omitidos**, nunca `null`.
- `normalize` = NFC + trim + colapso; `body_hash` e `id` derivam dele (D95).
- `id = <prefixo>_<base36(8)>` endereçado por `type + U+001F + normalize(statement)`; o prefixo é
  **histórico** (reclassificar `type` não reescreve o `id`).
- `type` é enum fechado de 11; `scope`/`classification`/`status` também. Chave desconhecida:
  rejeita no write, warning no read; **tipo** desconhecido: rejeita a nota (sem derrubar a leitura).
- Mudou o formato? Atualize `TOON.md`, os goldens e `schema_version`/rebuild (D15).

## 8. O que não fazer

- Não reescrever o brainstorm histórico (`plan/brainstorm/`) — ele é registro.
- Não introduzir aliases/retrocompatibilidade de campos (D14) nem migração de corpus.
- Não adicionar `tokio full`, `reqwest`, servidor HTTP ou DB embutido sem decisão explícita.
- Não “consertar” D92 relaxando lints para passar; conserte o código ou registre `#[allow]` com
  `reason`.
- Não logar corpos/segredos, nem misturar log com stdout.
- Não criar tipos abertos onde o projeto fixou enum fechado.

## 9. Checklist antes de concluir

- [ ] `make check` verde.
- [ ] Testes novos cobrem o comportamento e o bug (se houve) — sem `unwrap`.
- [ ] Se tocou uma borda: linha em [`DIVERGENCES.md`](DIVERGENCES.md) com o teste que a trava.
- [ ] Se adicionou/alterou verbo: linha na
      [`17_matriz_aceitacao.md`](plan/implementation/17_matriz_aceitacao.md) e golden atualizado.
- [ ] `MODULE.md`/docs atualizados se criou/removeu módulo ou mudou contrato.
- [ ] Se mudou decisão/contrato: `Dxx` registrado em `plan/03_decisoes-fechadas.md` e propagado.
- [ ] Se concluiu um épico/tarefa: marque `☑`/`[x]` em `plan/implementation/` e atualize
      `CHANGELOG.md`.
- [ ] Nenhum arquivo de `src/` acima de 300 linhas; nenhum `unwrap/expect/panic/unsafe`.

## 10. Referências

- Decisões fechadas: [`plan/03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md) (`D01–D101`).
- Políticas de engenharia: [`plan/implementation/14_revisao_tecnica.md`](plan/implementation/14_revisao_tecnica.md) (`R01–R44`).
- Lints: [`plan/implementation/15_clippy_config.md`](plan/implementation/15_clippy_config.md) + `clippy.toml`.
- Superfície `kd`: [`plan/implementation/16_cli_surface.md`](plan/implementation/16_cli_surface.md).
- Matriz de aceite: [`plan/implementation/17_matriz_aceitacao.md`](plan/implementation/17_matriz_aceitacao.md).
- Bordas: [`DIVERGENCES.md`](DIVERGENCES.md).
- Rust: [`.agents/skill/rust/SKILL.md`](.agents/skill/rust/SKILL.md).
