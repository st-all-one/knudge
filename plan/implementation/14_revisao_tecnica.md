# Revisão técnica — memória, recursos, logs e erros

> Revisão do plano de `plan/implementation/` à luz de `rust_skill/` (Rust **1.97.0**,
> **Edition 2024**) e da meta de **simplicidade/localidade** do knudge.
>
> Cada achado recebe um ID **`Rn`** (não é uma decisão de produto `Dxx`; é uma **política de
> engenharia**). As tarefas novas/alteradas dos épicos passam a citar `Rn`. Prioridade:
> 🔴 bloqueia o MVP · 🟠 estruturante · 🟡 incremental.
>
> **Princípio de corte:** só entra o que paga o custo no domínio do knudge (CLI local, arquivos,
> sem servidor). O que o `rust_skill` mostra como “receita” (ex.: `tokio` full, ORM, servidor
> HTTP) é **recusado** quando não serve ao caso — ver §8.

---

## Sumário executivo — os 10 achados de maior valor

1. **`unsafe` sem política** (R01): `forbid(unsafe_code)` nos crates puros, `unsafe` confinado
   ao adaptador de embedding, `// SAFETY`, Miri/geiger no CI.
2. **Runtime async grande demais** (R16): a digestão é local; usar **worker bloqueante + canal
   bounded** em vez de `tokio full` (reduz dependências, binário e superfície).
3. **stdout/stderr não separados** (R20): contrato `--json`/pipe quebra se log vazar em stdout.
4. **Redação só nos hooks** (R22): mover para o **layer de log** (segredos + corpos nunca).
5. **Sem taxonomia de erro** (R30/R31): `thiserror` no core, `anyhow` na CLI, **código estável**
   + `retryable` + `warnings[]` no envelope.
6. **Sem limpeza RAII / varredura de resíduos** (R05/R10): `*.tmp`/`*.lock` órfãos pós-crash.
7. **Log de eventos cresce para sempre** (R13): segmentação por tamanho/data + checkpoint.
8. **Cache/índice sem teto** (R14): eviction no `.idx/emb_cache`, limiar de memória do índice.
9. **`eventos`/índice carregados inteiros e strings montadas** (R15): streaming e `BufWriter`.
10. **Build/supply chain incompleto** (R40–R43): edition/MSRV/lints de workspace, perfil de
    release, `cargo deny/audit/machete`, `nextest`, `criterion`, features mínimas.

---

## 1. Segurança de memória

### R01 🔴 `unsafe` confinado e verificável
- **Estado no plano:** E01-T03 nega `unwrap/expect/panic`, mas nada sobre `unsafe`; embeddings
  locais (ONNX/Candle) podem arrastar FFI.
- **Risco:** `unsafe` disperso é onde a memória deixa de ser “segura por construção”.
- **Recomendação:** `#![forbid(unsafe_code)]` em `knudge-core`, `knudge-cli`, `knudge-mcp`;
  `unsafe` permitido **só** no adaptador de embedding com `#[allow(unsafe_code)]` local e
  comentário `// SAFETY:` obrigatório; `cargo miri test` nos crates puros e `cargo geiger`
  para auditar o grafo.
- **Onde:** E01-T08; E13-T08.
- **Aceite:** compila com `forbid`; Miri verde; zero `unsafe` fora de `embeddings`.

### R02 🟠 Overflow e política de abort
- **Estado:** perfil release padrão desliga `overflow-checks`; IDs base36(8), `rank`, orçamento.
- **Recomendação:** `overflow-checks = true` no release (custo desprezível aqui) e aritmética
  exposta com `checked_*`/`saturating_*`; `panic = "abort"` no release (coerente com D92),
  `unwind` em dev/test.
- **Onde:** E01-T03, E01-T08; E02-T06.
- **Aceite:** teste nos limites de ID/orçamento; perfil documentado.

### R03 🟠 Proibir `Rc`/`RefCell` no core
- **Estado:** ports + worker de embeddings; risco de interior mutability ad hoc.
- **Recomendação:** `Rc`/`RefCell` **proibidos** (não são `Send`); estado compartilhado só via
  `Arc` + `Mutex`/`RwLock` ou message passing; `OnceLock` para constantes; `Cell` só local e `Copy`.
- **Onde:** E01-T02, E01-T08; E11-T03.
- **Aceite:** `clippy::disallowed_types` no workspace; core sem `Rc`/`RefCell`.

