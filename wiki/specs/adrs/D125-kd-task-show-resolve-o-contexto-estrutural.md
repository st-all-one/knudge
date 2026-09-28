# D125 — kd task show resolve o contexto estrutural

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**`kd task show` resolve o contexto estrutural** do item — `parent`, `blocked_by`, `blocks` e `children` com **título** e estado (`task::context_of`) — em texto e `--json`. Um comando responde "onde isto se encaixa e o que o bloqueia" sem puxar a árvore inteira; era a lacuna apontada pela bancada de hierarquia.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
