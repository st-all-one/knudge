# D141 — Criação de tarefas em lote

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Criação de tarefas em lote (JSONL) e por objeto.** `kd task new --batch FILE\|-` cria tarefas/epics a partir de JSONL com as chaves canônicas (`statement`/`body`/`scope`/`kind`/`parent`/`checks`/`anchors`/`tags`/`classification`/`status`/`blocks`) + `key` local para `parent`/`depends_on`; `--params '{...}'` é o atalho de um item. Processa em ordem (pai antes do filho), **best-effort** com `warnings[]` (R33) e `--dry-run`; teto `task.batch_max`. Espelha `write --batch` (D110). Linhas podem ser criação (sem `id`) ou atualização (`id`) e carregam as 7 arestas explícitas (ids ou `key`s) — permite **re-parentar** itens existentes e **criar vínculos** no mesmo lote, reusando `write::link` (D126). A saída é a **lista do que foi criado/atualizado** (`key\|id\|scope\|status\|statement`) e o `--json` traz `key`→id e as arestas resolvidas — a IA liga os itens sem nova consulta.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