### R04 🟡 Alocações limitadas
- **Estado:** parser TOON, índice e corpos sem teto explícito.
- **Recomendação:** `try_reserve`/`try_reserve_exact` antes de buffers grandes; **cap de corpo**
  (ex.: 1 MiB, config) com erro tipado; `Box<[T]>`/`shrink_to_fit` para imutáveis; `Cow<'_, str>`
  no parser para evitar clones.
- **Onde:** E02-T02; E06-T07; E08-T09.
- **Aceite:** corpo acima do cap → erro (sem abort); falha de `try_reserve` vira `ErrorKind`.

### R05 🟠 Limpeza por RAII e ordem de drop
- **Estado:** locks/tmp descritos sem guard explícito.
- **Recomendação:** `TempFile`/`LockGuard` com `Drop` que remove/libera; `fsync` **antes** do
  drop; ordem documentada (nota antes do evento; evento antes do lock); sem `mem::forget`;
  abrir arquivos com `O_NOFOLLOW`/canonicalizar para evitar symlink em `.knudge/`.
- **Onde:** E03-T08.
- **Aceite:** erro/crash não deixa `*.tmp`/`*.lock`; symlink rejeitado.

---

## 2. Recursos

### R10 🟠 Varredura de resíduos na inicialização
- **Estado:** lock tem stale 30 s; `*.tmp`/`*.stale` órfãos são varridos ao abrir a sessão
  (`Session::sweep_residues`, D160) — **não** varre `*.lock`/`.locks/` (reclaim atômico no
  `lock.rs`/`doctor --fix`).
- **Recomendação:** no start, varrer `notas/`/`.idx/`/`cache/`/`eventos/` por `*.tmp`/`*.stale`
  mais velhos que `stale_ms` (30 s) e removê-los com `warn`; **não** varrer `*.lock`/`.locks/` —
  o reclaim de lock é atômico e fica no `lock.rs`/`doctor --fix` (D160).
- **Onde:** E03-T08; E09-T04.
- **Aceite:** órfão antigo removido; fresco preservado; lock vivo preservado.

### R11 🟠 Backpressure e limites de concorrência
- **Estado:** `max_pending` citado, mas sem canal/limites explícitos.
- **Recomendação:** canal **bounded** para o worker; `max_pending` como backpressure (adiar, não
  crescer sem limite); pool limitado a `available_parallelism()`; nenhum `thread::spawn` por nota.
- **Onde:** E11-T10; E01-T09.
- **Aceite:** rajada acima de `max_pending` não estoura memória.

### R12 🟠 Timeouts, retry e backoff
- **Estado:** timeout em hook/HTTP curto citado; sem política geral.
- **Recomendação:** todo I/O externo com **timeout tipado**; retry **só** idempotente (HTTP, lock)
  com backoff exponencial + jitter e teto; timeout → `ErrorKind::Timeout`; execução de hooks/processos
  **sem shell**, com `OsString`, env em allowlist e `cwd` explícito.
- **Onde:** E01-T09; E11-T10; E12-T04.
- **Aceite:** lentidão não trava; retry limitado; sem injeção de shell.

### R13 🟠 Rotação/checkpoint de `eventos.jsonl`
- **Estado:** append-only sem teto — cresce indefinidamente.
- **Recomendação:** segmentação (`eventos/<yyyy-mm>.jsonl`) + checkpoint de offset; `compact`
  pode consolidar; leitura mantém dedup on-read e tolerância.
- **Onde:** E03-T09.
- **Aceite:** corpus grande não carrega tudo; leitura pela segmentação.

### R14 🟠 Tetos de memória/disco com eviction
- **Estado:** `.idx/emb_cache` sem eviction; índice em memória sem teto.
- **Recomendação:** cache com `max_bytes`/TTL e eviction LRU; limiar documentado para o índice
  (acima → mmap/streaming ou aviso); `doctor` reporta tamanho de `.idx/` e cache.
- **Onde:** E11-T02; E06-T07.
- **Aceite:** cache respeita teto; índice acima do limiar não OOM.

### R15 🟡 Streaming de I/O e de saída
- **Estado:** `recall`/`prime` podem montar strings grandes.
- **Recomendação:** escrever direto no `BufWriter` do stdout; indexar lendo em streaming;
  `with_capacity` quando o tamanho é conhecido.
