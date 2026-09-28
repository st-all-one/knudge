# D212 — Conjuntos fechados: lista + sugestão da mais provável

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**. Linhagem: Revisa D158/D210.

## Decisão

**Conjuntos fechados: lista + sugestão da mais provável.** Toda flag/campo de valor pré-definido
(`--type`/`--class`/`--status`/`--scope`/`--kind`/`--outcome`, arestas `--link`/`--edge`/`--via`,
`--relation`, `--axis`, `--sort`, `--template`, `self setup`, `self completions`, `--log-level`,
`config --key`) **rejeita valor inválido** com a **lista das possibilidades** e a **sugestão da
mais provável** (distância de Levenshtein determinística, `schema/suggest.rs`). Flag **ausente**
não valida (sem erro nem lista). `--log-level` deixa de cair em `warn` silencioso (passa a
`invalid_input`, exit 2); `config --key` sugere a chave mais próxima.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #115.
- Testes: `schema::tests::{unknown_enum_lists_options_and_suggests_the_closest, suggest_distance_and_closest_are_deterministic}`,
  `tests/closed_sets.rs`, `improvements_032::ask_suggest_invalid_relation_is_rejected`.
- Revisa D158/D210.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
