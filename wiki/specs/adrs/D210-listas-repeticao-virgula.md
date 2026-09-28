# D210 — Listas na CLI: repetição + vírgula

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**. Linhagem: Revisa D140/D147/D209.

## Decisão

**Listas na CLI: repetição + vírgula (padrão único).** Toda flag de valor múltiplo com semântica
de **seleção** (`--id`, `--type`, `--class`, `--tag`, `--anchor`, `--edge`, `--claim`, `--checks`,
`--files`) aceita **repetição** (`--tag a --tag b`) e **lista com vírgula** (`--tag a,b`), via
`value_delimiter = ','` — as duas formas são equivalentes (teste `tests/list_args.rs`). O formato
por **espaço** deixa de existir (remove-se `num_args = 1..`), por competir com o posicional e gerar
descarte silencioso. Flags de **texto livre** (`[QUERY]`/corpo, `--step`, `--summary`, `--note`,
`--message`) **não** são divididas. Modos que não usam query (`ask --id`/`--around`) passam a
**conflitar** com a query textual (erro, exit 2) em vez de ignorá-la, inclusive via `--params`.
Entrada estruturada/array segue por `--params '<json>'`.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #113.
- Testes: `crates/knudge-cli/tests/list_args.rs`.
- Revisa D140/D147/D209.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
