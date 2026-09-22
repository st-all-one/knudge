//! Testes dos clusters estruturais (E10-T06).

use crate::Result;
use crate::graph::Graph;
use crate::lifecycle::Cluster;
use crate::lifecycle::clusters::{ClusterAxis, container_of, structural_clusters};
use crate::schema::{Classification, EdgeKind, NoteType, Scope};
use crate::write::Draft;

use super::{NOW, built, classified, note_created};

fn members_of(clusters: &[Cluster], axis: &ClusterAxis) -> Option<Vec<String>> {
    clusters
        .iter()
        .find(|cluster| &cluster.axis == axis)
        .map(|cluster| cluster.members.clone())
}

#[test]
fn groups_by_type_and_classification() -> Result<()> {
    let tactical = note_created(NoteType::Fact, "tática", "", NOW)?;
    let foundational = classified("fundamento", Classification::Foundational, NOW)?;
    let (index, graph) = built(&[tactical.clone(), foundational.clone()])?;
    let clusters = structural_clusters(&index, &graph);

    let by_type = members_of(&clusters, &ClusterAxis::NoteType(NoteType::Fact)).unwrap_or_default();
    assert_eq!(by_type.len(), 2);
    let by_foundational = members_of(
        &clusters,
        &ClusterAxis::Classification(Classification::Foundational),
    )
    .unwrap_or_default();
    assert_eq!(by_foundational, [foundational.id()?.to_string()]);
    let by_tactical = members_of(
        &clusters,
        &ClusterAxis::Classification(Classification::Tactical),
    )
    .unwrap_or_default();
    assert_eq!(by_tactical, [tactical.id()?.to_string()]);
    Ok(())
}

#[test]
fn container_of_finds_nearest_container() -> Result<()> {
    let mut plan = Draft::new(NoteType::Container, "plano");
    plan.scope = Some(Scope::Plan);
    let plan_note = plan.to_note(NOW)?;
    let plan_id = plan_note.id()?.to_string();

    let mut epic = Draft::new(NoteType::Container, "épico");
    epic.scope = Some(Scope::Epic);
    epic.edges = vec![(EdgeKind::DependsOn, plan_id)];
    let epic_note = epic.to_note(NOW)?;
    let epic_id = epic_note.id()?.to_string();

    let mut task = Draft::new(NoteType::Task, "tarefa");
    task.scope = Some(Scope::Task);
    task.edges = vec![(EdgeKind::DependsOn, epic_id.clone())];
    let task_note = task.to_note(NOW)?;
    let task_id = task_note.id()?.to_string();

    let graph = Graph::from_notes(vec![plan_note, epic_note, task_note])?;
    assert_eq!(container_of(&graph, &task_id), Some(epic_id));
    Ok(())
}

#[test]
fn clusters_are_deterministic() -> Result<()> {
    let a = note_created(NoteType::Fact, "a", "", NOW)?;
    let b = classified("b", Classification::Observational, NOW)?;
    let (index, graph) = built(&[a, b])?;
    let first = structural_clusters(&index, &graph);
    let second = structural_clusters(&index, &graph);
    assert_eq!(first, second);
    Ok(())
}
