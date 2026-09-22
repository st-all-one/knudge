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
| `Fs` | leitura/escrita atômica, append, create-exclusive, rename, `fsync`, mtime | `MemFs`, `FaultyFs` |
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
| `jsonl` | leitura/escrita JSONL + codec JSON canônico | E03 |
| `store` | notas (`notas/`), eventos, lock, rebuild, purge, sweep | E03 |
| `git` | worktree, persistência, `sync` | E04 |
| `retrieval` | BM25, âncoras, RRF | E06 |
| `lifecycle` | decay, confiança derivada, clusters | E10 |
| `embeddings` | provedor plugável e fila lazy | E11 |

## 5. Persistência (E03)

`notas/` é a verdade; `eventos/` é auditoria; `.idx/` é derivado e reconstruível.

| Peça | Regra |
|---|---|
| Escrita de nota | tmp + rename no mesmo diretório (D20); `fsync` em batch (D22) |
| Ordem de commit | **nota antes do evento** (D21); crash deixa nota sem evento |
| Lock | advisory por arquivo-alvo, stale 30 s, reclaim por rename (D23–D25) |
| Evento | `{id, op, note_id?, at, actor?, data?}`; `id` derivado do conteúdo |
| Dedup | on-read por `id` (D26/D28), idempotente sob `merge=union` (D31) |
| Rotação | `eventos/events.jsonl` ativo → `events-NNNN.jsonl` acima de `max_bytes` (R13) |
| Checkpoint | `.idx/events.checkpoint` guarda o último evento processado |
| Remoção | canônico primeiro, depois purga do derivado (D84) |
| Rebuild | double-buffer `.idx.new/` + rename atômico (D27) |
| Resíduos | `*.tmp`/`*.lock` velhos removidos no start, com `warn` (R10) |

`revision` é **contador de versões** (default 1; cada `update` incrementa) — sinal de
volatilidade, não CAS (D48).

## 6. Fluxo de uma operação

```
kd <verbo>
  → knudge-cli: parsing (clap) + envelope (--json)
  → monta adaptadores (SystemClock, StdFs, StdEnv, TracingLogger, …)
  → chama o domínio (knudge-core) que só usa ports
  → saída: stdout = dados | stderr = logs
  → exit code = ErrorKind::exit_code() (101 reservado a panic)
```

## 7. Invariantes de engenharia

- `#![forbid(unsafe_code)]` em `core`/`cli`/`mcp` (R01).
- Sem `Rc`/`RefCell` no core; estado compartilhado via `Arc<Mutex<_>>` (R03).
- Sem `unwrap`/`expect`/`panic` em `src/` (D92); poison com `into_inner` (R32).
- Arquivos de produção ≤ 300 linhas (D92).
- `clippy -D warnings` lendo `clippy.toml` (R44); perfis e supply chain (R40–R43).

## 8. Referências

- Visão: `plan/00_panorama.md`
- Decisões: `plan/03_decisoes-fechadas.md` (D01–D96)
- Contrato de bytes: `TOON.md`
- Políticas de engenharia: `plan/implementation/14_revisao_tecnica.md` (R01–R44)
- Superfície CLI: `plan/implementation/16_cli_surface.md`
