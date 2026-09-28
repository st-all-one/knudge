# D145 — Fim do maintenance eval; index vira kd knowledge digest

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D90.

## Decisão

**Fim do `maintenance eval`; `index` vira `kd knowledge digest`.** O `eval` sai por completo (verbo, `extra::eval` e o módulo puro `embeddings/eval.rs` + re-exports/testes — nada em produção o usava; era stub e a doc overprometia Recall@k/nDCG@k/MRR). O `maintenance index` passa a **`kd knowledge digest`** (`--status`/`--drain`), no escopo de conhecimento: digere o conteúdo num vetor (384d). A avaliação de modelo segue na bancada externa `bench/`. Revisa D90.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
