# Saúde e validação

O corpus **não apodrece em silêncio**: sabe-se o que está quebrado, o que é *stale* e por quê, e
o reparo do reversível é automático (`doctor --fix`). Uma nota ruim **nunca derruba um comando**.

- Código: `crates/knudge-core/src/health/`
- Decisões: D16–D19, D46, D48, D52, D54/D55, D86, D99, D119, D156, D162, D163, D191

## Princípios (D16–D18)

- Chave desconhecida no read: **tolera** com *warning*.
- Tipo desconhecido: **rejeita a nota** (por-nota, com erro explícito — não derruba a leitura).
- Nota/linha malformada: **skip com warning + orientação de correção**.
- `doctor --fix`: **corrige o reversível** (hash, âncoras quebradas, locks stale, duplicatas — D19).

## Validators (D54/D99/D156)

- Catálogo em `.knudge/validators.toml` (subset TOML próprio, D97).
- Cada validator: `cmd` (obrigatório), `scope` (globs), `severity` (`error|warn|info`),
  `timeout` (ms, default 120000) e `kind` (`check|gate`).
- Resolução: `checks(task) = explícitos ∪ globals ∪ por_âncora(anchors(task))`; validator
  explícito ausente vira `missing[]` (não é erro fatal).
- A **execução** fica na borda (`HookRunner`, E12); o núcleo só resolve e descreve.
- **Gate** (D156): stdin `{op,before,after}` → stdout `{passed,score_before,score_after}`. A
  decisão pura (`accept`) fica no core; `learn`/`compact --verify` anexam o veredito.

## `doctor` (D163)

`kd doctor [--fix] [--explain]` roda os **13 checks + auditoria** num só relatório:

- `healthy` só com **zero achados**; advisórios viram `degraded` (nunca "saudável" silencioso).
- `--explain` detalha cada achado (`esperado`/`encontrado`/`ação`).
- `--audit` deixou de existir (fundido no default).
- Checks notáveis: `integrity` (referências/bidirecionalidade + **claims/ontologia** — D207),
  `body` (corpo + **slots mínimos** — D162/D191, advisório), `program-anchor` (D119, warn),
  `Contradictions`, `duplicates`, `stale locks`.

## Auditoria (D46)

`audit.rs` produz um relatório de integridade/saúde do corpus: referências quebradas, órfãos,
bidirecionalidade `replaces ↔ superseded_by`, duplicatas, âncoras quebradas.

## Evidência e fechamento (D48/D55)

`evidence.rs`: fechamento por evidência — roda os validators e infere o `outcome`. `CheckOutcome`/
`CheckResult`/`CloseOutcome`; `close_task` fecha com lastro.

## Leitura tolerante (D16–D18)

`tolerant.rs`: `read_tolerant`/`read_note_tolerant` devolvem `TolerantRead` com `SkippedNote[]` —
nada é perdido em silêncio; o skip é reportado com orientação.

## Âncoras (D86)

- `anchors/verify.rs`: `content_hash` derivado + **verify-on-hit** — `cited` invalida, `context`
  não; nota *stale* é **sinalizada, não apagada**.
- `anchors/store.rs`: registro derivado em `.idx/anchors.jsonl` (nunca no frontmatter).

## Portão de propostas (D156)

`gate.rs::accept(outcome, min_delta)` é a decisão **pura** sobre um `GateOutcome`; a borda
(`proposals.gate`/`min_delta`/`enforce`) aplica. `enforce=true` faz o `pre-record` bloquear com
`conflict` (exit 4). Gate ausente/timeout/JSON inválido **degrada com aviso** (R33).

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Catálogo/resolução de validators | `health/validator/` |
| `doctor` + checks + fix | `health/doctor/` |
| Auditoria | `health/audit.rs` |
| Evidência | `health/evidence.rs` |
| Portão | `health/gate.rs` |
| Leitura tolerante | `health/tolerant.rs` |
| Âncoras | `health/anchors/` |

## Testes

`health/tests/` (doctor, doctor_conflict, audit, evidence, anchors, body, tolerant, validator).
`DIVERGENCES.md` #100 (slots), #110 (claims).
