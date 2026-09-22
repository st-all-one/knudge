# E10 — Ciclo de vida, decay e clusters

> **Fase 2.** O que mantém o corpus vivo e enxuto: shelf-life por classificação, decay de
> âncoras, purge de inativos com histórico, supersessão sem demolição de válidos, e clusters
> em **duas fases** (estrutural determinístico primeiro; semântico só depois e dentro dele).
>
> **Decisões:** D28, D43, D44, D45, D47, D48, D52, D56.
> **Políticas:** R33 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

> Nota: o container como **view derivada** (D28/D52) é implementado em E08-T07; aqui ele
> entra apenas como escopo de clustering/retenção.

## Objetivo do épico

Retenção previsível e consolidação barata, sem apagar verdade e sem deixar o `recall` mostrar
redundância que o `compact` deveria resolver.

## Pré-requisitos

E09.

## Tarefas

### E10-T01 ☐ Shelf-life por classificação
- **Objetivo:** TTL/expiração por `classification`: `foundational` nunca expira; `tactical` e
  `observational` com prazos configuráveis (defaults conservadores).
- **Entregáveis:** política de shelf-life; config.
- **Decisões:** D44.
- **Aceite:** `foundational` sobrevive; `observational` expira no prazo; prazos por config.

### E10-T02 ☐ Decay de âncoras
- **Objetivo:** no rebuild, validar `anchors` (existe? glob ainda casa?); demover após
  **grace period** se a fração válida < threshold.
- **Entregáveis:** `computeAnchorValidity`; grace/threshold.
- **Decisões:** D43.
- **Aceite:** nota com âncoras majoritariamente quebradas é demovida após grace; grace
  respeitado (não demove antes).

### E10-T03 ☐ Purge de inativos com histórico
- **Objetivo:** registrar `retired_at` em supersessão/demissão e **purgar conteúdo só após a
  janela de retenção**; nunca hard-delete imediato.
- **Entregáveis:** `retired_at` derivado; purge agendado.
- **Decisões:** D48 (auditoria).
- **Aceite:** conteúdo retired permanece até o prazo; purge purga também o derivado (E03-T07).

### E10-T04 ☐ Supersessão com ciclos
- **Objetivo:** usar a detecção de ciclos (E05-T04) no rebuild; **membros de ciclo não
  demovem**.
- **Entregáveis:** integração ciclo × decay.
- **Decisões:** D45.
- **Aceite:** ciclo não demove nenhum membro; fora do ciclo, supersede normal demove.

### E10-T05 ☐ `not_before` separado de `expires_at`
- **Objetivo:** quando surgir necessidade de agendamento, introduzir `not_before` sem
  confundir com expiração.
- **Entregáveis:** campo opcional + semântica de `ready`.
- **Decisões:** D56.
- **Aceite:** agendamento não afeta expiração; views `ready`/`blocked` consideram `not_before`.

### E10-T06 ☐ Clusters fase 1 (estrutural)
- **Objetivo:** agregação determinística por `anchor`, `type`, `classification` e container —
  auditoria barata, sem estatística.
- **Entregáveis:** clusterizador fase 1.
- **Decisões:** D47 (contexto).
- **Aceite:** clusters reproduzíveis; sem dependência de embeddings.

### E10-T07 ☐ Clusters fase 2 (semântico, opcional)
- **Objetivo:** clustering semântico **somente dentro** de um cluster estrutural, em batch e
  **off-path**; alimenta `compact`.
- **Entregáveis:** clusterizador fase 2 (requer E11).
- **Decisões:** D42/D47 (contexto).
- **Aceite:** só roda com volume que justifique; não entra no caminho crítico do `recall`.

## Definition of Done

- [ ] Retenção/decay configuráveis e testados nos limites.
- [ ] Nada de conhecimento válido é demolido por ciclo.
- [ ] Clusters fase 1 determinísticos; fase 2 opcional e off-path.

## Não-objetivos

- Embeddings (E11) — a fase 2 depende dele.
- `compact` como proposta (E08-T08).
