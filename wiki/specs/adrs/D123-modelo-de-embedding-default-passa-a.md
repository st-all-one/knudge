# D123 — Modelo de embedding default passa a

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Modelo de embedding default passa a `ibm-granite/granite-embedding-97m-multilingual-r2`** (384d, Apache-2.0, 200+ idiomas com **PT** explícito), substituindo `msmarco-MiniLM-L12-cos-v5` (inglês). Medido na bancada PT-BR (`bench/`): +0.070 nDCG@5 sobre BM25 e melhor R@1/MRR que o default antigo, **mesmo tamanho de índice**. Execução/provedor inalterados (D101); o modelo antigo segue servível por config.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
