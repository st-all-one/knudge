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

/// Agrupa `ids` por similaridade com **complete-link** (greedy).
///
/// Um id só entra num cluster se for similar (`>= threshold`) a **todos** os membros — evita o
/// encadeamento em que um item central puxa vizinhos dissimilares entre si. Determinístico: a
/// ordem de `ids` (canônica) define o resultado; o primeiro cluster compatível recebe o id.
pub fn cluster_by_similarity<F>(ids: &[String], threshold: f64, similarity: F) -> Vec<Vec<String>>
where
    F: Fn(&str, &str) -> f64,
{
    let mut clusters: Vec<Vec<String>> = Vec::new();
    for id in ids {
        let mut placed = false;
        for cluster in &mut clusters {
            let compatible = cluster
                .iter()
                .all(|member| similarity(member, id) >= threshold);
            if compatible {
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

/// Fase 2 aplicada a um cluster estrutural, preservando a origem (D128).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticCluster {
    /// Cluster estrutural de origem.
    pub parent: Cluster,
    /// Grupos de ids similares (complete-link).
    pub groups: Vec<Vec<String>>,
}

/// Roda a fase 2 e preserva o cluster estrutural de origem (D128).
///
/// Só inclui clusters acima de `min_volume`; a similaridade é injetada (determinístico).
pub fn semantic_clusters<F>(
    structural: &[Cluster],
    min_volume: usize,
    threshold: f64,
    similarity: F,
) -> Vec<SemanticCluster>
where
    F: Fn(&str, &str) -> f64,
{
    let mut out = Vec::new();
    for cluster in structural {
        if !should_run(cluster.members.len(), min_volume) {
            continue;
        }
        out.push(SemanticCluster {
            parent: cluster.clone(),
            groups: cluster_by_similarity(&cluster.members, threshold, &similarity),
        });
    }
    out
}
