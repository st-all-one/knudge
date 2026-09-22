# Arquitetura do knudge

> Registro da separação **núcleo puro + portas + adaptadores** (D65) e dos escopos temáticos
> (D66/D68). Complementa `plan/00_panorama.md` §2 e o grafo de dependências de
> `plan/implementation/README.md`.

## 1. Princípio

O domínio **não conhece** terminal, `argv`, relógio global, RNG global nem sistema de arquivos.
Todo acesso ao mundo externo atravessa uma **porta** (`knudge_core::ports`). Isso permite:

- testar o domínio só com **fakes** (`ports::fakes`), reprodutível byte a byte;
- trocar implementação (real ↔ fake ↔ remota) sem tocar no domínio;
- confinar `unsafe`, I/O impuro e dependências de SO a poucos módulos.

## 2. Camadas e crates

```
knudge-mcp ──┐
             ├── knudge-core (núcleo puro + portas + adaptadores std)
knudge-cli ──┘
```

| Crate | Papel | Pode conter |
|---|---|---|
| `knudge-core` | Modelo, schema, retrieval, ciclo de vida e **portas** | Lógica pura; `adapters` (std) isolado |
| `knudge-cli` | Binário `kd`; monta adaptadores e escreve a saída | `clap`, `tracing`, I/O de terminal |
| `knudge-mcp` | Servidor MCP reativo (E12-T03) | Protocolo MCP; nada de domínio |

**Regra:** os adaptadores **não** são dependência do domínio. `knudge-core::adapters` existe para
conveniência, mas só `cli`/`mcp` o importam.

## 3. Portas (`knudge-core::ports`)

| Porta | Fornece | Fake |
|---|---|---|
| `Clock` | instante UTC (`Timestamp`) | `FixedClock` |
| `Rng` | aleatoriedade (só jitter) | `SeqRng` |
| `Env` | variáveis e argumentos | `FakeEnv` |
| `Fs` | leitura/escrita atômica | `MemFs` |
| `Git` | worktree, status, `--common-dir` | `FakeGit` |
| `HookRunner` | hooks externos (sem shell) | `NoopHookRunner` |
| `Logger` | log estruturado (stderr) | `RecordingLogger` |

## 4. Escopos temáticos (`knudge-core`)

| Módulo | Responsabilidade | Épico |
|---|---|---|
| `error` | `Error`/`ErrorKind`, mapa código→exit, poison | E01 |
| `time` | `Timestamp` UTC com milissegundos (D07) | E01/E02 |
| `logging` | redação de segredos (R22) | E01 |
| `ports` | traits + fakes | E01 |
| `adapters` | implementações `std` | E01+ |
| `schema` | schema canônico, tipos, IDs | E02 |
| `toon` | parser/emissor TOON | E02 |
| `jsonl` | eventos append-only, streaming | E03 |
| `git` | worktree, persistência, `sync` | E04 |
| `retrieval` | BM25, âncoras, RRF | E06 |
| `lifecycle` | decay, confiança derivada, clusters | E10 |
| `embeddings` | provedor plugável e fila lazy | E11 |

## 5. Fluxo de uma operação

```
kd <verbo>
  → knudge-cli: parsing (clap) + envelope (--json)
  → monta adaptadores (SystemClock, StdFs, StdEnv, TracingLogger, …)
  → chama o domínio (knudge-core) que só usa ports
  → saída: stdout = dados | stderr = logs
  → exit code = ErrorKind::exit_code() (101 reservado a panic)
```

## 6. Invariantes de engenharia

- `#![forbid(unsafe_code)]` em `core`/`cli`/`mcp` (R01).
- Sem `Rc`/`RefCell` no core; estado compartilhado via `Arc<Mutex<_>>` (R03).
- Sem `unwrap`/`expect`/`panic` em `src/` (D92); poison com `into_inner` (R32).
- Arquivos de produção ≤ 300 linhas (D92).
- `clippy -D warnings` lendo `clippy.toml` (R44); perfis e supply chain (R40–R43).

## 7. Referências

- Visão: `plan/00_panorama.md`
- Decisões: `plan/03_decisoes-fechadas.md` (D01–D94)
- Políticas de engenharia: `plan/implementation/14_revisao_tecnica.md` (R01–R44)
- Superfície CLI: `plan/implementation/16_cli_surface.md`