- **Onde:** E06-T06; E08-T09.
- **Aceite:** saída grande não cresce memória linearmente.

### R16 🟠 Runtime mínimo
- **Estado:** `mode=lazy, async=true` sugere runtime async; `rust_skill` sugere `tokio` full.
- **Recomendação:** **não** adotar `tokio full`. Worker **bloqueante** + canal bounded para a
  digestão; se HTTP concorrente exigir async, `tokio` com `default-features=false` e features
  mínimas (`rt`, `time`, `sync`, `macros`); inferência local em `spawn_blocking`/thread dedicada.
- **Onde:** E01-T03/T09; E11-T10.
- **Aceite:** `cargo tree` sem runtime pesado; ORT/HTTP isolados no adaptador.

---

## 3. Logs

### R20 🔴 stdout = dados, stderr = logs
- **Estado:** port `Logger` existe, mas a separação não é regra.
- **Recomendação:** regra dura — **stdout só dados** (pipe/JSON); **logs sempre em stderr**;
  em `--json`, nenhum log em stdout; `--log-level`/`RUST_LOG`; `--quiet` silencia stderr.
- **Onde:** E01-T07; E12-T07.
- **Aceite:** `kd … --json 2>/dev/null` é JSON válido; pipe nunca é poluído.

### R21 🟠 Logging estruturado e spans
- **Estado:** port sem semântica de níveis/estrutura.
- **Recomendação:** `tracing` + `tracing-subscriber` (`EnvFilter`) como impl do port; níveis com
  significado documentado; `#[instrument]` em `recall`/`write`/`prime`/`embed`; campos
  estruturados de métrica (candidatos, tempo, orçamento, `pending`, `rrf_k`, cache hit/miss).
- **Onde:** E01-T07; E12-T07.
- **Aceite:** log com campos; métrica por operação.

### R22 🔴 Redação de segredos e corpos
- **Estado:** redaction só em hooks (E12-T04).
- **Recomendação:** redação no **layer de log** (não só hooks): nunca logar corpo de nota, valores
  de `[secrets]`, `Authorization`, tokens; logar ids/contagens; allowlist de campos; teste que
  falha se um segredo plantado aparecer.
- **Onde:** E01-T07; E12-T07; E12-T04.
- **Aceite:** segredo plantado nunca aparece no log; corpo nunca por default.

### R23 🟡 Correlação e retenção
- **Estado:** `context_id` existe; sem trace id.
- **Recomendação:** usar `context_id`/`session_id` como span raiz; logs de CLI são stderr
  (efêmeros); log em arquivo opcional com rotação e tamanho máximo; nunca unbounded.
- **Onde:** E01-T07; E12-T07.
- **Aceite:** operações correlacionadas; arquivo rotaciona.

---

## 4. Tratamento de erros

### R30 🔴 `thiserror` no core, `anyhow` na CLI
- **Estado:** proíbe `unwrap/expect/panic`, mas não define tipos.
- **Recomendação:** `knudge-core` com `enum Error` (`thiserror`, `#[from]`, `#[source]`),
  `#[non_exhaustive]`, `Send + Sync + 'static`; `knudge-cli`/`mcp` com `anyhow` + `with_context`;
  **nunca** `Box<dyn Error>` na API pública do core.
- **Onde:** E01-T06.
- **Aceite:** `source()` encadeia; compila em contexto `Send + Sync`.

### R31 🔴 Códigos de erro estáveis e envelope
- **Estado:** envelope `{success, command, error}` sem taxonomia.
- **Recomendação:** `error: { code, message, retryable, details? }` + `warnings[]`;
  `code` é enum estável (`not_found`, `invalid_input`, `conflict`, `io`, `timeout`, `config`,
  `schema`, `unsafe_blocked`, `internal`); mapa **código → exit code**; mensagens humanas
  congeladas (E12-T02), **códigos** são o contrato de máquina.
- **Onde:** E01-T06; E12-T08.
- **Aceite:** matriz código×exit; agentes distinguem `retryable`.

### R32 🟠 Poison e falhas de lock
- **Estado:** `.lock().unwrap()` proibido, sem padrão.
- **Recomendação:** helper que trata poison com `unwrap_or_else(PoisonError::into_inner)` ou
  converte em `ErrorKind::Poison`, com `warn`; **nunca** `panic`.
  *Nota:* `rust_skill` §11 sugere `.expect("poisoned")`, mas D92 **proíbe** `expect` — adotamos
  `into_inner`.
