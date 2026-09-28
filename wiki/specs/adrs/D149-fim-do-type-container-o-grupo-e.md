# D149 — Fim do type=container; o grupo é derivado de scope=epic

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D93/D113/D134.

## Decisão

**Fim do `type=container`; o grupo é derivado de `scope=epic`.** O enum de `type` cai de 11 para 10 (sai `container`); um grupo é uma nota com `scope=epic`, **sem `type`** (tipo efetivo `epic`, prefixo do `id`). O filtro/eixo `container` vira `scope` (D146); `container_of` → `scope_of`; `Graph::is_work_item` exclui épicos. Remove a tripla ambiguidade de "container". Revisa D93/D113/D134.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
