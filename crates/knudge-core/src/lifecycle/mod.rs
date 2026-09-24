//! Escopo `lifecycle`: shelf-life, decay, purga, confiança e clusters (E09/E10).
//!
//! Retenção previsível e consolidação barata: nada de conhecimento válido é demolido por ciclo
//! (D45) e nenhum conteúdo é removido antes da janela de retenção (E10-T03).

pub mod clusters;
pub mod confidence;
pub mod decay;
pub mod plan;
pub mod retire;
pub mod semantic;
pub mod shelf_life;
pub mod supersession;

#[cfg(test)]
mod tests;

pub use clusters::{
    Cluster, ClusterAxis, scope_of, structural_clusters, structural_clusters_filtered,
};
pub use confidence::{
    ConfidenceInput, DEFAULT_TASK_CONFIRMATION, age_factor, confidence_score, drift_factor,
    from_tasks, from_tasks_with, is_success_task,
};
pub use decay::{
    AnchorValidity, DecayPolicy, compute_anchor_validity, compute_anchor_validity_with, has_glob,
    should_demote, walk_paths,
};
pub use plan::{DemotionCandidate, DemotionInput, DemotionReason, demotion_candidates};
pub use retire::{
    DEFAULT_RETIRED_DAYS, Retention, Retirement, due_for_purge, purge_due, retirements,
};
pub use semantic::{
    MIN_SEMANTIC_VOLUME, SemanticCluster, cluster_by_similarity, semantic_clusters,
    semantic_phase2, should_run,
};
pub use shelf_life::{
    DAY_MS, EXPIRING_GRACE_DAYS, Freshness, ShelfLife, age_days, expired_ids, expiry_for,
    freshness, is_expired,
};
pub use supersession::{cycle_members, demote, filter_protected, protected};
