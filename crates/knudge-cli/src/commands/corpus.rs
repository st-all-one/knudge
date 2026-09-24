//! Escopo de corpus compartilhado (D143/D144/D146): filtros estruturais + vizinhança.
//!
//! O princípio "nada de operação sem escopo" (D143 §2.1) vale para todo verbo que varre o
//! corpus: `knowledge map`/`rank`, `rewind` e `learn`/`compact`/`prune`/`task list`. Aqui vive
//! a fonte de verdade do predicado (`--tag`/`--anchor`/`--type`/`--class`/`--around`/`--universe`).

use std::collections::{BTreeMap, BTreeSet};

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::graph::Graph;
use knudge_core::retrieval::{Filter, Index, Meta};

use crate::cli::CorpusArgs;

use super::parse;

/// Filtros de corpus pedidos na linha de comando.
#[derive(Debug, Clone, Default)]
pub struct CorpusScope {
    /// Tipos aceitos (`--type`).
    pub types: Vec<String>,
    /// Classificações aceitas (`--class`).
    pub classes: Vec<String>,
    /// Tags exigidas (`--tag`; basta uma).
    pub tags: Vec<String>,
    /// Âncoras (path/glob) exigidas (`--anchor`; basta uma).
    pub anchors: Vec<String>,
    /// Nota central da vizinhança (`--around`).
    pub around: Option<String>,
    /// Profundidade da vizinhança (`--depth`).
    pub depth: u8,
    /// Varredura explícita do projeto inteiro (`--universe`).
    pub universe: bool,
}

impl CorpusScope {
    /// `true` se algum filtro estrutural/vizinhança foi pedido.
    #[must_use]
    pub fn has_filters(&self) -> bool {
        !self.types.is_empty()
            || !self.classes.is_empty()
            || !self.tags.is_empty()
            || !self.anchors.is_empty()
            || self.around.is_some()
    }

    /// `true` se há escopo explícito (filtro ou `--universe`).
    #[must_use]
    pub fn is_scoped(&self) -> bool {
        self.has_filters() || self.universe
    }

    /// Erro `invalid_input` (2) quando não há escopo nem `--universe` (D143/D144).
    ///
    /// # Errors
    /// `ErrorKind::InvalidInput` com a orientação do `command`.
    pub fn require(&self, command: &str) -> Result<()> {
        if self.is_scoped() {
            Ok(())
        } else {
            Err(Error::invalid_input(format!(
                "`{command}` exige escopo: use --tag/--anchor/--type/--class/--around ou --universe"
            )))
        }
    }

    /// Resolve o escopo contra o índice/grafo (filtro + vizinhança + metadados por id).
    ///
    /// # Errors
    /// `not_found` (3) se a nota de `--around` não existir; propaga tipo/classe inválidos.
    pub fn select<'a>(&self, index: &'a Index, graph: &Graph) -> Result<Selection<'a>> {
        let filter = Filter {
            types: parse::types(&self.types)?,
            classifications: parse::classifications(&self.classes)?,
            tags: self.tags.clone(),
            anchors: self.anchors.clone(),
            ..Filter::new()
        };
        let allowed = self.allowed(graph)?;
        let metas = index
            .docs
            .iter()
            .map(|doc| (doc.meta.id.as_str(), &doc.meta))
            .collect();
        Ok(Selection {
            filter,
            allowed,
            metas,
        })
    }

    /// Ids da vizinhança de `--around` (inclui o centro); `None` sem `--around`.
    ///
    /// # Errors
    /// `not_found` (3) se a nota central não existir.
    fn allowed(&self, graph: &Graph) -> Result<Option<BTreeSet<String>>> {
        let Some(around) = &self.around else {
            return Ok(None);
        };
        if !graph.contains(around) {
            return Err(Error::not_found(format!("nota ausente: {around}")));
        }
        let mut allowed = BTreeSet::new();
        let _ignored = allowed.insert(around.clone());
        for hit in graph.expand(around, None, u32::from(self.depth)) {
            let _ignored = allowed.insert(hit.id);
        }
        Ok(Some(allowed))
    }
}

impl From<&CorpusArgs> for CorpusScope {
    fn from(args: &CorpusArgs) -> Self {
        Self {
            types: args.types.clone(),
            classes: args.classes.clone(),
            tags: args.tags.clone(),
            anchors: args.anchor.clone(),
            around: args.around.clone(),
            depth: args.depth,
            universe: args.universe,
        }
    }
}

/// Corpus já resolvido: filtro estrutural, vizinhança e metadados por id.
pub struct Selection<'a> {
    filter: Filter,
    allowed: Option<BTreeSet<String>>,
    metas: BTreeMap<&'a str, &'a Meta>,
}

impl Selection<'_> {
    /// Filtro estrutural (para `structural_clusters_filtered`/`recall`).
    #[must_use]
    pub fn filter(&self) -> &Filter {
        &self.filter
    }

    /// Ids permitidos pela vizinhança; `None` = tudo.
    #[must_use]
    pub fn allowed(&self) -> Option<&BTreeSet<String>> {
        self.allowed.as_ref()
    }

    /// `true` se a nota (por metadados) entra no escopo.
    #[must_use]
    pub fn matches(&self, meta: &Meta) -> bool {
        self.filter.matches(meta)
            && self
                .allowed
                .as_ref()
                .is_none_or(|ids| ids.contains(&meta.id))
    }

    /// `true` se a nota (por id) entra no escopo.
    #[must_use]
    pub fn matches_id(&self, id: &str) -> bool {
        self.metas.get(id).is_some_and(|meta| self.matches(meta))
    }

    /// Quantidade de notas no escopo.
    #[must_use]
    pub fn docs(&self) -> usize {
        self.metas
            .values()
            .filter(|meta| self.matches(meta))
            .count()
    }
}
