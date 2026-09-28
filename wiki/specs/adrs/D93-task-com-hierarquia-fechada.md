# D93 — task com hierarquia fechada

- **Status:** Aceita
- **Categoria:** R. Superfície CLI v2 (D93–D94)

## Contexto

Bloco **R. Superfície CLI v2 (D93–D94)**.

## Decisão

**`task` com hierarquia fechada**: novo campo `scope ∈ {plan, epic, issue, task}` (enum fechado; **não** altera o enum de `type`). `plan`/`epic` são `type=container` (view derivada, D52) + `scope`; `issue`/`task` são `type=task` + `scope`. Aninhamento `plan ⊃ epic ⊃ issue ⊃ task` (profundidade máx. **4**), pai único via membership/backref (D52). Tudo de tarefa vive em `kd task`; `kd write` **rejeita** `--type task\|container`.

## Impacto

- `task` ganha `scope` fechado (`plan\|epic\|issue\|task`) e hierarquia máx. 4.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
