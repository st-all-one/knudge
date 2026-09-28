# D126 — Arestas têm via única

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Arestas têm via única:** `kd write --link <FROM:ARESTA:TO>`. `kd task new` deixa de aceitar `--depends-on` e `TaskSpec.depends_on` sai do core; `plan submit` cria as dependências dos passos via `write::link`. Reduz a superfície de API e reaproveita o caminho de grafo (valida id/auto-aresta, grava evento e revisão).

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
