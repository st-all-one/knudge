# 12 — Erros, logs e determinismo

Erros são **valores** (`Result<T, Error>`), logs vão para stderr via porta e o domínio é
determinístico. Especificação: [`../specs/erros.md`](../specs/erros.md) e
[`../specs/determinismo.md`](../specs/determinismo.md).

## Erros e exit codes (R30–R35)

```rust
use knudge_core::{Error, ErrorKind, Result};

fn ler(id: &str) -> Result<String> {
    let note = store.read(id)?;                 // propaga Error
    Ok(note.body)
}

match ler("fact_x") {
    Ok(body) => { /* ... */ }
    Err(error) => {
        let kind = error.kind();               // ErrorKind (contrato de máquina)
        eprintln!("código={} exit={} retry={}", kind.code(), kind.exit_code(), error.retryable());
    }
}
```

| `ErrorKind` | `exit_code` |
|---|---|
| `NotFound` | 3 |
| `InvalidInput` | 2 |
| `Conflict` | 4 |
| `Io` | 5 |
| `Timeout` | 6 |
| `Config` | 7 |
| `Schema` | 8 |
| `UnsafeBlocked` | 9 |
| `Internal` | 70 |

(101 é reservado a panic; 0 é sucesso.)

- `ErrorKind::code()` é o **contrato** (string estável); `exit_code()` é para processos.
- `retryable()` é `true` só para `Timeout`.
- Construtores nomeados: `Error::not_found`, `invalid_input`, `conflict`, `timeout`, `config`,
  `schema`, `internal`. **I/O sempre** com `Error::io(path, source)` (contexto de path — R34).
- Nunca use `Box<dyn Error>` na sua API pública; use `knudge_core::Error`.

### Poison de mutex

```rust
use knudge_core::lock_or_recover;
let guard = lock_or_recover(&mutex); // recupera o estado mesmo após panic
```

Nunca `.expect("poisoned")`.

## Degradação graciosa e `strict` (R33/D94)

Funções que agregam recursos opcionais devolvem **resultado parcial + `warnings[]`**:

```rust
let out = recall(&index, &graph, &query)?;    // out.warnings pode ser não-vazio
for aviso in &out.warnings { log.warn(&aviso); }
```

Com `behavior.strict = true` (na config), o aviso vira `Error` (o campo `query.strict` reflete
isso). Em serviço, prefira propagar avisos a falhar.

## Logs (R20–R22)

**stdout = dados; stderr = logs.** O core não imprime; você injeta a porta `Logger`.

```rust
use knudge_core::ports::{Level, LogRecord, Logger};

struct MeuLogger;
impl Logger for MeuLogger {
    fn log(&self, record: &LogRecord<'_>) {
        // escreva no seu destino de log (nunca stdout)
        eprintln!("[{:?}] {} {:?}", record.level, record.message, record.fields);
    }
}
```

- `LogRecord { level, message, fields }`; `Level` = `Error|Warn|Info|Debug|Trace`.
- **Nunca** logue corpo de nota/segredos; logue ids, contagens e métricas.
- `logging::Redactor` substitui segredos por `[REDACTED:<tipo>]` (D159):

```rust
use knudge_core::logging::Redactor;

let redator = Redactor::new(config.get_str("secrets.api_token").map(str::to_string));
let seguro = redator.redact(&texto_livre); // Cow<str>
```

No seu adapter de log, aplique o `Redactor` **antes** de emitir.

## Determinismo total com fakes

Para testar sem tocar SO/tempo/ambiente, use os fakes e a API pura:

```rust
use knudge_core::ports::fakes::{FixedClock, FakeEmbedder, MemFs, RecordingLogger, SeqRng};
use knudge_core::ports::Clock;
use knudge_core::store::Store;
use knudge_core::retrieval::Index;
use knudge_core::write::{Draft, WriteContext, write};
use knudge_core::store::EventLog;
use knudge_core::schema::NoteType;

let fs = MemFs::new();
let mut clock = FixedClock::default();
clock.set(knudge_core::Timestamp::from_millis(1_700_000_000_000));

let store = Store::new(&fs, "/proj/.knudge");
store.ensure_dirs()?;
let events = EventLog::new(&fs, "/proj/.knudge", EventLog::DEFAULT_MAX_BYTES);
let index = Index::build(&[])?;
let ctx = WriteContext::new(store, events, index, clock.now().as_millis());

let out = write(&ctx, &Draft::new(NoteType::Fact, "A nota é a verdade"), &Default::default())?;
assert_eq!(out.action, knudge_core::write::WriteAction::Created);
```

| Fake | Substitui |
|---|---|
| `FixedClock` | `Clock` (tempo determinístico) |
| `SeqRng` | `Rng` (jitter determinístico) |
| `MemFs` / `FaultyFs` | `Fs` (I/O em memória; injeção de falha) |
| `FakeEnv` / `FakeGit` | `Env` / `Git` |
| `RecordingLogger` / `NoopHookRunner` | `Logger` / `HookRunner` |
| `FakeEmbedder` | `Embedder` |

### Testes

- Use `?` em testes que retornam `Result`; evite `unwrap`/`expect`/`panic`.
- `proptest` para funções puras (normalize, ids, TOON, RRF, decay): invariantes e idempotência.
- **Golden** para contrato de bytes (TOON, hash, exit codes, mensagens).
- `catch_unwind` + `resume_unwind` para testar poison de mutex (sem `panic!`).

## Recomendações

- **Mapeie erros, não vaze detalhes.** Converta `Error` para sua taxonomia na borda, preservando
  `kind()`.
- **`retryable()` + backoff** para `Timeout`; não repita `InvalidInput`.
- **Nunca misture log com stdout**; em `--json`, stdout é só o envelope.
- **Redija na borda**, no adapter de log — o domínio nunca vê segredo.
- **Determinismo é requisito**, não opção: nada de `SystemTime::now`/`env::var` no seu código de
  domínio; injete portas.
