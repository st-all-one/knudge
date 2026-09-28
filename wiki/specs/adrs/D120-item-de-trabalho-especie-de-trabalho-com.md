# D120 — Item de trabalho = espécie de trabalho com scope

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Item de trabalho = espécie de trabalho com `scope`** (`NoteType::is_work_kind` + `Graph::is_work_item`); as views `ready`/`blocked`, o `impact` e o `next:` passam a enxergar `--kind error\|question\|risk\|decision`. Containers e conhecimento (sem `scope`) ficam de fora.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
