//! Resolução de `checks` a partir do catálogo (E09-T01).

use std::collections::BTreeMap;

use super::{Severity, ValidatorCatalog};

/// Origem de um check resolvido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckSource {
    /// Declarado explicitamente pela task.
    Explicit,
    /// Global do catálogo (`AGENTS.md`).
    Global,
    /// Aplicado por âncora.
    Anchor,
}

impl CheckSource {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Global => "global",
            Self::Anchor => "anchor",
        }
    }
}

/// Check resolvido, com a fonte que o introduziu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCheck {
    /// Nome do validator.
    pub name: String,
    /// Fonte.
    pub source: CheckSource,
    /// Severidade herdada do catálogo.
    pub severity: Severity,
}

/// Resultado da resolução de `checks`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedChecks {
    /// Checks resolvidos, ordenados por nome.
    pub checks: Vec<ResolvedCheck>,
    /// Nomes explícitos que não existem no catálogo.
    pub missing: Vec<String>,
}

/// Resolve `checks(task) = explícitos ∪ globais ∪ por_âncora` (D54).
#[must_use]
pub fn resolve_checks(
    catalog: &ValidatorCatalog,
    explicit: &[String],
    anchors: &[String],
) -> ResolvedChecks {
    let mut sources: BTreeMap<String, CheckSource> = BTreeMap::new();
    let mut missing = Vec::new();
    for name in explicit {
        if catalog.get(name).is_some() {
            sources.entry(name.clone()).or_insert(CheckSource::Explicit);
        } else {
            missing.push(name.clone());
        }
    }
    for name in catalog.globals() {
        if catalog.get(name).is_some() {
            sources.entry(name.clone()).or_insert(CheckSource::Global);
        }
    }
    for (name, validator) in catalog.iter() {
        if sources.contains_key(name) {
            continue;
        }
        if validator.applies_to(anchors) {
            sources.insert(name.to_string(), CheckSource::Anchor);
        }
    }
    missing.sort();
    missing.dedup();
    let checks = sources
        .into_iter()
        .map(|(name, source)| {
            let severity = catalog.get(&name).map_or(Severity::Error, |v| v.severity);
            ResolvedCheck {
                name,
                source,
                severity,
            }
        })
        .collect();
    ResolvedChecks { checks, missing }
}
