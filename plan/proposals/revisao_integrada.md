# Revisão integrada das propostas E16–E19

> **Status:** revisão (não implementada). Cruza
> [`qualidade_busca_depreciacao.md`](qualidade_busca_depreciacao.md) (E16),
> [`worker_embeddings_install.md`](worker_embeddings_install.md) (E17),
> [`comandos_scriptados.md`](comandos_scriptados.md) (E18) e
> [`modelo_conhecimento_rico.md`](modelo_conhecimento_rico.md) (E19).
>
> **Objetivo:** garantir que as quatro possam ser implementadas **juntas, sem conflito** —
> resolvendo sobreposições de decisão, referências obsoletas e a superfície disputada por três
> épicos. O que depende de escolha do usuário está em **§4 (decisões necessárias)**.

---

## 1. Método

1. Mapear decisões (`D172–D201`), tarefas (`Txx`) e **módulos** tocados por épico.
2. Cruzar por **módulo** e por **superfície**; classificar: *duplicado*, *sobreposto*,
   *subsumido*, *conflito*, *sequência*.
3. Propor resolução; o que não é resolvível por princípio vira **pergunta**.

---

## 2. Mapa por módulo (quem toca o quê)

| Módulo | E16 | E17 | E18 | E19 |
|---|---|---|---|---|
| `retrieval/token.rs` | T03/T04 | — | — | T07 (usa) |
| `retrieval/{bm25,rrf,pipeline,rank}.rs` | T05/T06/T09 | — | — | T04/T06 |
| `lifecycle/confidence.rs` | T05 | — | — | T01 |
| `lifecycle/shelf_life.rs` | T08 | — | — | T02 |
| `lifecycle/{clusters,plan}.rs` | T02 | — | — | T05/T10 |
| `write/dedup` | T03 (dedup) | — | — | T07 |
| `schema/` | — | — | — | T03/T09 |
| `task/` | — | — | — | T11 |
| `embeddings/` | T09 | T01/T02 | — | T08 |
| `commands/maintenance/watch.rs` | — | T01–T07 | T01–T04 | — |
| `commands/self_cmd.rs` | — | — | T05 | T12 |
| `cli/` (superfície) | T02 | T04 | T04/T06 | T12 |

---

## 3. Conflitos e resoluções propostas

