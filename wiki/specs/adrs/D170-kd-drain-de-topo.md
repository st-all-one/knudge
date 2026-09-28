# D170 — kd drain de topo

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D145.

## Decisão

**`kd drain` de topo.** Absorve `kd knowledge digest` (D145): sem flag = `--help` (não executa); `--status` = estado rico + recomendação; `--digest [--force]` = digestão (`--force` apaga `.idx/` e redigeri tudo, último recurso); `--force` sem `--digest` = `invalid_input` (2). Revisa D145.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
