# 08 — Saúde e manutenção

Saúde (`doctor`/`audit`/validators) e manutenção (`diff`/`learn`/`compact`) são **somente
leitura ou proponentes** — nunca mudam o corpus sem ação explícita (D33/D47/D163).
Especificação: [`../specs/saude.md`](../specs/saude.md) e [`../specs/manutencao.md`](../specs/manutencao.md).

## `doctor`: 13 checks + reparo

```rust
use knudge_core::health::{doctor, doctor_fix, DoctorInput};

let store = kd.store();
let events = kd.events();
let graph = kd.graph()?;
let thresholds = kd.thresholds()?;

let input = DoctorInput {
    fs: kd.fs_dyn(),
    root: &kd.knowledge_dir(),
    project_root: kd.project_root(),
    store: &store,
    events: &events,
    config: kd.config(),
    graph: &graph,
    now_ms: kd.now_ms(),
    lock_stale_ms: 30_000,
    thresholds: &thresholds,
};

let report = doctor(&input)?;          // somente leitura
for check in &report.checks { /* check.id, check.status, ... */ }

let repaired = doctor_fix(&input)?;    // idempotente: body_hash, âncoras, locks, índice (D19/D84)
```

`doctor_fix` corrige `body_hash`, âncoras quebradas, locks stale e índice divergente — e é
**idempotente**.

## `audit`: leitura pura

```rust
use knudge_core::health::{audit, AuditInput};

let index = kd.index()?;
let report = audit(&AuditInput {
    fs: kd.fs_dyn(),
    root: &kd.knowledge_dir(),
    project_root: kd.project_root(),
    store: &store,
    graph: &graph,
    index: &index,
    now_ms: kd.now_ms(),
    lock_stale_ms: 30_000,
    thresholds: &thresholds,
})?;
```

Auditoria acusa integridade, ciclos, âncoras quebradas, duplicatas, arestas sugeridas faltantes e
locks stale (D46).

## Validators e fechamento por evidência (D54/D55)

```rust
use knudge_core::health::validator::ValidatorCatalog;
use knudge_core::health::{close_task, CheckOutcome, CheckResult, Severity};

let catalogo = ValidatorCatalog::load(kd.fs_dyn(), &kd.knowledge_dir())?;
let nomes: Vec<&str> = catalogo.iter().map(|(name, _)| name).collect();

// você executa os comandos na borda e reporta os resultados
let resultados = vec![CheckOutcome {
    name: "cargo test".into(),
    result: CheckResult::Pass,
    severity: Severity::Info, // Error | Warn | Info
    duration_ms: Some(1234),
    output: None,
}];

let fechamento = close_task(&kd.write_context()?, "task_x", &resultados, Some("agente"))?;
println!("outcome inferido: {:?}", fechamento.status);
```

Regras: `checks` = explícitos ∪ globais ∪ por âncora (D54); `close_task` **exige** evidência; o
`outcome` é inferido pela severidade. Sem evidência, não fecha.

## Gate de proposta (D156)

```rust
use knudge_core::health::{GateOutcome, accept};

let veredito = GateOutcome { passed: true, score_before: 0.70, score_after: 0.75 };
if accept(&veredito, 0.02) { /* aplica a transformação */ }
```

O core decide; executar o comando externo é papel da borda (via `HookRunner`, sem shell).

## Leitura tolerante

```rust
use knudge_core::health::{read_tolerant, read_note_tolerant};

let read = read_tolerant(&store)?;          // { notes, skipped, warnings }
let one = read_note_tolerant(&store, id)?;  // nota ou skip + orientação
```

Uma nota malformada **não** derruba a leitura: é pulada com orientação (D16–D18).

## Manutenção

```rust
use knudge_core::maintenance::{diff, learn, LearnInput, propose_compact, apply_compact};

let (events, _) = kd.events().read_all()?;
let mudancas = diff(&events, None, None, None, &graph);

let propostas = learn(&LearnInput {
    index: &index,
    graph: &graph,
    events: &events,
    changed_paths: &changed,
    scope: None,
    thresholds: &thresholds,
}); // create_note / merge / supersede / link — nunca escreve (D33/D47)

let compact = propose_compact(&store, &index, &thresholds);
if approved { apply_compact(&ctx, &compact[0])?; }
```

## Recomendações

- **Off-path.** Rode doctor/audit/manutenção fora do caminho quente de `ask`/`write`.
- **`doctor_fix` é seguro para repetir**, mas trate `warnings` de contexto.
- **Evidência sempre.** `close_task` sem resultados é erro; capture `output` (nunca corpo de
  nota).
- **Gate antes de aplicar** compact/learn: o core só propõe; você decide e aplica.
- **Não derive `outcome` na mão** — use `infer_outcome`/`close_task`.
- **Mantenha `.idx/` fresco** para que `doctor`/`corpus` não reconheçam divergência espúria.
