//! `audit`: relatório de integridade e saúde do corpus (D46, E09-T03).
//!
//! Leitura pura, nunca mutação: reúne problemas de integridade, ciclos, âncoras quebradas,
//! duplicatas, arestas sugeridas faltantes e locks stale. Determinístico e ordenado.

use std::path::Path;

use crate::Result;
use crate::graph::Graph;
use crate::graph::integrity::Issue;
use crate::graph::suggestions::SuggestionStore;
use crate::ports::Fs;
use crate::retrieval::Index;
use crate::store::Store;
use crate::write::dedup::{DedupThresholds, propose_merges};

use super::anchors::is_glob;

/// Âncora literal que aponta para arquivo inexistente.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BrokenAnchor {
    /// Id da nota.
    pub id: String,
    /// Caminho da âncora.
    pub anchor: String,
}

/// Par de quase-duplicatas.
#[derive(Debug, Clone, PartialEq)]
pub struct Duplicate {
    /// Nota que permanece.
    pub keep: String,
    /// Nota a fundir.
    pub drop: String,
    /// Similaridade.
    pub score: f64,
}

/// Aresta sugerida ainda não materializada.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MissingEdge {
    /// Nota de origem.
    pub id: String,
    /// Tipo da aresta.
    pub kind: String,
    /// Alvos ausentes.
    pub targets: Vec<String>,
    /// Motivo.
    pub reason: String,
}

/// Lock abandonado no disco.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StaleLock {
    /// Caminho do arquivo de lock.
    pub path: String,
    /// Idade em ms.
    pub age_ms: i64,
}

/// Relatório de auditoria.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AuditReport {
    /// Problemas de integridade do grafo.
    pub integrity: Vec<Issue>,
    /// Ciclos de supersessão.
    pub supersession_cycles: Vec<Vec<String>>,
    /// Ciclos de dependência.
    pub dependency_cycles: Vec<Vec<String>>,
    /// Âncoras quebradas.
    pub broken_anchors: Vec<BrokenAnchor>,
    /// Quase-duplicatas.
    pub duplicates: Vec<Duplicate>,
    /// Arestas sugeridas faltantes.
    pub missing_edges: Vec<MissingEdge>,
    /// Locks stale.
    pub stale_locks: Vec<StaleLock>,
}

impl AuditReport {
    /// `true` se não há nenhum problema.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.integrity.is_empty()
            && self.supersession_cycles.is_empty()
            && self.dependency_cycles.is_empty()
            && self.broken_anchors.is_empty()
            && self.duplicates.is_empty()
            && self.missing_edges.is_empty()
            && self.stale_locks.is_empty()
    }

    /// Soma de todos os problemas encontrados.
    #[must_use]
    pub fn total(&self) -> usize {
        [
            self.integrity.len(),
            self.supersession_cycles.len(),
            self.dependency_cycles.len(),
            self.broken_anchors.len(),
            self.duplicates.len(),
            self.missing_edges.len(),
            self.stale_locks.len(),
        ]
        .into_iter()
        .fold(0_usize, usize::saturating_add)
    }
}

/// Contexto da auditoria.
pub struct AuditInput<'a> {
    /// Porta de FS.
    pub fs: &'a dyn Fs,
    /// Raiz do `.knudge/`.
    pub root: &'a Path,
    /// Raiz do projeto (âncoras são relativas a ela).
    pub project_root: &'a Path,
    /// Store de notas.
    pub store: &'a Store<'a>,
    /// Grafo de arestas.
    pub graph: &'a Graph,
    /// Índice derivado.
    pub index: &'a Index,
    /// Instante atual (ms).
    pub now_ms: i64,
    /// Idade a partir da qual um lock é stale (ms).
    pub lock_stale_ms: i64,
    /// Limiares de dedup.
    pub thresholds: &'a DedupThresholds,
}

/// Executa a auditoria (somente leitura).
///
/// # Errors
/// Propaga erros de I/O e de listagem do derivado.
pub fn audit(input: &AuditInput<'_>) -> Result<AuditReport> {
    Ok(AuditReport {
        integrity: input.graph.integrity(),
        supersession_cycles: input.graph.supersession_cycles(),
        dependency_cycles: input.graph.dependency_cycles(),
        broken_anchors: broken_anchors(input),
        duplicates: duplicates(input),
        missing_edges: missing_edges(input)?,
        stale_locks: stale_locks(input)?,
    })
}

fn broken_anchors(input: &AuditInput<'_>) -> Vec<BrokenAnchor> {
    let mut broken = Vec::new();
    for doc in &input.index.docs {
        for anchor in &doc.meta.anchors {
            if is_glob(anchor) {
                continue;
            }
            if !input.fs.exists(&input.project_root.join(anchor)) {
                broken.push(BrokenAnchor {
                    id: doc.meta.id.clone(),
                    anchor: anchor.clone(),
                });
            }
        }
    }
    broken.sort();
    broken
}

fn duplicates(input: &AuditInput<'_>) -> Vec<Duplicate> {
    propose_merges(input.index, input.thresholds)
        .into_iter()
        .map(|merge| Duplicate {
            keep: merge.keep,
            drop: merge.drop,
            score: merge.score,
        })
        .collect()
}

fn missing_edges(input: &AuditInput<'_>) -> Result<Vec<MissingEdge>> {
    let store = SuggestionStore::new(input.fs, input.root);
    let mut missing = Vec::new();
    for record in store.list()? {
        let present = input
            .graph
            .targets(&record.id, record.kind)
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let absent: Vec<String> = record
            .targets
            .iter()
            .filter(|target| !present.contains(*target))
            .cloned()
            .collect();
        if absent.is_empty() {
            continue;
        }
        missing.push(MissingEdge {
            id: record.id,
            kind: record.kind.as_str().to_string(),
            targets: absent,
            reason: record.reason,
        });
    }
    missing.sort();
    Ok(missing)
}

fn stale_locks(input: &AuditInput<'_>) -> Result<Vec<StaleLock>> {
    let dir = input.root.join(".locks");
    if !input.fs.exists(&dir) {
        return Ok(Vec::new());
    }
    let mut stale = Vec::new();
    for path in input.fs.list_dir(&dir)? {
        if !is_lock_file(&path) {
            continue;
        }
        let Ok(Some(modified)) = input.fs.modified_ms(&path) else {
            continue;
        };
        let age = input.now_ms.saturating_sub(modified);
        if age > input.lock_stale_ms {
            stale.push(StaleLock {
                path: path.display().to_string(),
                age_ms: age,
            });
        }
    }
    stale.sort();
    Ok(stale)
}

/// `true` se o caminho é um lock (`*.lock`) ou sidecar (`*.stale`).
fn is_lock_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("lock" | "stale")
    )
}

/// Lista os arquivos de lock stale (para o `--fix`).
///
/// # Errors
/// Propaga erros de listagem.
pub fn stale_lock_paths(
    fs: &dyn Fs,
    root: &Path,
    now_ms: i64,
    stale_ms: i64,
) -> Result<Vec<std::path::PathBuf>> {
    let dir = root.join(".locks");
    if !fs.exists(&dir) {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for path in fs.list_dir(&dir)? {
        if !is_lock_file(&path) {
            continue;
        }
        let Ok(Some(modified)) = fs.modified_ms(&path) else {
            continue;
        };
        if now_ms.saturating_sub(modified) > stale_ms {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}
