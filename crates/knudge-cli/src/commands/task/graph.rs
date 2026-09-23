//! `kd task graph` — WBS derivado (D116/D119): papel, dono e modo por nó.

use std::collections::{BTreeMap, BTreeSet};

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::schema::{EdgeKind, NoteType, Value};
use knudge_core::store::Store;
use knudge_core::task::ownership::CLAIM;
use knudge_core::task::{
    Child, Container, Mode, children, mode, ownership, role, root_for_path, subtree,
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
    let data = json!({ "program": program, "roots": roots, "nodes": tree.json_rows });
    Ok(Output::new(tree.lines.join("\n"), data))
}

/// Acumulador da árvore (linhas do pipe + nós JSON).
#[derive(Default)]
struct Tree {
    lines: Vec<String>,
    json_rows: Vec<serde_json::Value>,
}

/// Sinais derivados do log (dono, claims, criação) por id.
struct Signals {
    owners: BTreeMap<String, Option<String>>,
    claims: BTreeMap<String, usize>,
    created: BTreeMap<String, i64>,
}

impl Signals {
    fn collect(session: &Session) -> Result<Self> {
        let (events, _warnings) = session.events().read_all()?;
        let mut owners = BTreeMap::new();
        for id in session.store().list_ids()? {
            let _ignored = owners.insert(id.clone(), ownership(&events, &id));
        }
        let mut claims: BTreeMap<String, usize> = BTreeMap::new();
        let mut created = BTreeMap::new();
        for record in &events {
            let Some(id) = record.note_id.as_deref() else {
                continue;
            };
            if record.op == CLAIM {
                let count = claims.entry(id.to_string()).or_insert(0_usize);
                *count = count.saturating_add(1);
            }
            let submit = record.data.get("action").and_then(Value::as_str) == Some("submit");
            if record.op == "task" && submit {
                let _ignored = created.insert(id.to_string(), record.at);
            }
        }
        Ok(Self {
            owners,
            claims,
            created,
        })
    }

    fn owner(&self, id: &str) -> Option<&str> {
        self.owners.get(id).and_then(Option::as_deref)
    }

    fn handoff(&self, ids: &[String]) -> bool {
        ids.iter()
            .any(|id| self.claims.get(id).copied().unwrap_or(0) >= 2)
    }

    fn incremental(&self, children: &[String]) -> bool {
        let stamps: BTreeSet<i64> = children
            .iter()
            .filter_map(|id| self.created.get(id).copied())
            .collect();
        stamps.len() >= 2
    }

    #[allow(
        clippy::fn_params_excessive_bools,
        reason = "`handoff` é um sinal derivado do log"
    )]
    fn container(&self, graph: &Graph, id: &str, handoff: bool) -> Container {
        let mut nodes = Vec::new();
        let mut ids = Vec::new();
        for child in children(graph, id) {
            ids.push(child.clone());
            nodes.push(Child {
                id: child.clone(),
                owner: self.owners.get(&child).and_then(Option::clone),
                depends_on: graph.targets(&child, EdgeKind::DependsOn).to_vec(),
            });
        }
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        ids.sort();
        Container {
            owner: self.owners.get(id).and_then(Option::clone),
            children: nodes,
            handoff,
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
            notes.push(store.read(&id)?);
        }
        let Some(root_id) = root_for_path(&notes, path)? else {
            return Err(Error::not_found(format!(
                "nenhum Épico-raiz ancorado a {path}"
            )));
        };
        return Ok(vec![root_id]);
    }
    if let Some(root) = root {
        if !graph.contains(root) {
            return Err(Error::not_found(format!("nó {root} não existe")));
        }
        return Ok(vec![root.to_string()]);
    }
    let mut roots = Vec::new();
    for id in store.list_ids()? {
        let note = store.read(&id)?;
        if note.frontmatter.note_type()? == NoteType::Container && !graph.has_parent(&id) {
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
    let ids: Vec<String> = nodes.iter().map(|entry| entry.id.clone()).collect();
    let handoff = signals.handoff(&ids);
    for entry in &nodes {
        let note = store.read(&entry.id)?;
        let kind = note.frontmatter.note_type()?;
        let scope = note
            .frontmatter
            .scope()?
            .ok_or_else(|| Error::schema(format!("{} não é item de trabalho", entry.id)))?;
        let status = note.frontmatter.status()?;
        let statement = note.frontmatter.statement().unwrap_or_default().to_string();
        let has_children = !children(graph, &entry.id).is_empty();
        let role = role::role(scope, kind, has_children);
        let owner = signals.owner(&entry.id).unwrap_or("-");
        let mode = if kind == NoteType::Container {
            Some(mode::mode(&signals.container(graph, &entry.id, handoff)))
        } else {
            None
        };
        let mode_label = mode.map_or("-", Mode::as_str);
        let indent = "  ".repeat(entry.depth.saturating_add(1));
        tree.lines.push(format!(
            "{indent}{}|{}|{}|{}|{owner}|{mode_label}|{statement}",
            entry.id,
            role.as_str(),
            kind.as_str(),
            status.as_str()
        ));
        tree.json_rows.push(json!({
            "id": entry.id,
            "depth": entry.depth,
            "role": role.as_str(),
            "kind": kind.as_str(),
            "status": status.as_str(),
            "owner": if owner == "-" { None } else { Some(owner) },
            "mode": mode.map(Mode::as_str),
            "statement": statement,
        }));
    }
    Ok(())
}
