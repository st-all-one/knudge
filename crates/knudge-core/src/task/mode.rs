//! Modo de execução derivado do grafo (D116/D136).
//!
//! O modo (`sequential`/`concurrent`/`magentic`) é uma **função pura** da projeção do
//! container: filhos (com `depends_on`) e um sinal derivado do log (`incremental` = filhos
//! criados em momentos distintos). Nada é armazenado.

use std::collections::BTreeSet;

/// Modo de execução de um container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Filhos em cadeia (`depends_on`/ordem estrita).
    Sequential,
    /// Filhos independentes entre si.
    Concurrent,
    /// Filhos criados incrementalmente (planejamento dinâmico).
    Magentic,
}

impl Mode {
    /// Rótulo canônico (exibição; nunca armazenado).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::Concurrent => "concurrent",
            Self::Magentic => "magentic",
        }
    }
}

/// Filho projetado para o classificador (`children` em ordem de `id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Child {
    /// Id do filho.
    pub id: String,
    /// `depends_on` entre irmãos.
    pub depends_on: Vec<String>,
}

/// Container projetado para o classificador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    /// Filhos em ordem de `id`.
    pub children: Vec<Child>,
    /// `true` se os filhos foram criados em momentos distintos.
    pub incremental: bool,
}

/// Classifica o modo. Precedência: `magentic` > `sequential` > `concurrent`.
#[must_use]
pub fn mode(container: &Container) -> Mode {
    if container.incremental {
        return Mode::Magentic;
    }
    if sequential(container) {
        return Mode::Sequential;
    }
    Mode::Concurrent
}

/// `true` se todo filho (exceto o primeiro, por `id`) depende de um irmão anterior.
fn sequential(container: &Container) -> bool {
    let mut iter = container.children.iter();
    let Some(first) = iter.next() else {
        return true;
    };
    if container.children.len() < 2 {
        return true;
    }
    let mut earlier: BTreeSet<&str> = BTreeSet::new();
    let _ignored = earlier.insert(first.id.as_str());
    for child in iter {
        let depends_on_earlier = child
            .depends_on
            .iter()
            .any(|dep| earlier.contains(dep.as_str()));
        if !depends_on_earlier {
            return false;
        }
        let _ignored = earlier.insert(child.id.as_str());
    }
    true
}
