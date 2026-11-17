//! Testes da deny-list fixa de caminhos sensíveis (D226).

use proptest::prelude::*;

use crate::store::paths::{DENIED_PATTERNS, is_denied};

#[test]
fn fixed_patterns_are_exposed() {
    assert_eq!(DENIED_PATTERNS.len(), 12);
}

#[test]
fn sensitive_basenames_are_denied() {
    for path in [
        ".env",
        ".env.local",
        "server.pem",
        "server.key",
        "server.p12",
        "server.jks",
        "server.keystore",
        "id_ed25519",
        "credentials.json",
        ".netrc",
        ".pgpass",
        "secrets.prod.yaml",
        "*.tfstate",
        "terraform.tfstate.backup",
    ] {
        assert!(is_denied(path), "esperado deny para {path}");
    }
}

#[test]
fn separators_and_resolved_globs_use_basename() {
    assert!(is_denied("src/.env.local"));
    assert!(is_denied("C:\\Users\\dev\\.env.local"));
    assert!(is_denied("**/*.pem"));
    assert!(is_denied("src/**/server.key"));
}

#[test]
fn ordinary_paths_are_allowed() {
    for path in [
        "identity/",
        "README.md",
        "src/main.rs",
        "src/identity/readme.md",
    ] {
        assert!(
            !is_denied(path),
            "caminho comum não deve ser negado: {path}"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// O sufixo não transforma `identity` em `id_*`, mesmo com separadores.
    #[test]
    fn basename_is_matched_not_substring(suffix in 0_u32..1000) {
        let denied_path = format!("src/id_key{suffix}");
        let allowed_path = format!("src/identity{suffix}/");
        prop_assert!(is_denied(&denied_path));
        prop_assert!(!is_denied(&allowed_path));
    }
}
