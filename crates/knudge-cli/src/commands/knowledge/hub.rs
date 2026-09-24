//! Materialização do mapa de conhecimento (D150): `MAP.md` + notas-hub (`references`).
//!
//! Um **hub** é uma nota `meta` real que `references` os membros de um cluster — entra no `ask`
//! e versiona o mapa como grafo. O `MAP.md` é o índice legível (aponta para os hubs). Tudo é
//! **versionado** (não derivado): o churn é aceito em troca de ponto de entrada humano.

use knudge_core::Result;
use knudge_core::lifecycle::{Cluster, ClusterAxis, SemanticCluster};
use knudge_core::schema::{EdgeKind, NoteType, id};
use knudge_core::store::Store;
use knudge_core::write::Draft;

use crate::session::Session;

/// Materializa `notas/MAP.md` + uma nota-hub por cluster. Devolve os ids dos hubs.
///
/// # Errors
/// Propaga erros de leitura/escrita do store.
pub fn materialize(
    session: &Session,
    clusters: &[Cluster],
    semantic: &[SemanticCluster],
) -> Result<Vec<String>> {
    let store = session.store();
    let mut hubs = Vec::new();
    let mut map_lines = vec![
        "# Mapa de conhecimento".to_string(),
        String::new(),
        "## Clusters estruturais".to_string(),
    ];
    for cluster in clusters {
        let statement = structural_statement(&cluster.axis);
        let hub = write_hub(session, &store, &statement, &cluster.members)?;
        map_lines.push(format!(
            "- `{}` `{}` — {} nota(s) → `{hub}`",
            cluster.axis.axis(),
            cluster.axis.key(),
            cluster.members.len()
        ));
        hubs.push(hub);
    }
    if !semantic.is_empty() {
        map_lines.push(String::new());
        map_lines.push("## Clusters semânticos".to_string());
        for entry in semantic {
            for (index, group) in entry.groups.iter().enumerate() {
                let statement = semantic_statement(&entry.parent.axis, index);
                let hub = write_hub(session, &store, &statement, group)?;
                map_lines.push(format!(
                    "- `{}` `{}` #{} — {} nota(s) → `{hub}`",
                    entry.parent.axis.axis(),
                    entry.parent.axis.key(),
                    index.saturating_add(1),
                    group.len()
                ));
                hubs.push(hub);
            }
        }
    }
    map_lines.push(String::new());
    let path = store.notes_dir().join("MAP.md");
    session
        .fs_dyn()
        .write_atomic(&path, map_lines.join("\n").as_bytes())?;
    Ok(hubs)
}

/// Afirmação canônica do hub de um cluster estrutural.
fn structural_statement(axis: &ClusterAxis) -> String {
    format!("Mapa de conhecimento: {} {}", axis.axis(), axis.key())
}

/// Afirmação canônica do hub de um subgrupo semântico.
fn semantic_statement(axis: &ClusterAxis, index: usize) -> String {
    format!(
        "Mapa semântico: {} {} #{}",
        axis.axis(),
        axis.key(),
        index.saturating_add(1)
    )
}

/// Cria/atualiza a nota-hub; idempotente (não reescreve se os membros não mudaram).
fn write_hub(
    session: &Session,
    store: &Store<'_>,
    statement: &str,
    members: &[String],
) -> Result<String> {
    let hub_id = id::note_id(NoteType::Meta, statement);
    let mut sorted: Vec<String> = members.to_vec();
    sorted.sort();
    if let Ok(existing) = store.read(&hub_id) {
        let current: Vec<String> = existing
            .frontmatter
            .string_list("references")?
            .into_iter()
            .map(str::to_string)
            .collect();
        if current == sorted {
            return Ok(hub_id);
        }
    }
    let mut draft = Draft::new(NoteType::Meta, statement);
    draft.body = hub_body(&sorted);
    for member in &sorted {
        draft.edges.push((EdgeKind::References, member.clone()));
    }
    let note = draft.to_note(session.now_ms())?;
    store.write(&note)?;
    Ok(hub_id)
}

/// Corpo do hub: lista legível dos membros.
fn hub_body(members: &[String]) -> String {
    let mut body = String::from("Membros:\n");
    for member in members {
        body.push_str("- ");
        body.push_str(member);
        body.push('\n');
    }
    body
}
