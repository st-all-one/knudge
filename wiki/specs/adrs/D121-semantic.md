# D121 — semantic

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

`why` do `ask` ganha **`semantic`** (canal vetorial), com precedência `file_match > anchor_match > tracker_match > stars > semantic > recent > universal`; o default de `recall.default_limit` cai de 10 para **5**.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
