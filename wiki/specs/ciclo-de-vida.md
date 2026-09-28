# Ciclo de vida e confiança

Retenção previsível e consolidação barata: **nada de conhecimento válido é demolido por ciclo**
(D45) e **nada é removido antes da janela de retenção**. A confiança é sempre **derivada** (D87),
nunca armazenada.

- Código: `crates/knudge-core/src/lifecycle/`
- Decisões: D42–D48, D87, D108, D154, D175, D177, D189–D193, D203, D208
- Fórmulas: [`matematica.md`](matematica.md) §3–§6.

## Shelf-life e retenção (D44/D190)

- O shelf-life é derivado da **`classification`** (D135): `foundational` **nunca expira**;
  `tactical`/`observational` têm prazo.
- **Curva de esquecimento FSRS-like** (D190): cada `outcome` de **sucesso** estende o prazo em
  `retention.growth_percent` (default **50 %**) e **reseta o relógio**
  (`origin = max(created, último ensaio, último uso se renew_on_use)`).
- `retention(t, ttl, limiar) = limiar^(t/ttl)` (`retention.rs`, pura); o cruzamento `R = limiar`
  (default 0,5) é o prazo. O `prune --json` informa a `retention` atual.

## Renovação por uso (D154)

O uso (citação) vira derivado `.idx/usage.jsonl` (`UsageStore`), nunca verdade nem evento. Com
`retention.renew_on_use=true`, a expiração é `max(created_at, last_seen) + prazo` e **só estende**.
`ask`/`rewind` creditam os ids devolvidos, coalescidos numa escrita por invocação.

## Confiança derivada (D87/D189)

`confidence.rs`/`beta.rs`:

- Os `outcomes` viram **ensaios de Bernoulli**: `success`=1, `partial`=0,5,
  `failure`/`abandoned`=1 falha.
- Posterior `Beta(1+s, 1+f)` (revisa D87): a **média** alimenta o canal `stars`; o **limite
  inferior de 95 %** (Wilson, `Z_95 = 1.96`) é a confiança conservadora (1 sucesso ≈ 0,21 × 20 ≈
  0,84).
- `Meta` ganha `failures`; `INDEX_FORMAT → retrieval-v3`.
- `confidence_score` soma base + `lower_bound` (+ feedback explícito 0,2), idade aditiva
  (`AGE_WEIGHT = 0.05`, D175) e desconto por **drift** (`drift_factor`, D203).
- Confirmação **tarefa→conhecimento** (D108): tarefas com outcomes de sucesso que compartilham
  âncoras confirmam a nota (`recall.confirmation_from_tasks`, default 0,1).

## Decay e drift de âncoras (D43/D86/D203)

- **Decay** (`decay.rs`): no rebuild, computa a fração de âncoras válidas
  (`compute_anchor_validity_cached`, varredura única — E16/T10; `MAX_WALK_DEPTH = 32`). Demove após
  *grace* se a fração < threshold.
- **Drift** (`drift.rs`): a validade vira o derivado `.idx/drift.jsonl` (`DriftStore`/`DriftIndex`,
  purgável); `drift = 1 − fraction`. `prune` persiste; `ask`/`rank` carregam e alimentam
  `confidence_score`. Arquivo ausente ⇒ `drift = 0` (R33).

## Drift de termos (D208)

`term_drift.rs` compara a distribuição de termos da metade antiga × nova (por `created_at`) de cada
tópico (âncora) por **Jensen-Shannon** (base 2, suavizada). Acima de `DRIFT_THRESHOLD = 0.5` com
≥ `MIN_DRIFT_NOTES = 4` notas, propõe revisão.

## Clusters e comunidades (D47/D128/D129/D193)

- **Fase 1 — estruturais** (`clusters.rs`): agrupamento determinístico por eixo (container, tag,
  âncora, tipo…); `scope_of` sobe pelos pais (`results_in`) com fallback `depends_on`.
- **Fase 2 — semânticos** (`semantic.rs`): opcional, off-path, **complete-link** (D129) — um id só
  entra se for similar (≥ `clusters.similarity_threshold`) a **todos** os membros.
- **Comunidades** (`communities.rs`, D193): GraphRAG sobre arestas + âncoras compartilhadas
  (ver [`grafo.md`](grafo.md)).

## Demolição (D45/D112/D177/D208)

`plan.rs::demotion_candidates` decide o que **propor** (nunca grava — D47/D112), numa cadeia
if/else-if:

| Motivo (`DemotionReason`) | Condição |
|---|---|
| `Expired` | shelf-life vencido (D190) |
| `AnchorDecay` | fração de âncoras válidas < threshold (D43) |
| `Contradicted` | lado perdedor de `contradicts` (D177) |
| `Defeated` | dependente de premissa retratada / alvo de `replaces` (D208) |
| `Drifted` | vocabulário do tópico mudou (D208) |

Membros de ciclo são protegidos (D45). A **aplicação** é sempre via `kd forget` (D112).

## Supersessão, purga e retenção

- `supersession.rs`: `protected`/`filter_protected`/`cycle_members`/`demote`.
- `retire.rs`: purga de inativos com histórico (D48); nada é apagado sem decisão (D52/D86).
- `purge_derived` (D84) limpa os derivados em toda remoção.

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Shelf-life | `lifecycle/shelf_life.rs` |
| Retenção/FSRS | `lifecycle/retention.rs` |
| Uso | `lifecycle/usage.rs` |
| Confiança/Beta | `lifecycle/{confidence,beta}.rs` |
| Decay/drift | `lifecycle/{decay,drift}.rs` |
| Drift de termos | `lifecycle/term_drift.rs` |
| Planejamento de demolição | `lifecycle/plan.rs` |
| Clusters/comunidades | `lifecycle/{clusters,semantic,communities}.rs` |
| Supersessão/retirada | `lifecycle/{supersession,retire}.rs` |

## Testes

`lifecycle/tests/` (shelf_life, retention, usage, confidence, beta, decay, drift, term_drift,
plan, clusters, semantic, communities, supersession, retire). `DIVERGENCES.md` #98–#103,
#106, #111.
