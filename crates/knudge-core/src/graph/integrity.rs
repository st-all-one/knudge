//! Integridade do grafo (D46).
//!
//! Checa arestas penduradas (dangling), auto-arestas e a bidirecionalidade
//! `replaces ↔ superseded_by`. Diagnóstico consumido pelo `doctor` (E09).

use crate::schema::EdgeKind;

use super::Graph;

/// Tipo de problema de integridade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IssueKind {
    /// Aresta para id inexistente (dangling).
    Dangling,
    /// `replaces` sem o `superseded_by` correspondente na nota antiga.
    MissingBackref,
    /// `superseded_by` sem o `replaces` correspondente na nota nova.
    MissingForward,
    /// A aresta aponta para a própria nota.
    SelfEdge,
}

impl IssueKind {
    /// Rótulo estável de máquina.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dangling => "dangling",
            Self::MissingBackref => "missing_backref",
            Self::MissingForward => "missing_forward",
            Self::SelfEdge => "self_edge",
        }
    }
}

/// Problema de integridade encontrado no grafo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// Tipo do problema.
    pub kind: IssueKind,
    /// Nota de origem.
    pub from: String,
    /// Nota de destino (ou a própria, em [`IssueKind::SelfEdge`]).
    pub to: String,
    /// Aresta envolvida, quando aplicável.
    pub edge: Option<EdgeKind>,
}

impl Graph {
    /// Diagnóstico de integridade, ordenado e determinístico.
    #[must_use]
    pub fn integrity(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        for node in self.nodes.values() {
            for (kind, targets) in &node.edges {
                for target in targets {
                    if target == &node.id {
                        issues.push(issue(IssueKind::SelfEdge, &node.id, target, Some(*kind)));
                    } else if !self.nodes.contains_key(target) {
                        issues.push(issue(IssueKind::Dangling, &node.id, target, Some(*kind)));
                    }
                }
            }
            self.check_supersession(node, &mut issues);
        }
        issues.sort_unstable_by(|left, right| {
            (&left.kind, &left.from, &left.to, left.edge).cmp(&(
                &right.kind,
                &right.from,
                &right.to,
                right.edge,
            ))
        });
        issues
    }

    fn check_supersession(&self, node: &super::Node, issues: &mut Vec<Issue>) {
        if let Some(superseded) = &node.superseded_by {
            match self.nodes.get(superseded) {
                None => issues.push(issue(
                    IssueKind::Dangling,
                    &node.id,
                    superseded,
                    Some(EdgeKind::Replaces),
                )),
                Some(successor) => {
                    let acknowledges = successor
                        .edges
                        .get(&EdgeKind::Replaces)
                        .is_some_and(|targets| targets.contains(&node.id));
                    if !acknowledges {
                        issues.push(issue(
                            IssueKind::MissingForward,
                            &node.id,
                            superseded,
                            Some(EdgeKind::Replaces),
                        ));
                    }
                }
            }
        }
        for target in node
            .edges
            .get(&EdgeKind::Replaces)
            .map_or(&[][..], Vec::as_slice)
        {
            match self.nodes.get(target) {
                Some(old) if old.superseded_by.as_deref() == Some(node.id.as_str()) => {}
                Some(_) => issues.push(issue(
                    IssueKind::MissingBackref,
                    &node.id,
                    target,
                    Some(EdgeKind::Replaces),
                )),
                // Dangling já é reportado no laço genérico de arestas.
                None => {}
            }
        }
    }
}

fn issue(kind: IssueKind, from: &str, to: &str, edge: Option<EdgeKind>) -> Issue {
    Issue {
        kind,
        from: from.to_string(),
        to: to.to_string(),
        edge,
    }
}
