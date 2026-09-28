# D127 — Rollup de progresso por épico é derivado

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Rollup de progresso por épico é derivado** (`task::epic_of`/`progress_of`): conta os **itens de trabalho folha** (`Graph::is_work_item` sem filhos de trabalho) no subárvore do épico e quantos estão `closed`; sem verdade nova. Folhas = a fronteira acionável (`task` + `issue` não decomposta): fechar um `issue` com tarefas abertas não infla, esquecer de fechá-lo não trava. Exposto no `kd task close` (linha `epico: <id>\|<título> (<done>/<total>)`), no `kd task show` (`epico:`/`progresso:`) e nos containers do `task graph` (`(done/total)`). Fecha a lacuna "nenhum faz rollup automático" da bancada de hierarquia.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
