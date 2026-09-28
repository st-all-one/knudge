# D203 — Drift de âncoras persistido e aplicado à confiança

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Drift de âncoras persistido e aplicado à confiança** (absorve E16/T05/D174). A validade de âncoras (já computada off-path, varredura única — E16/T10) vira o derivado `.idx/drift.jsonl` (`lifecycle/drift.rs`, `DriftStore`/`DriftIndex`, purgável por D84), com `drift = 1 - AnchorValidity::fraction`. `prune` persiste o arquivo; `ask`/`knowledge rank` o carregam (`RecallQuery.drift`/`RankQuery.drift`) e alimentam `ConfidenceInput.drift`. `confidence_score` passa a descontar o score **inteiro** por `drift_factor` (antes só a base, o que anulava o efeito no `rank`, onde `similarity = 0`): uma nota com âncoras quebradas perde confiança mesmo sem query. Arquivo ausente ⇒ `drift = 0` (degradação graciosa, R33). Sem chave nova e sem bump de `schema_version`. `DIVERGENCES.md` #106. Fecha E19-T01b.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #106.
- Fecha E19-T01b.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
