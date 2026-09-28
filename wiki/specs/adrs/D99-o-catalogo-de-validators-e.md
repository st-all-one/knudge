# D99 — O catálogo de validators é **`

- **Status:** Aceita
- **Categoria:** W. Catálogo de validators (D99)

## Contexto

Bloco **W. Catálogo de validators (D99)**.

## Decisão

O catálogo de validators é **`.knudge/validators.toml`**, no **subset TOML próprio** (D97) — **não** YAML, para não introduzir dependência nem parser novo. Cada validator é uma tabela com `cmd` (obrigatório), `scope` (globs), `severity` (`error\|warn\|info`, default `error`) e `timeout` (ms, default 120000). A chave de topo `globals` lista os validators globais (fonte que o `AGENTS.md` renderiza). A resolução é `checks(task) = explícitos ∪ globais ∪ por_âncora(anchors(task))`; validator explícito ausente vira `missing[]` (não é erro fatal). A **execução** fica na borda (`HookRunner`, E12); o núcleo só resolve e descreve.

## Impacto

- Catálogo de validators em **TOML** (`validators.toml`), com `globals` no topo e resolução de `checks` em três fontes.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
