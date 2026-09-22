//! Clusters fase 2: semânticos, opcionais e off-path (D42/D47, E10-T07).
//!
//! Só roda **dentro** de um cluster estrutural, acima de um volume mínimo e fora do caminho
//! crítico do `recall`. A similaridade é **injetada** (E11 fornece os embeddings), então esta
//! camada permanece pura e testável com um oráculo falso.

use super::clusters::Cluster;

/// Volume mínimo para acionar o clustering semântico (default conservador).
pub const MIN_SEMANTIC_VOLUME: usize = 10;

/// `true` se o volume justifica rodar a fase 2.
#[must_use]
pub fn should_run(member_count: usize, min_volume: usize) -> bool {
    member_count >= min_volume
}

/// Agrupa `ids` por similaridade **greedy single-link** (representante = primeiro membro).
///
/// Determinístico: a ordem de `ids` e o resultado são estáveis; empates vão para o primeiro
/// cluster compatível.
pub fn cluster_by_similarity<F>(ids: &[String], threshold: f64, similarity: F) -> Vec<Vec<String>>
where
    F: Fn(&str, &str) -> f64,
{
    let mut clusters: Vec<Vec<String>> = Vec::new();
    for id in ids {
        let mut placed = false;
        for cluster in &mut clusters {
            if let Some(representative) = cluster.first()
                && similarity(representative, id) >= threshold
            {
                cluster.push(id.clone());
                placed = true;
                break;
            }
        }
        if !placed {
            clusters.push(vec![id.clone()]);
        }
    }
    clusters
}

/// Roda a fase 2 apenas nos clusters estruturais acima do volume mínimo.
pub fn semantic_phase2<F>(
    structural: &[Cluster],
    min_volume: usize,
    threshold: f64,
    similarity: F,
) -> Vec<Vec<String>>
where
    F: Fn(&str, &str) -> f64,
{
    let mut out = Vec::new();
    for cluster in structural {
        if should_run(cluster.members.len(), min_volume) {
            out.extend(cluster_by_similarity(
                &cluster.members,
                threshold,
                &similarity,
            ));
        }
    }
    out
}
