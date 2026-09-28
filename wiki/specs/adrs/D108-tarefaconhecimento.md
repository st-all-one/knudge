# D108 — tarefa→conhecimento

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

Feedback derivado **tarefa→conhecimento**: tarefas com `outcomes` de sucesso que compartilham `anchors` confirmam a nota (`task_confirmation`), alimentando o boost do BM25 e a confiança derivada — sem `write`. Peso em `recall.confirmation_from_tasks` (float, default 0.1); o manifest promove a `star`.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
