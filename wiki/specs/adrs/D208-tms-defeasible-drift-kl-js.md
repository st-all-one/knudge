# D208 — TMS/defeasible + drift KL/JS

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**TMS/defeasible + drift KL/JS** (fecha E19-T10). **TMS/ATMS:** `graph/tms.rs` deriva, do índice reverso de `depends_on`, os **dependentes transitivos** de premissas retratadas (`forgotten`/`superseded`) e os derrotados por `replaces` — sem apagar nada (D14). **Defeasible:** o lado substituído/contradito é derrotado, não removido. **Drift:** `lifecycle/term_drift.rs` compara a distribuição de termos da metade antiga × nova (por `created_at`) de cada tópico (âncora) por **Jensen-Shannon** (base 2, suavizada) e, acima de `DRIFT_THRESHOLD=0.5` com ≥ `MIN_DRIFT_NOTES=4` notas, propõe revisão. Ambos viram **motivos de demolição** (`DemotionReason::Defeated`/`Drifted`) no `maintenance prune` — off-path, só propõem. Puro, determinístico, sem dep nova. `DIVERGENCES.md` #111.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #111.
- Fecha E19-T10.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
