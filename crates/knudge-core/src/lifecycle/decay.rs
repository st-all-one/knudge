//! Decay de âncoras no rebuild (D43, E10-T02).
//!
//! Valida as `anchors` (literal existe? glob ainda casa?) e **demove** a nota quando a fração
//! válida cai abaixo do threshold **e** o grace period já venceu. A varredura do projeto é
//! **limitada** (profundidade/entradas) e roda **off-path** (no rebuild), nunca no `recall`.

use std::ffi::OsStr;
use std::path::Path;

use crate::config::Config;
use crate::ports::Fs;
use crate::retrieval::anchor::glob_match;

/// Profundidade máxima da varredura de projeto.
pub const MAX_WALK_DEPTH: usize = 32;

/// Tetos de entradas da varredura (proteção de recursos — R11).
pub const MAX_WALK_ENTRIES: usize = 20_000;

/// Diretórios ignorados na varredura de âncoras.
pub const IGNORED_DIRS: [&str; 5] = [".git", ".knudge", "target", "node_modules", ".hg"];

/// Validade das âncoras de uma nota.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AnchorValidity {
    /// Total de âncoras declaradas.
    pub total: u32,
    /// Âncoras válidas (arquivo existe / glob casa).
    pub valid: u32,
    /// Âncoras quebradas.
    pub broken: u32,
}

impl AnchorValidity {
    /// Fração válida em `[0,1]` (`1.0` quando não há âncoras).
    #[must_use]
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            return 1.0;
        }
        f64::from(self.valid) / f64::from(self.total)
    }
}

/// Política de decay de âncoras.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecayPolicy {
    /// Fração válida mínima (abaixo dela, candidata a demolição).
    pub threshold: f64,
    /// Dias de carência antes de demover.
    pub grace_days: i64,
}

impl Default for DecayPolicy {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            grace_days: 30,
        }
    }
}

impl DecayPolicy {
    /// Lê a política da config efetiva.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let defaults = Self::default();
        Self {
            threshold: config
                .get_float("decay.anchor_threshold")
                .unwrap_or(defaults.threshold),
            grace_days: config
                .get_int("decay.grace_days")
                .unwrap_or(defaults.grace_days),
        }
    }
}

/// `true` se a nota deve ser demovida (fração baixa **e** grace vencido).
#[must_use]
pub fn should_demote(validity: &AnchorValidity, policy: &DecayPolicy, age_days: i64) -> bool {
    validity.total > 0 && validity.fraction() < policy.threshold && age_days >= policy.grace_days
}

/// Valida as âncoras contra a árvore do projeto (literais e globs).
#[must_use]
pub fn compute_anchor_validity(
    fs: &dyn Fs,
    project_root: &Path,
    anchors: &[String],
) -> AnchorValidity {
    let paths = walk_paths(fs, project_root);
    compute_anchor_validity_cached(fs, project_root, &paths, anchors)
}

/// Como [`compute_anchor_validity`], mas reusa os caminhos já coletados (E16/T10).
///
/// O `prune` valida as âncoras de **todas** as notas: caminhar o projeto uma vez e reusar a lista
/// troca `O(notas × projeto)` por `O(projeto + notas)` sem mudar o resultado.
#[must_use]
pub fn compute_anchor_validity_cached(
    fs: &dyn Fs,
    project_root: &Path,
    paths: &[String],
    anchors: &[String],
) -> AnchorValidity {
    compute_anchor_validity_with(anchors, |anchor| {
        if has_glob(anchor) {
            paths.iter().any(|path| glob_match(anchor, path))
        } else {
            let path = project_root.join(anchor);
            // Diretório não é âncora de conteúdo (mesma regra do verify-on-hit/D86).
            fs.exists(&path) && !fs.is_dir(&path)
        }
    })
}

/// Valida as âncoras com um oráculo injetado (puro e testável).
pub fn compute_anchor_validity_with(
    anchors: &[String],
    is_valid: impl Fn(&str) -> bool,
) -> AnchorValidity {
    let mut validity = AnchorValidity::default();
    for anchor in anchors {
        validity.total = validity.total.saturating_add(1);
        if is_valid(anchor) {
            validity.valid = validity.valid.saturating_add(1);
        } else {
            validity.broken = validity.broken.saturating_add(1);
        }
    }
    validity
}

/// `true` se a âncora contém metacaracteres de glob.
#[must_use]
pub fn has_glob(anchor: &str) -> bool {
    anchor.contains(['*', '?', '['])
}

/// Caminhos relativos do projeto, ordenados e limitados (ignora `.git`/`target`/…).
///
/// Usa [`IGNORED_DIRS`]; para um layout de conhecimento alternativo (ex.: `.a/b`), prefira
/// [`walk_paths_ignoring`] com [`Project::ignored_dirs`](crate::git::Project::ignored_dirs).
#[must_use]
pub fn walk_paths(fs: &dyn Fs, root: &Path) -> Vec<String> {
    let ignored: Vec<String> = IGNORED_DIRS
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    walk_paths_ignoring(fs, root, &ignored)
}

/// Como [`walk_paths`], mas ignora os diretórios de topo em `ignored`.
#[must_use]
pub fn walk_paths_ignoring(fs: &dyn Fs, root: &Path, ignored: &[String]) -> Vec<String> {
    let mut walker = Walker {
        fs,
        root,
        ignored,
        out: Vec::new(),
    };
    walker.visit(root, 0);
    let mut out = walker.out;
    out.sort();
    out.dedup();
    out
}

/// Estado da varredura (props para manter a assinatura enxuta).
struct Walker<'a> {
    fs: &'a dyn Fs,
    root: &'a Path,
    ignored: &'a [String],
    out: Vec<String>,
}

impl Walker<'_> {
    fn visit(&mut self, dir: &Path, depth: usize) {
        if depth > MAX_WALK_DEPTH || self.out.len() >= MAX_WALK_ENTRIES {
            return;
        }
        let Ok(entries) = self.fs.list_dir(dir) else {
            return;
        };
        for path in entries {
            if self.out.len() >= MAX_WALK_ENTRIES {
                return;
            }
            let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");
            if self.fs.is_dir(&path) {
                if self.ignored.iter().any(|ignored| ignored == name) {
                    continue;
                }
                self.visit(&path, depth.saturating_add(1));
            } else if let Ok(relative) = path.strip_prefix(self.root) {
                self.out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}
