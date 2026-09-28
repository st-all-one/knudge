# D139 — graph enxuto e plan

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D115/D116/D119.

## Decisão

**`graph` enxuto e `plan.md` ancorando vários épicos (floresta).** O texto do `kd task graph` passa a `id\|kind\|status\|statement (done/total)` com indentação — `role`/`mode` saem do texto (seguem no `--json`, que perde `owner` por D136). `plan.md` pode ancorar **vários** épicos-raiz: `roots_for_path` devolve todos (ordem de `id`) e `kd task graph --program` renderiza a floresta; `--root` segue raiz única. Revisa D115/D116/D119.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
