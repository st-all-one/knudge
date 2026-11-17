//! Deny-list fixa de caminhos sensíveis (D226, extensão de R22/D91).
//!
//! Âncoras, hints e índices não são log: um caminho sensível não pode entrar neles. A lista é
//! fixa — a config não pode reduzi-la. O casamento é por componente basename (com `/` ou `\` como
//! separadores), nunca por substring: `id_ed25519` é negado, `identity/` é permitido.

use crate::retrieval::anchor::glob_match;

/// Padrões fixos negados; a config não pode reduzir esta lista.
pub const DENIED_PATTERNS: [&str; 12] = [
    ".env*",
    "*.pem",
    "*.key",
    "*.p12",
    "*.jks",
    "*.keystore",
    "id_*",
    "credentials*",
    ".netrc",
    ".pgpass",
    "secrets.*",
    "*.tfstate*",
];

/// `true` se o basename de `path` satisfaz algum padrão negado.
///
/// Globs já resolvidos também caem na lista: `**/*.pem` tem basename `*.pem`.
#[must_use]
pub fn is_denied(path: &str) -> bool {
    let Some(basename) = path.rsplit(['/', '\\']).find(|part| !part.is_empty()) else {
        return false;
    };
    DENIED_PATTERNS
        .iter()
        .any(|pattern| glob_match(pattern, basename))
}
