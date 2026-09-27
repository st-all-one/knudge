//! Testes do stemmer PT conservador (D206/E16-T11).

use std::borrow::Cow;

use crate::retrieval::stem::{content_terms_stemmed, stem};

/// Radicais dos termos de conteúdo de `text` (público: dobra antes de cortar).
fn stemmed(text: &str) -> Vec<String> {
    content_terms_stemmed(text)
        .into_iter()
        .map(Cow::into_owned)
        .collect()
}

#[test]
fn plural_and_singular_share_a_stem() {
    // `stem` opera sobre o token já dobrado (D172); o fold é feito por `content_terms_stemmed`.
    for (plural, singular) in [
        ("consultas", "consulta"),
        ("indices", "indice"),
        ("arvores", "arvore"),
        ("configuracoes", "configuracao"),
        ("sessoes", "sessao"),
        ("retencoes", "retencao"),
        ("decoders", "decoder"),
    ] {
        assert_eq!(stem(plural), stem(singular), "{plural} vs {singular}");
    }
}

#[test]
fn accented_forms_fold_before_stemming() {
    assert_eq!(stemmed("configurações"), stemmed("configuração"));
    assert_eq!(stemmed("configuração"), vec!["configur"]);
}

#[test]
fn short_words_are_left_alone() {
    for word in ["gas", "pais", "mes", "casa"] {
        assert_eq!(stem(word).as_ref(), word);
    }
}

#[test]
fn stem_is_idempotent() {
    for word in [
        "configuracao",
        "configuracoes",
        "analise",
        "decoder",
        "casa",
    ] {
        let once = stem(word);
        let twice = stem(&once);
        assert_eq!(once, twice, "{word}");
    }
}

#[test]
fn content_terms_are_stemmed_and_filtered() {
    assert_eq!(
        stemmed("a configuração de índices"),
        vec!["configur", "indice"]
    );
}
