//! `kd knowledge map` — visualiza o mapa de conhecimento: clusters estruturais (fase 1) e, com
//! `--semantic`, semânticos (fase 2) — D128.
//!
//! Fase 1 é determinística e sem embeddings (agrega por `anchor`/`type`/`classification`/
//! `container`); fase 2 só roda **dentro** de um cluster estrutural acima de `clusters.min_volume`.

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::embeddings::{EmbeddingIndex, similarity};
use knudge_core::graph::Graph;
use knudge_core::handoff::manifest::belongs_to;
use knudge_core::lifecycle::{
    Cluster, ClusterAxis, MIN_SEMANTIC_VOLUME, SemanticCluster, semantic_clusters,
    structural_clusters_filtered,
};
use knudge_core::store::Store;
use serde_json::json;

use crate::cli::KnowledgeMapArgs;
use crate::output::Output;
use crate::session::Session;

use super::super::corpus::CorpusScope;
use super::super::embedder;
use super::hub;

/// Linhas de texto + nós JSON de uma seção.
type Section = (Vec<String>, Vec<serde_json::Value>);

/// `kd knowledge map [--axis A] [--scope C] [--semantic] [--members] [--write]` + filtros de
/// corpus (`--tag`/`--anchor`/`--type`/`--class`/`--around`/`--universe`) — D128/D143.
///
/// # Errors
/// Propaga erros de leitura do índice/store/config; `invalid_input` para eixo desconhecido e
/// quando não há escopo nem `--universe` (D143).
pub fn run(session: &Session, args: &KnowledgeMapArgs) -> Result<Output> {
    if let Some(axis) = args.axis.as_deref() {
        validate_axis(axis)?;
    }
    let scope = CorpusScope {
        types: args.types.clone(),
        classes: args.classes.clone(),
        tags: args.tags.clone(),
        anchors: args.anchor.clone(),
        around: args.around.clone(),
        depth: args.depth,
        universe: args.universe,
    };
    scope.require("knowledge map")?;
    let index = session.index()?;
    let graph = session.graph()?;
    let selection = scope.select(&index, &graph)?;
    let mut clusters = structural_clusters_filtered(&index, &graph, selection.filter());
    if let Some(allowed) = selection.allowed() {
        clusters = restrict(clusters, allowed);
    }
    if let Some(axis) = args.axis.as_deref() {
        clusters.retain(|cluster| cluster.axis.axis() == axis);
    }
    if let Some(scope) = args.scope.as_deref() {
        clusters = scope_clusters(clusters, &graph, scope);
    }
    let store = session.store();
    let docs = selection.docs();
    let mut lines = vec![format!("docs={docs} clusters={}", clusters.len())];
    let mut data = Vec::new();
    for cluster in &clusters {
        let (line, value) = render_cluster(&store, cluster, args.members);
        lines.push(line);
        data.push(value);
    }
    let mut warnings = Vec::new();
    let mut semantic_entries = Vec::new();
    let semantic_data = if args.semantic {
        let ((semantic_lines, semantic_json), entries) =
            semantic_section(session, &clusters, &mut warnings)?;
        semantic_entries = entries;
        lines.extend(semantic_lines);
        semantic_json
    } else {
        Vec::new()
    };
    if args.write {
        let hubs = hub::materialize(session, &clusters, &semantic_entries)?;
        warnings.push(format!(
            "mapa materializado: {n} hub(s) + notas/MAP.md",
            n = hubs.len()
        ));
    }
    let value = json!({
        "docs": docs,
        "clusters": data,
        "semantic": semantic_data,
    });
    Ok(Output::new(lines.join("\n"), value).with_warnings(warnings))
}

/// Restringe os membros dos clusters à vizinhança (`--around`) e descarta clusters vazios.
fn restrict(clusters: Vec<Cluster>, allowed: &std::collections::BTreeSet<String>) -> Vec<Cluster> {
    clusters
        .into_iter()
        .filter_map(|cluster| {
            let members: Vec<String> = cluster
                .members
                .into_iter()
                .filter(|member| allowed.contains(member))
                .collect();
            (!members.is_empty()).then_some(Cluster {
                axis: cluster.axis,
                members,
            })
        })
        .collect()
}

/// Valida o nome do eixo.
fn validate_axis(axis: &str) -> Result<()> {
    const AXES: [&str; 4] = ["anchor", "type", "classification", "scope"];
    if AXES.contains(&axis) {
        Ok(())
    } else {
        Err(Error::invalid_input(format!(
            "eixo desconhecido: {axis:?} (use anchor|type|classification|scope)"
        )))
    }
}

