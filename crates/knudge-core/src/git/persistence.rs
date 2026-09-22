//! Política de persistência do conhecimento no projeto (D34).

/// Onde o conhecimento mora em relação ao git.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Persistence {
    /// `persist_in_project = true`: `notas/`/`eventos/` são versionados; só o derivado é
    /// excluído via `.git/info/exclude`.
    Versioned,
    /// `persist_in_project = false`: o `.knudge/` inteiro é local-only.
    LocalOnly,
}

impl Persistence {
    /// `true` quando `notas/`/`eventos/` acompanham o repositório.
    #[must_use]
    pub const fn is_versioned(self) -> bool {
        matches!(self, Self::Versioned)
    }
}
