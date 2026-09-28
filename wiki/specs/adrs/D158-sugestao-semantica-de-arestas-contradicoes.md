# D158 — Sugestão semântica de arestas/contradições

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Sugestão semântica de arestas/contradições.** `kd knowledge suggest` classifica pares do índice vetorial em `duplicate`/`contradiction`/`link` (banda `suggestions.contradiction_low..high`; `dedup.merge_below` para duplicata; `EdgeState`/`AnchorShare` em vez de `bool`). **Advisory**: persiste em `.idx/suggestions.jsonl` (D50), nunca vira aresta (D49) e é purgado (D84). Determinístico (`score desc, from asc, to asc`); zero-LLM. Config `suggestions.enabled`.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
