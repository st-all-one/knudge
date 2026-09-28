# D163 — kd doctor de topo

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D119.

## Decisão

**`kd doctor` de topo.** `kd maintenance doctor [--audit]` vira `kd doctor [--fix] [--explain]`: a execução padrão roda os **13 checks + auditoria** num só relatório; `--fix` corrige o reversível; `--explain` detalha cada achado (`esperado`/`encontrado`/`ação`); `--audit` **deixa de existir**. `healthy` só com **zero achados**; advisórios viram `degraded` (nunca "saudável" silencioso). Revisa D119.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
