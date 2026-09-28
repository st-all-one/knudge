# D205 — Métricas de fluxo e caminho crítico

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Métricas de fluxo e caminho crítico** (fecha E19-T11). `task/flow.rs` (puro) deriva do log de eventos `cycle time` (`task/review − primeiro evento`), `lead time` (`close ou agora − create`) e `throughput` (fechamentos por janela), e calcula o **caminho crítico** (PERT/CPM) do DAG `depends_on` como o maior caminho ponderado (peso = `lead time`; empate pelo caminho mais longo, depois menor id). Exposto em `kd task flow` (texto `resumo|`/`throughput|`/`critico|` + `--json`) e em `rewind --json` (`data.flow`, aditivo). Sem chave nova, sem bump de `schema_version` e sem gravar verdade — tudo derivado do log. `DIVERGENCES.md` #108.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #108.
- Fecha E19-T11.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
