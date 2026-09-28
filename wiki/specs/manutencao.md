# Manutenção

Operações de **passado** (auditoria) e de **consolidação**. Nada aqui escreve sem aceite
(D33/D47): `diff` lê a auditoria, `learn` e `compact` **propõem**.

- Código: `crates/knudge-core/src/maintenance/`
- Decisões: D33, D47, D111, D112, D156

## `diff` (D33)

`diff.rs` lê a **auditoria de eventos** (`EventLog`) e devolve `DiffEntry[]` — o que mudou no
corpus num intervalo. Não escreve.

## `learn` (D33/D111)

`learn.rs` propõe, de forma **determinística**, a partir de eventos + âncoras (`LearnInput` →
`LearnProposal[]`):

| `LearnKind` | Sinal |
|---|---|
| `CreateNote` | atividade sem registro (write-gap) — inclui tarefa fechada sem nota (D111) |
| `Merge` | quase-duplicata |
| `Supersede` | substituição |
| `Link` | lacuna de grafo (aresta sugerida) |

Read-only (D47). Não usa diff global.

## `compact` (D47)

`compact.rs` propõe fusões (`CompactProposal`) com estratégia:

| `CompactStrategy` | Efeito |
|---|---|
| `Concat` | concatena corpos |
| `KeepLatest` | mantém a mais recente |
| `MergeOutcomes` | funde outcomes |

`apply_compact(ctx, proposal)` materializa a proposta **após aceite** (escreve via o protocolo de
`write`).

## Portão de evidência (D156)

`learn`/`compact --verify` anexam um veredito **read-only** (`gate=passed|failed` no pipe;
`gate` no `--json`), produzido pelos validators de `kind="gate"`. A decisão pura (`accept`) fica
em `health/gate.rs`; com `proposals.enforce=true`, o `pre-record` bloqueia com `conflict` (4).

## Prune (D112/D177/D208)

`kd maintenance prune` **propõe** `forget|id|motivo` a partir de `lifecycle::plan::demotion_candidates`
(shelf-life, decay, contradição, TMS, drift), excluindo membros de ciclo (D45) e sem gravar.
A aplicação é sempre via `kd forget` (D112).

## Escopo obrigatório (D144)

`learn`/`compact`/`prune` exigem `--tag`/`--anchor`/`--type`/`--class`/`--scope` ou `--universe`
(D143/D144); sem escopo → exit 2 (D130).

## Onde vive

| Aspecto | Arquivo |
|---|---|
| `diff` | `maintenance/diff.rs` |
| `learn` | `maintenance/learn.rs` |
| `compact` + `apply` | `maintenance/compact.rs` |

## Testes

`maintenance/tests/` (diff, learn, compact).
