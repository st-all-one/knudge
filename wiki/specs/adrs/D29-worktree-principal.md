# D29 — worktree principal

- **Status:** Aceita
- **Categoria:** F. Git, diretórios e persistência

## Contexto

Bloco **F. Git, diretórios e persistência**.

## Decisão

`.knudge/` resolve no **worktree principal** (`git rev-parse --git-common-dir`); submódulo não conta.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
