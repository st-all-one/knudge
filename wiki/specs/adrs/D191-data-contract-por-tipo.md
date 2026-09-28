# D191 — Data contract por tipo

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Data contract por tipo (soft).** O `write` e o `doctor` conferem **slots mínimos de corpo** por espécie (`decision`→Alternativas/Por quê/Consequência; `error`→Causa/Correção; `risk`→Probabilidade/Impacto; `def`→Significado; `snippet`→Linguagem+âncora; `question`→âncora/`depends_on`), casados por cabeçalho/rótulo com fold de diacríticos (D172), em PT-BR ou inglês. Sem chave nova e sem bump de `schema_version` (R1/D14). **Soft**: aviso em `write` (`warnings[]`), `invalid_input` só sob `behavior.strict`; `--dry-run` expõe `missing_slots`; o check `body` do `doctor` (D162) passa a contar slots ausentes e o `status` fica `degraded` (advisório — `healthy` intacto). `fact` sem lastro segue coberto por D162. `DIVERGENCES.md` #100. Fecha E19-T03.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #100.
- Fecha E19-T03.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