- **Onde:** E01-T06/T08; E11-T10.
- **Aceite:** envenenamento não derruba o processo.

### R33 🟠 Degradação graciosa e resultados parciais
- **Estado:** leitura tolerante (E09-T05) e RRF degradam; não é regra geral.
- **Recomendação:** canais de retrieval, embeddings e git falham **recolhendo `warnings[]`** e
  retornam resultado parcial; nunca aborta; `strict` (config, D94) promove warning a erro; fila de embeddings
  → `pending`.
- **Onde:** E06-T07; E09-T05; E12-T08.
- **Aceite:** canal desligado retorna resultados + warning.

### R34 🟡 Contexto obrigatório em erros de I/O
- **Recomendação:** anexar sempre `path`/`id`/operação (`with_context`); erro de I/O sem contexto
  é defeito de revisão.
- **Onde:** E01-T06; E03.
- **Aceite:** mensagens incluem path/id.

### R35 🟠 Panic hook e exit
- **Estado:** `panic` negado; EPIPE → exit 0.
- **Recomendação:** `panic = "abort"` no release (sem unwinding, binário menor); `unwind` em
  dev/test; panic hook **redige** e loga em stderr; exit **101** reservado a panic, sem colidir
  com os códigos de erro; documentar que `--json` não emite JSON em panic.
- **Onde:** E01-T03/T06; E12-T08.
- **Aceite:** exit codes documentados e sem colisão.

---

## 5. Estrutura, build e supply chain

- **R40 🟠 Workspace disciplinado:** `edition = "2024"`, `rust-version = "1.97"` (**MSRV
  mínimo obrigatório**), `resolver = "2"`,
  `[workspace.lints]`, `Cargo.lock` commitado, `publish = false`, `default-members`.
- **R41 🟠 Perfis:** release `lto = "fat"`, `codegen-units = 1`, `strip = "symbols"`,
  `overflow-checks = true`, `panic = "abort"`; dev com `debug-assertions`/`overflow-checks`.
- **R42 🟠 Supply chain e ferramentas:** `cargo deny` (licenças/advisories), `cargo audit`,
  `cargo machete`, `typos`, `nextest`, `llvm-cov`/`tarpaulin`, `criterion` (retrieval).
- **R43 🟠 Orçamento de dependências:** `default-features = false`; evitar `tokio full`,
  `reqwest`; preferir HTTP bloqueante no adaptador; documentar cada dependência e o porquê.
- **R44 🔴 Rigor de Clippy:** `clippy.toml` + `[workspace.lints]` com os grupos
  `all/pedantic/nursery/cargo` em deny/warn e curadoria de `restriction` (unwrap/expect/panic,
  `unsafe`, aritmética, `print_*`, `indexing_slicing`, `disallowed_*`, `await_holding_*`);
  análise individual das 95 opções e config pronta em [`15_clippy_config.md`](15_clippy_config.md)
  e [`clippy.toml`](clippy.toml).
- **R45 🔴 Invariantes de retenção e reescrita (D154–D159):** toda feature que toca shelf-life,
  ranking, curadoria ou reescrita deve respeitar: (1) **default identidade** — no-op
  byte-a-byte até o usuário ligar a config, **inclusive sem criar derivados** (com
  `retention.renew_on_use=false`, `ask`/`rewind` não gravam nem `.idx/usage.jsonl`);
  (2) **renovação de uso só estende** retenção, nunca encurta; (3) **supersessão vence
  evidência** — contagem/confiança sombreia ranking, não decide se a correção entra;
  (4) **acesso ≠ evidência** — "ainda é usado" e "ainda é verdade" são eixos separados;
  (5) **transformação não deleta a fonte** — supersessão/merge/decay/compact só propõem ou
  reescrevem a linhagem, e `purge_derived` (D84) toca só `.idx/`; a **única** remoção do
  canônico é o `forget --purge` explícito, após tombstone + retenção, com detach de referrers
  (D32/D48/D84); (6) **toda varredura executada deixa relatório** (`warnings[]`/`--json`,
  D47/R33); (7) **leitura nunca escreve nota** (só derivados em `.idx/`). Checklist no DoD de
  cada PR. Detalhe em [`plan/proposals/melhorias_ai_memory.md`](../proposals/melhorias_ai_memory.md);
  travado por [`crates/knudge-cli/tests/invariants.rs`](../../crates/knudge-cli/tests/invariants.rs).

