# D98 — Arestas explícitas são chaves de frontmatter de primeiro nível

- **Status:** Aceita
- **Categoria:** V. Arestas explícitas (D98)

## Contexto

Bloco **V. Arestas explícitas (D98)**.

## Decisão

**Arestas explícitas são chaves de frontmatter de primeiro nível**, nomeadas pelo `EdgeKind` (`references`, `depends_on`, `contradicts`, `supports`, `extends`, `replaces`, `rejects`, `results_in`), cada uma uma **lista de ids** (omitida quando vazia — D05). Ordem canônica: as 8 arestas ficam logo após `superseded_by` e antes de `revision` — **27 chaves**. `superseded_by` continua sendo o **ponteiro reverso** (id único) de `replaces`; a bidirecionalidade é validada pela integridade (D46). A extração textual é **sugestão derivada** em `.idx/suggestions.jsonl` (nunca aresta; D49/D50) e o `expand` percorre só o explícito. Ciclos de supersessão são detectados por SCC sobre `replaces` (D45) e os membros **não demovem**. (**D100** acrescenta `not_before` logo após `expires_at`, totalizando **28 chaves**.)

## Impacto

- Fixa as 8 chaves de aresta, a ordem canônica de 27 chaves, o ponteiro reverso `superseded_by` e o storage derivado de sugestões.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
