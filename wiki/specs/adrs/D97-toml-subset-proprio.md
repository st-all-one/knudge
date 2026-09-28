# D97 — TOML subset próprio

- **Status:** Aceita
- **Categoria:** U. Config e worktree (D97)

## Contexto

Bloco **U. Config e worktree (D97)**.

## Decisão

**TOML subset próprio** (sem dependência externa): aceita comentários, `[seção]`/`[seção.sub]`, chaves bare/citadas pontilhadas, strings básicas/literais de **uma linha**, inteiros com `_`, floats, booleanos e listas (inclusive multilinha). Rejeita `[[...]]`, strings multilinha e `null` com erro `config`. A leitura **preserva a ordem** (diff mínimo) e a escrita é canônica (D63). `sync` executa `git -C <raiz>` (guard de worktree, D32); `onboard` degrada para defaults quando não há config global.

## Impacto

- Fixa o subset TOML (ordem preservada/canônica), o guard `git -C` do `sync` e a degradação do `onboard` sem global.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