**Onde:** E01-T03 (R40/R41/R43/R44), E01-T08 (R01/R44), E13-T07 (R44), E13-T09 (R42).

---

## 6. Matriz de aplicação épico → Rn

| Épico | Achados |
|---|---|
| E01 | R01, R02, R03, R05, R20, R21, R22, R23, R30, R31, R32, R34, R35, R40–R44 |
| E02 | R02, R04 |
| E03 | R05, R10, R13, R34 |
| E04 | R05 (symlink/nofollow), R12 |
| E05 | R04 |
| E06 | R04, R14, R15, R33 |
| E07 | R31, R33 |
| E08 | R04, R15, R33 |
| E09 | R10, R33 |
| E10 | R33 |
| E11 | R11, R12, R14, R16, R32 |
| E12 | R12, R20, R21, R22, R23, R31, R33, R35 |
| E13 | R01, R11, R42, R44 |

## 7. Novas tarefas a incorporar (resumo)

| Tarefa | Épico | Entrega |
|---|---|---|
| E01-T03 (amp.) | E01 | lints de workspace, perfil, supply chain |
| **E01-T06** | E01 | modelo de erro (`ErrorKind`, thiserror/anyhow, poison) |
| **E01-T07** | E01 | logging/observabilidade (port + política + redação) |
| **E01-T08** | E01 | política de memória e `unsafe` |
| **E01-T09** | E01 | política de recursos (limites, timeouts, runtime) |
| **E03-T08** | E03 | RAII + varredura de resíduos |
| **E03-T09** | E03 | rotação/checkpoint de eventos |
| **E06-T07** | E06 | orçamento de índice + resultados parciais |
| **E08-T09** | E08 | orçamento/streaming de saída |
| E11-T02 (amp.) | E11 | eviction do cache |
| **E11-T10** | E11 | runtime/backpressure/timeout |
| **E12-T07** | E12 | stdout vs stderr + logging estruturado |
| **E12-T08** | E12 | envelope de erro + mapa de exit |
| **E13-T08** | E13 | Miri/loom/fuzz |
| **E13-T09** | E13 | supply chain, cobertura e benchmark |
| **E13-T07** (amp.) | E13 | gate com `clippy.toml`, `nextest`, `deny` |

## 8. Riscos e trade-offs — o que **não** fazer

- **Não** adotar `tokio full`/`reqwest`/`axum` por padrão (R16/R43): o knudge é local.
- **Não** trocar o núcleo puro por `tracing` global sem o port: perderíamos testes determinísticos
  de log/redação (R20–R22).
- **Não** usar `Box<dyn Error>` na API do core (R30): quebra `Send + Sync` e o mapa de código.
- **Não** “consertar” a sugestão do `rust_skill` de `.expect("poisoned")` ignorando D92 (R32):
  usar `into_inner` + `warn`.
- **Não** logar corpo de nota nem segredo “por conveniência” (R22): a redação é um teste.
- **Não** deixar `eventos.jsonl`/cache/índice crescerem sem teto (R13/R14): `doctor` mede.

## 9. Seeds de implementação (referência `rust_skill/`)

| Tema | Arquivo |
|---|---|
| Estratégia de erro, combinators, `main -> Result` | `rust_skill/06-error-handling.md` |
| `Arc`/`Mutex`, poison, `Send/Sync`, message passing | `rust_skill/11-concurrency.md` |
| cancellation safety, backpressure, timeouts | `rust_skill/12-async.md` |
| `unsafe`, `// SAFETY`, Miri, FFI | `rust_skill/14-unsafe-macros-ffi.md` |
| lints, perfis, workspaces, features | `rust_skill/15-cargo-deep.md` |
| `tracing`, lib+bin, CI, `cargo deny` | `rust_skill/16-project-practical.md` |
| `try_reserve`, `with_capacity`, `&str`/`Cow` | `rust_skill/05-collections.md` |
| RAII/Drop, move vs clone | `rust_skill/03-ownership.md`, `rust_skill/10-smart-pointers.md` |
| nextest, proptest, cobertura | `rust_skill/13-testing.md` |
