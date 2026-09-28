# D176 — Consistência de status na leitura

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Consistência de status na leitura.** `Status::VISIBLE` (`schema/types.rs`) é a **fonte única** do default (D43): `CorpusScope::select` (`knowledge rank`/`map`, `rewind`, `maintenance learn/compact/prune`) e `ask` excluem `Forgotten \| Superseded`; `Status::is_deprecated()` centraliza o teste. `--status` explícito continua inspecionando linhagem. Teste de regressão por consumidor (`real_usage`); `DIVERGENCES.md` #95. Fecha E16-T02.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #95.
- Fecha E16-T02.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
