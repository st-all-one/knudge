# D136 — Um agente principal, sempre

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D114.

## Decisão

**Um agente principal, sempre: fim do eixo de posse.** `kd task claim`/`--by`/`--release`, `task::ownership`, `--owner`/`--mine` e o `owner` do `task graph` saem; `KNUDGE_AGENT` deixa de ser usado; "estou fazendo isto" passa a ser só `status=in_progress`. `--since` sai de `kd task list` (coerência com D135 — tarefa não tem tempo); `--since`/`--until` seguem em `ask`/`rewind`. Revisa D114 e D116 (reduz `mode` a `{sequential, concurrent, magentic}`, removendo `supervisor`/`handoff`); remove o `actor` do schema de evento (sem escritor).

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
