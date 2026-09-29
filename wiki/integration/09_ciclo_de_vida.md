# 09 — Ciclo de vida: confiança, shelf-life, decay e clusters

Nada disso é armazenado: **confiança, retenção, drift e clusters são derivados** de notas/eventos
(D44/D87/D135/D189/D203). Especificação: [`../specs/ciclo-de-vida.md`](../specs/ciclo-de-vida.md).

## Confiança derivada (Beta-Bernoulli — D87/D189)

```rust
use knudge_core::lifecycle::{ConfidenceInput, confidence_score, posterior_mean, lower_bound};

let score = confidence_score(&ConfidenceInput {
    similarity: 0.8,        // similaridade da query (0..=1); 0 em ranking sem query
    successes: 3.0,         // success + partial*0.5
    failures: 1.0,          // failure + abandoned + partial*0.5
    drift: 0.0,             // drift de âncoras (doc abaixo)
    age_days: 12.0,
    feedback: 0.0,
    task_confirmation: 0.1, // já ponderada por recall.confirmation_from_tasks
});
```

- A **média** posterior alimenta o canal `stars`; o **limite inferior** de 95 % é a confiança
  conservadora.
- `confidence_score` combina similaridade + evidência + drift + idade + feedback + tarefas.
- Nunca armazene confiança — recalcule.

## Shelf-life e retenção (D44/D135/D190)

```rust
use knudge_core::lifecycle::{ShelfLife, retention_for, is_expired, expired_ids, freshness};

let policy = ShelfLife::from_config(kd.config());
let agora = kd.now_ms();

let nota = kd.store().read(id)?;
let retencao = retention_for(&nota, agora, &policy, None)?; // R(t) em [0,1]
let expirou = is_expired(&nota, agora, &policy)?;
let vencidas = expired_ids(&notes, agora, &policy)?;
let fresh = freshness(&notes, agora, &policy, pending)?;    // Freshness { stale, expiring, pending }
```

- `foundational` nunca expira; `tactical`/`observational` têm TTL por config.
- Cada `outcome` de sucesso **estende** o prazo (FSRS-like — D190).
- Retenção é `limiar^(t/ttl)`; o cruzamento `R=limiar` é o prazo.

## Decay de âncoras e drift (D43/D203)

```rust
use knudge_core::lifecycle::{
    AnchorValidity, DecayPolicy, DriftStore, entries_from_validity, walk_paths_ignoring,
};
use std::collections::BTreeMap;

// caminhe o projeto uma vez, ignorando o layout de conhecimento
let ignored = kd.project().ignored_dirs();
let paths = walk_paths_ignoring(kd.fs_dyn(), kd.project_root(), &ignored);

let policy = DecayPolicy::from_config(kd.config()); // threshold=0.5, grace=30d
// compute_anchor_validity_cached(...) por nota reusa `paths`

// persista o drift derivado (`.idx/drift.jsonl`)
let validity: BTreeMap<String, AnchorValidity> = /* id -> validade */;
let store = DriftStore::new(kd.fs_dyn(), kd.knowledge_dir());
store.persist(&entries_from_validity(&validity))?;
let drift = store.index()?;                                  // alimenta recall/rank
```

Drift = `1 - fração de âncoras válidas`. Ancorar código e manter `--anchor` atualizado reduz
demolição por `anchor_decay`.

## Uso (renovação por citação — D154)

```rust
use knudge_core::lifecycle::UsageStore;

let usage = UsageStore::new(kd.fs_dyn(), kd.knowledge_dir());
let index = usage.index()?;                        // UsageIndex (last_seen por id)
// após um `recall`, credite os ids citados (coalescido: uma escrita por invocação):
let registros = usage.record(&ids_citados, kd.now_ms())?;
```

Com `retention.renew_on_use = true`, citar uma nota estende a vida dela.

## Clusters e comunidades (D128/D193)

```rust
use knudge_core::lifecycle::{
    MIN_SEMANTIC_VOLUME, structural_clusters, structural_clusters_filtered, should_run,
};

let estruturais = structural_clusters(&index, &graph);          // por anchor/type/class/scope
let filtrados = structural_clusters_filtered(&index, &graph, &filter);
if should_run(member_count, MIN_SEMANTIC_VOLUME) {
    // fase 2: semântica DENTRO de um cluster estrutural (similaridade injetada)
}
```

Clusters fase 1 são determinísticos e sem embeddings; fase 2 roda **off-path**, dentro de um
cluster, acima do volume mínimo.

## Demolição (soft, por política — D45/D177/D208)

```rust
use knudge_core::lifecycle::{demotion_candidates, DemotionInput, DemotionReason};

let plano = demotion_candidates(&DemotionInput {
    now_ms: kd.now_ms(),
    shelf_life: &policy,
    decay: &decay_policy,
    validity: &validity,
    usage: Some(&usage_index),
});
for candidato in &plano {
    match candidato.reason {
        DemotionReason::Expired | DemotionReason::AnchorDecay => { /* soft forget */ }
        _ => {}
    }
}
```

Motivos: `expired`, `anchor_decay`, `contradicted`, `defeated` (TMS), `drifted` (KL/JS). Membros de
ciclo são **protegidos** (`graph.cycle_members()` — D45).

## Recomendações

- **Off-path.** Decay/drift/demolição rodam no rebuild/manutenção, nunca no `recall` quente.
- **Derive, não guarde.** Confiança, retenção, drift e clusters são calculados; `.idx/` é
  descartável.
- **Renove âncoras.** É o sinal que mantém a nota viva e confiável.
- **Drift zero por padrão.** Sem `.idx/drift.jsonl`, `drift = 0` (degradação graciosa — R33).
- **Cuidado com `foundational`.** Nunca expira; use para regras estruturais, não para observações.
- **Proteja ciclos** antes de qualquer demolição.
