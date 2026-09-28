# Determinismo, concorrência e crash

O knudge trata **determinismo como requisito**, não como bônus: a mesma entrada produz a mesma
saída, byte a byte, em qualquer execução. Isso é o que permite goldens, proptest e handoff
reprodutível.

- Decisões: D65, D92, D95, D155, D204, R01–R05, R33
- Bordas: [`DIVERGENCES.md`](DIVERGENCES.md)

## Garantias de determinismo

- **Ordem estável:** toda iteração sobre coleções tem ordem canônica. `HashMap`/`HashSet` são
  **proibidos** (use `BTreeMap`/`IndexMap`); `LinkedList` também.
- **Tempo via porta:** `Clock` (nunca `SystemTime::now`/`Instant::now`). `Timestamp` UTC em ms.
- **RNG via porta:** `Rng` (só para jitter); o sistema nunca decide por aleatoriedade global.
- **Ambiente via porta:** `Env` (nunca `env::var`).
- **FS via porta:** `Fs` (I/O atômico, sem seguir symlink).
- **Ordenações explícitas:** tie-breaks documentados — RRF `(score desc, id asc)` (D81), rank
  `(confidence desc, id asc)` (D107), impacto `(impacto desc, created asc, id asc)` (D109),
  sugestões `(score desc, from asc, to asc)` (D158).
- **Algoritmos determinísticos:** PageRank/PPR (ordem canônica dos ids — D192), Louvain (empate
  mantém a comunidade corrente — D193), MinHash/LSH (FNV-1a + splitmix64 — D204).
- **Views:** a estática (`compute_views`) é byte-idêntica; a dinâmica considera `now_ms`
  (`compute_views_at`) — usada só onde o tempo importa.

## Fakes (D65)

Testes do core **nunca** tocam o SO: usam as portas fake.

| Porta | Fake |
|---|---|
| `Clock` | `FixedClock` |
| `Rng` | `SeqRng` |
| `Env` | `FakeEnv` |
| `Fs` | `MemFs` |
| `Git` | `FakeGit` |
| `HookRunner` | `NoopHookRunner` |
| `Logger` | `RecordingLogger` |
| `Embedder` | `FakeEmbedder` |

## Concorrência

- Estado compartilhado via `Arc<Mutex<_>>` (sem `Rc`/`RefCell` — R03).
- **Lock advisory** por arquivo-alvo, ordem documentada (externo = container, interno = nota) para
  evitar deadlock ABBA (D25); liberação RAII.
- Dois processos (CLI + MCP) convivem: o lock + atomicidade protegem a escrita concorrente.
- `lock_or_recover` trata poison sem panic.
- **Multi-dev** (D153): verdade = notas (arquivo-por-nota → merge natural); índice derivado
  (reconstruir); eventos e cache vetorial por `merge=union` + dedup; conflito de nota é **pulado e
  reportado** pelo `doctor`, nunca auto-mergeado.

## Crash-injection

`FaultyFs` (`ports/fakes/fs/faulty.rs`) injeta falhas de escrita para provar que:

- tmp + rename sobrevive a crash no meio da escrita (D20);
- o rebuild double-buffer (`.idx.new/` + rename) deixa o leitor ver o índice antigo **ou** o novo
  (D27);
- a ordem nota→evento não gera estado inconsistente (D21).

## Verificação

- `make miri` — verificação dinâmica de UB no core puro.
- `make fuzz` — alvos de fuzz (TOON, JSONL).
- proptest nas funções puras (normalize, IDs, TOON, RRF, decay, confiança).
- `DIVERGENCES.md` cataloga cada borda (Unicode, ordem, lock, atomicidade, TOON) **com o teste que
  a trava**.

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Portas/fakes | `crates/knudge-core/src/ports/` |
| Adaptadores reais | `crates/knudge-core/src/adapters/` |
| Lock | `store/lock.rs` |
| Atomicidade/rebuild | `store/{commit,rebuild}.rs` |
| Crash-injection | `ports/fakes/fs/faulty.rs` |
