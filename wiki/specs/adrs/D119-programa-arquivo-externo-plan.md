# D119 — Programa = arquivo externo plan/

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Programa = arquivo externo `plan/*.md` ancorado ao Épico-raiz** (`scope=epic`, sem pai); `scope=plan` deprecado; `kd task graph --program`; check `program-anchor` no `doctor`; config `programs.glob`; `--source` em `task new`.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
