# D207 — Claims SPO + ontologia leve + proveniência

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Claims SPO + ontologia leve + proveniência** (fecha E19-T09). Três adições **aditivas** ao frontmatter (bump de `schema_version` 1→2; notas v1 seguem válidas e byte-idênticas no rebuild): (1) **ontologia leve SKOS-lite** — arestas `same_as`/`broader`/`narrower`/`related` (`EdgeKind` 8→12), com `broader`↔`narrower` inversos e `same_as`/`related` simétricos; (2) **`claims`** — lista de mapas `{subject, relation, object}` (SPO) validada (strings trimadas, ≤120 escalares, relação de mundo aberto); (3) **`provenance`** — mapa PROV-lite `{entity, activity, agent}`. Inferência **derivada** em `graph/ontology.rs` (`equivalence_classes`, `broader_ancestors`, `narrower_descendants`, `has_hierarchy_cycle`) e contradição **precisa** por claims (`claim_conflicts`: mesma `(sujeito, relação)` com objetos divergentes), reportada no check `integrity` do `doctor`. Escrita por `kd write --claim S:R:O` + `--agent`/`--activity`; merge/update unem claims sem duplicar. Sem LLM, sem dep nova, sem gravar derivados. `DIVERGENCES.md` #110.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #110.
- Fecha E19-T09.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
