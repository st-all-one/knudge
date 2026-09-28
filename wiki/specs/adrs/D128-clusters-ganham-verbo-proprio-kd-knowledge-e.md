# D128 — Clusters ganham verbo próprio kd knowledge e o eixo container passa a

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Clusters ganham verbo próprio `kd knowledge` e o eixo `container` passa a usar a hierarquia.** `container_of` sobe pelos pais (`results_in`, D52/D93) — a mesma relação de `Graph::parent`/`belongs_to` — com fallback para `depends_on`; o eixo Container deixa de ser vazio em projetos reais (era `0/53`, `0/125`). `kd knowledge map [--axis A] [--scope C] [--semantic] [--members]` expõe a fase 1 e, com `--semantic`, a fase 2 dentro de cada cluster acima de `clusters.min_volume` — config que antes era ignorada. Read-only (D47). Sobe de `kd maintenance` para um verbo próprio porque o mapa de conhecimento é consulta primária, não manutenção.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
