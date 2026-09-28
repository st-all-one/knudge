# D152 — Busca sem resultado devolve [no_results]

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Busca sem resultado devolve `[no_results]`.** `kd ask` sem hits → stdout `[no_results]` (literal fixo; `--json` com `hits: []`), em vez de vazio — o agente distingue "busca vazia" de erro. **A demanda de observabilidade de busca é removida** (sem `.idx/recall_stats.jsonl`, sem `--explain-miss`).

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
