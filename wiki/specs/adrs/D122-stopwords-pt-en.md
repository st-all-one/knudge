# D122 — stopwords PT+EN

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

O canal lexical descarta **stopwords PT+EN** e **fragmentos de 1 caractere** (`retrieval::token::content_terms`); corrige votos espúrios (ex.: `de`) que afogavam o canal vetorial em consultas por sinônimo.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
