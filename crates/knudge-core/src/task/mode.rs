//! Modo de execução derivado do grafo (D116).
//!
//! O modo (`sequential`/`concurrent`/`supervisor`/`handoff`/`magentic`) é uma **função pura** da
//! projeção do container: dono, filhos (com dono e `depends_on`) e dois sinais derivados do log
//! (`handoff` = troca de dono; `incremental` = filhos criados em momentos distintos). Nada é
//! armazenado.

use std::collections::BTreeSet;

/// Modo de execução de um container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Filhos em cadeia (`depends_on`/ordem estrita).
    Sequential,
    /// Filhos independentes entre si.
    Concurrent,
    /// Container com dono e ≥2 filhos com donos distintos.
    Supervisor,
    /// Algum nó trocou de dono (≥2 `claim`).
    Handoff,
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
            Self::Supervisor => "supervisor",
            Self::Handoff => "handoff",
            Self::Magentic => "magentic",
        }
    }
}

/// Filho projetado para o classificador (`children` em ordem de `id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Child {
    /// Id do filho.
    pub id: String,
    /// Dono derivado de `claim`/`release`.
    pub owner: Option<String>,
    /// `depends_on` entre irmãos.
    pub depends_on: Vec<String>,
}

/// Container projetado para o classificador.
#[allow(clippy::struct_excessive_bools, reason = "sinais derivados do log")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    /// Dono do próprio container.
    pub owner: Option<String>,
    /// Filhos em ordem de `id`.
    pub children: Vec<Child>,
    /// `true` se algum nó da subárvore teve ≥2 `claim` (troca de dono).
    pub handoff: bool,
    /// `true` se os filhos foram criados em momentos distintos.
    pub incremental: bool,
}

/// Classifica o modo. Precedência: `handoff` > `supervisor` > `magentic` > `sequential` >
/// `concurrent`.
#[must_use]
pub fn mode(container: &Container) -> Mode {
    if container.handoff {
        return Mode::Handoff;
    }
    if supervisor(container) {
        return Mode::Supervisor;
    }
    if container.incremental {
        return Mode::Magentic;
    }
    if sequential(container) {
        return Mode::Sequential;
    }
    Mode::Concurrent
}

fn supervisor(container: &Container) -> bool {
    if container.owner.is_none() {
        return false;
    }
    let mut owners: BTreeSet<&str> = BTreeSet::new();
    for child in &container.children {
        if let Some(owner) = &child.owner {
            let _ignored = owners.insert(owner.as_str());
        }
    }
    owners.len() >= 2
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