| # | Conflito | Tipo | Resolução proposta |
|---|---|---|---|
| **C1** | E16/**D174** (preencher `drift`+`feedback`) × E19/**D189** (fórmula Beta) — ambos editam `confidence.rs` | sobreposto | **E19/D189 absorve E16/D174**: o Beta usa `drift`/`feedback` como entradas; E16/T05 vira o "wire dos sinais" e E19/T01 troca a matemática. Ou remover E16/T05. **→ Q1** |
| **C2** | E16/**D178** (shelf-life por evidência) × E19/**D190** (retenção FSRS) — ambos reimplementam retenção | conflito | **E19/D190 substitui E16/D178**; remover E16/T08. **→ Q2** |
| **C3** | E16/**D177** (`contradicts`) × E19/**D198** (TMS/defeasible) | sobreposto | E16/D177 é o **passo mínimo**; E19/T10 generaliza. Sequenciar (E16→E19) ou E19 absorve. **→ Q12** |
| **C4** | E16/**D179** (recalibrar `rrf_k`/pesos) × E19/T06 (fusão calibrada + reranking) | sobreposto | E16/T09 é a **1ª calibração**; E19/T06 estende (normalização + reranking). Sequenciar. |
| **C5** | E16 (canal de âncoras/RRF) × E19/**D192** (PPR) | sobreposto | E19/T04 **generaliza** o canal; recalibrar **depois** de T04. **→ Q10** |
| **C6** | E17 assume `kd maintenance watch-service` × E18/**D186** (`kd drain service`) | referência obsoleta | E17 reescreve as referências para o novo caminho. **→ Q9** |
| **C7** | E17/**D181** (stream stderr) × E18/**D184**+T01 (stream no wrapper) | duplicado | E18/T01 é a **implementação canônica**; E17/T03 referencia. |
| **C8** | E17/**D180** (sem confirmação) × E18/D184 ("ação explícita = aceite") | redundante (ok) | Manter o princípio em E18; E17 referencia. |
| **C9** | E17/**D183** (checksum) × E18/D184 (`curl`+checksum) | redundante (ok) | Idem. |
| **C10** | E16 (comportamento de `ask`/`knowledge`) × E18 (worker→`drain`) × E19/**D201** (superfície enxuta) | conflito de superfície | **Um só** ponto de verdade: `16_cli_surface.md` atualizado em **E18** (worker) e depois em **E19/T12** (consolidação); **E16 só muda comportamento, não nomes**. **→ Q5** |
| **C11** | E18/**D187** (`self upgrade` real) × E19/**D201** (`self` enxuto) | sobreposto | E18 implementa `upgrade`; E19/T12 **só enxuga** (não remove `upgrade`). Sequenciar. |
| **C12** | E17/T01/T02 (reconciliação/probe **no binário**) × E18/D184 (lógica **no script**) | conflito de arquitetura | A lógica vai para o **script**; o binário só invoca. E17/T01/T02 reescopados. **→ Q7** |
| **C13** | E16/E17/E18 = **0.5.0** × E19 = **0.6.0** (mas R1 é byte-free) | versionamento | Dividir E19: R1/R4 em **0.5.x** (sem tocar bytes); R5+ em **0.6.0**. **→ Q4** |
| **C14** | E19/**D191** (obrigatoriedades) × E16 ("nenhuma chave TOON muda") × D162/D156 | conflito de contrato | R1 usa **seções de corpo** (sem bump); chaves novas só na R5. Hard vs soft. **→ Q3** |
| **C15** | E19/D201 (`forget`→`write --status`) × **D112** (aplicação via `kd forget`) | conflito de decisão | Decidir: manter `forget` ou mover e **revisar D112**. **→ Q6** |
| **C16** | E19/T09 (R5, schema bump) × E19/T03 (R1, se adicionar chaves) | ordem | T03 sem chaves (corpo) ⇒ R1 não bumpa; chaves só em T09. |
| **C17** | E18 (`install.sh` raiz × `scripts/`) × E17 (referencia) | localização | Decidir; E17/E18 alinham. **→ Q8** |
| **C18** | E19/T06 (reranker) × E17/**D182** (reconciliação de endpoint) | cobertura | A reconciliação de D182 deve cobrir **também** o endpoint do reranker. **→ Q11** |
| **C19** | E16/T02 (fix de status em `knowledge`) × E19/D201 (mover `knowledge`) | ordem | E16/T02 **primeiro** (corrige); E19/T12 depois (move). |
| **C20** | E18/T06 (auditoria de verbos) × E19/D201 (superfície enxuta) | sobreposto | E18/T06 **lista candidatos**; E19/T12 **decide o alvo final**. |

---

## 4. Decisões — **resolvidas** (recomendações aceitas)

> As 12 decisões foram respondidas com "seguir as recomendações"; aplicadas em E16–E19.

| # | Decisão | Resposta aplicada |
|---|---|---|
| Q1 | Confiança | **E19/D189 (Beta) absorve E16/D174**; E16/T05 marcada absorvida |
| Q2 | Retenção | **E19/D190 (FSRS) substitui E16/D178**; E16/T08 marcada substituída |
| Q3 | Obrigatoriedades | **soft** + **seções de corpo** (R1 sem bump); hard/chaves depois |
| Q4 | Versão | E19 **dividido**: R1/R4 em **0.5.x**, R5+ em **0.6.0** |
| Q5 | Superfície | confirmada (≤10 verbos; `knowledge`→`ask`/`map`; `self` enxuto; `drain service`) |
| Q6 | `forget` | **permanece verbo** (D112 preservado); E19/T12 ajustado |
| Q7 | Reconciliação/probe | no **script** (wrapper fino, D184); E17/T01/T02 ajustados |
| Q8 | `install.sh` | permanece na **raiz** (target do `curl \| bash`) |
| Q9 | Worker | **`kd drain service <ação>`** (subcomando `service`) |
| Q10 | PPR | **soma** ao canal de âncoras (não substitui) |
| Q11 | Reranker | **mesmo servidor** llama.cpp; D182 cobre o endpoint |
| Q12 | Contradição/TMS | **sequenciados**: E16/T07 (mínimo) → E19/T10 (TMS) |

---

## 5. Ordem de implementação integrada

> A **ordem canônica** (trilhas + gates + orçamento de performance) vive no
> [`../implementation/24_plano_mestre_0.5.0.md`](../implementation/24_plano_mestre_0.5.0.md).
> Resumo: **todo o escopo E16–E19 entra em 0.5.0** (escopo único; supersede o corte Q4 de
> 0.5.x/0.6.0). A ordem preserva as dependências resolvidas em §3 (C1–C20):

```
E16/T01 (bancada) → E16/T02 (status)
E18/T01 → E17/T03 → E17/T01/T02/T04 → E18/T03 → E18/T04 (drain service) → E17/T05–T07 · E18/T05/T06
E16/T03/T04 → E19/T01/T02/T03 (R1) → E19/T04/T05 (R4) → E16/T06/T07/T09 · E19/T06
E19/T07/T08 (R3) → E19/T09 (R5, schema bump) → E19/T10/T11/T12 (R6/R7)
Fecho: E16/T11/T12 · E17/T08 · E18/T07 · E19/T13
```

**Regra de ouro:** cada épico atualiza **a mesma linha** de `16_cli_surface.md`/`17_matriz_aceitacao.md`
no commit que toca a superfície; nenhum épico "reserva" mudanças de nome para depois.

---

## 6. Pontos em aberto

- _(a preencher)_
