//! `kd task graph` — WBS derivado (D116/D119): papel e modo por nó.

use std::collections::{BTreeMap, BTreeSet};

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::schema::{EdgeKind, NoteType, Value};
use knudge_core::store::Store;
use knudge_core::task::{
    Child, Container, Mode, children, mode, progress_of, role, roots_for_path, subtree,
};
use serde_json::json;

use crate::output::Output;
use crate::session::Session;

/// `kd task graph [--program PATH|--root ID]` — WBS derivado (D116/D119).
///
/// # Errors
/// Retorna `ErrorKind::NotFound` se o programa/raiz não existir; propaga erros de I/O.
pub(super) fn graph_tree(
    session: &Session,
    program: Option<&str>,
    root: Option<&str>,
) -> Result<Output> {
    let store = session.store();
    let graph = session.graph()?;
    let signals = Signals::collect(session)?;
    let roots = resolve_roots(&store, &graph, program, root)?;
    let mut tree = Tree::default();
    if let Some(path) = program {
        tree.lines.push(path.to_string());
    }
    for root_id in &roots {
        render_subtree(&store, &graph, &signals, root_id, &mut tree)?;
    }
    let empty = roots.is_empty();
    let data = json!({ "program": program, "roots": roots, "nodes": tree.json_rows });
    let output = Output::new(tree.lines.join("\n"), data);
    Ok(if empty {
        output.with_warnings(vec![
            "nenhum container: crie um épico com `kd task new --scope epic` (ou use --program/--root)"
                .to_string(),
        ])
    } else {
        output
    })
}

/// Acumulador da árvore (linhas do pipe + nós JSON).
#[derive(Default)]
struct Tree {
    lines: Vec<String>,
    json_rows: Vec<serde_json::Value>,
}

/// Sinais derivados do log (criação) por id.
struct Signals {
    created: BTreeMap<String, i64>,
}

impl Signals {
    fn collect(session: &Session) -> Result<Self> {
        let (events, _warnings) = session.events().read_all()?;
        let mut created = BTreeMap::new();
        for record in &events {
            let Some(id) = record.note_id.as_deref() else {
                continue;
            };
            let submit = record.data.get("action").and_then(Value::as_str) == Some("submit");
            if record.op == "task" && submit {
                let _ignored = created.insert(id.to_string(), record.at);
            }
        }
        Ok(Self { created })
    }

    fn incremental(&self, children: &[String]) -> bool {
        let stamps: BTreeSet<i64> = children
            .iter()
            .filter_map(|id| self.created.get(id).copied())
            .collect();
        stamps.len() >= 2
    }

    fn container(&self, graph: &Graph, id: &str) -> Container {
        let mut nodes = Vec::new();
        let mut ids = Vec::new();
        for child in children(graph, id) {
            ids.push(child.clone());
            nodes.push(Child {
                id: child.clone(),
                depends_on: graph.targets(&child, EdgeKind::DependsOn).to_vec(),
            });
        }
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        ids.sort();
        Container {
            children: nodes,
            incremental: self.incremental(&ids),
        }
    }
}

fn resolve_roots(
    store: &Store<'_>,
    graph: &Graph,
    program: Option<&str>,
    root: Option<&str>,
) -> Result<Vec<String>> {
    if let Some(path) = program {
        let mut notes = Vec::new();
        for id in store.list_ids()? {
            if let Some(note) = store.read_optional(&id)? {
                notes.push(note);
            }
        }
        let roots = roots_for_path(&notes, path)?;
        if roots.is_empty() {
            return Err(Error::not_found(format!(
                "nenhum Épico-raiz ancorado a {path}"
            )));
        }
        return Ok(roots);
    }
    if let Some(root) = root {
        if !graph.contains(root) {
            return Err(Error::not_found(format!("nó {root} não existe")));
        }
        return Ok(vec![root.to_string()]);
    }
    let mut roots = Vec::new();
    for id in store.list_ids()? {
        let Some(note) = store.read_optional(&id)? else {
            continue;
        };
        if note.frontmatter.note_type()? == NoteType::Epic && !graph.has_parent(&id) {
            roots.push(id);
        }
    }
    Ok(roots)
}

fn render_subtree(
    store: &Store<'_>,
    graph: &Graph,
    signals: &Signals,
    root: &str,
    tree: &mut Tree,
) -> Result<()> {
    let nodes = subtree(graph, root);
    for entry in &nodes {
        let note = store.read(&entry.id)?;
        let kind = note.frontmatter.note_type()?;
        note.frontmatter
            .scope()?
            .ok_or_else(|| Error::schema(format!("{} não é item de trabalho", entry.id)))?;
        let status = note.frontmatter.status()?;
        let statement = note.frontmatter.statement().unwrap_or_default().to_string();
        let has_children = !children(graph, &entry.id).is_empty();
        let role = role::role(entry.depth, kind, has_children);
        let mode = if kind == NoteType::Epic {
            Some(mode::mode(&signals.container(graph, &entry.id)))
        } else {
            None
        };
        let progress = (kind == NoteType::Epic).then(|| progress_of(graph, &entry.id));
        let progress_label =
            progress.map_or(String::new(), |value| format!(" ({})", value.label()));
        let indent = "  ".repeat(entry.depth.saturating_add(1));
        tree.lines.push(format!(
            "{indent}{}|{}|{}|{statement}{progress_label}",
            entry.id,
            kind.as_str(),
            status.as_str()
        ));
        tree.json_rows.push(json!({
            "id": entry.id,
            "depth": entry.depth,
            "role": role.as_str(),
            "kind": kind.as_str(),
            "status": status.as_str(),
            "mode": mode.map(Mode::as_str),
            "statement": statement,
            "progress": progress.map(|value| json!({"done": value.done, "total": value.total})),
        }));
    }
    Ok(())
}