/// Restringe os membros aos que pertencem ao container dado (`belongs_to`).
fn scope_clusters(clusters: Vec<Cluster>, graph: &Graph, scope: &str) -> Vec<Cluster> {
    clusters
        .into_iter()
        .filter_map(|cluster| {
            let members: Vec<String> = cluster
                .members
                .into_iter()
                .filter(|member| belongs_to(graph, member, scope))
                .collect();
            (!members.is_empty()).then_some(Cluster {
                axis: cluster.axis,
                members,
            })
        })
        .collect()
}

/// Renderiza um cluster (texto + JSON).
#[allow(
    clippy::fn_params_excessive_bools,
    reason = "`members` é a flag de CLI `--members`"
)]
fn render_cluster(
    store: &Store<'_>,
    cluster: &Cluster,
    members: bool,
) -> (String, serde_json::Value) {
    let statement = match &cluster.axis {
        ClusterAxis::Scope(id) => statement_of(store, id),
        _ => String::new(),
    };
    let mut line = if statement.is_empty() {
        format!(
            "{}|{}|{}",
            cluster.axis.axis(),
            cluster.axis.key(),
            cluster.members.len()
        )
    } else {
        format!(
            "{}|{}|{statement}|{}",
            cluster.axis.axis(),
            cluster.axis.key(),
            cluster.members.len()
        )
    };
    let mut member_json = Vec::new();
    for member in &cluster.members {
        let member_statement = statement_of(store, member);
        if members {
            line.push_str("\n  ");
            line.push_str(member);
            line.push('|');
            line.push_str(&member_statement);
        }
        member_json.push(json!({ "id": member, "statement": member_statement }));
    }
    let value = json!({
        "axis": cluster.axis.axis(),
        "key": cluster.axis.key(),
        "statement": statement,
        "count": cluster.members.len(),
        "members": member_json,
    });
    (line, value)
}

/// Fase 2: agrupa por similaridade dentro dos clusters acima de `clusters.min_volume`.
fn semantic_section(
    session: &Session,
    structural: &[Cluster],
    warnings: &mut Vec<String>,
) -> Result<(Section, Vec<SemanticCluster>)> {
    let meta = embedder::meta(session)?;
    let Some(index) =
        EmbeddingIndex::load(session.fs_dyn(), &session.knowledge_dir(), &meta, warnings)?
    else {
        warnings.push(
            "fase 2 ignorada: sem índice de embeddings (rode `kd drain --status`)".to_string(),
        );
        return Ok(((Vec::new(), Vec::new()), Vec::new()));
    };
    let metric = meta.similarity;
    let entries = semantic_clusters(
        structural,
        min_volume(session),
        similarity_threshold(session),
        |a, b| match (index.vector(a), index.vector(b)) {
            (Some(va), Some(vb)) => f64::from(similarity(va, vb, metric)),
            _ => 0.0,
        },
    );
    let mut lines = Vec::new();
    let mut data = Vec::new();
    for entry in &entries {
        lines.push(format!(
            "semantic|{}|{}|groups={}",
            entry.parent.axis.axis(),
            entry.parent.axis.key(),
            entry.groups.len()
        ));
        for group in &entry.groups {
            lines.push(format!("  {}", group.join(", ")));
        }
        data.push(json!({
            "axis": entry.parent.axis.axis(),
            "key": entry.parent.axis.key(),
            "groups": entry.groups,
        }));
    }
    Ok(((lines, data), entries))
}

/// `clusters.min_volume` (default [`MIN_SEMANTIC_VOLUME`]).
fn min_volume(session: &Session) -> usize {
    session
        .config()
        .get_int("clusters.min_volume")
        .and_then(|raw| usize::try_from(raw).ok())
        .unwrap_or(MIN_SEMANTIC_VOLUME)
}

/// `clusters.similarity_threshold` (default `0.8`).
fn similarity_threshold(session: &Session) -> f64 {
    session
        .config()
        .get_float("clusters.similarity_threshold")
        .unwrap_or(0.8)
}

/// Afirmação de uma nota (vazia se ausente — R33).
fn statement_of(store: &Store<'_>, id: &str) -> String {
    store
        .read(id)
        .ok()
        .and_then(|note| note.frontmatter.statement().ok().map(str::to_string))
        .unwrap_or_default()
}
