# D30 — Exclusão absoluta via

- **Status:** Aceita (com ponto de atenção)
- **Categoria:** F. Git, diretórios e persistência

## Contexto

Bloco **F. Git, diretórios e persistência**.

## Decisão

**Exclusão absoluta via `.git/info/exclude`** — nunca `.gitignore`. O `.knudge/` inteiro e tudo dentro dele é atrelado ao `info/exclude`. (Interação com D34 — ver §Pendências.)

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
