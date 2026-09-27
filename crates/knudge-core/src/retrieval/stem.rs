//! Stemmer PT **conservador** (D206/E16-T11).
//!
//! Corta sufixos flexionais/derivacionais comuns do PT-BR sobre o token já **dobrado** (D172),
//! exigindo radical mínimo de 4 bytes — não over-stemma palavras curtas. O plural é normalizado
//! **antes** do corte derivacional, para `decoder`/`decoders` convergirem. Aplicado **só** ao canal
//! lexical (índice + consulta); o `normalize` do schema (`id`/`body_hash`, D06/D95) **não muda** e
//! o snippet usa os termos crus (`content_terms`), para casar o texto exibido.
//!
//! Adotado por medição (E16/T11): +25 % de nDCG@5 na bancada de qualidade (`bench/t11_stemming.md`),
//! com a família `morfologia` (consulta plural × nota singular) de 0 % para 100 %. Muda só o
//! derivado `.idx/` (`INDEX_FORMAT` → `retrieval-v4`), nunca os bytes de `notas/`.

use std::borrow::Cow;

use crate::retrieval::token::content_terms;

/// Radical mínimo (em bytes) preservado após o corte.
const MIN_STEM: usize = 4;

/// Sufixos derivacionais, do mais longo/menos ambíguo para o mais curto (o primeiro que casa
/// vence). Agrupados pelo **último byte** em [`suffixes_ending_with`] para cortar o custo por token.
const SUFFIXES_O: &[&str] = &[
    "imento", "amento", "icao", "ucao", "acao", "ismo", "eiro", "ando", "endo", "indo", "oso",
    "ado", "ido",
];
const SUFFIXES_S: &[&str] = &[
    "amentos", "imentos", "encias", "ancias", "acoes", "ucoes", "icoes", "dades", "ismos", "istas",
    "eiros", "eiras", "aveis", "iveis", "osos", "osas", "ados", "idos", "adas", "idas", "amos",
    "emos", "imos",
];
const SUFFIXES_A: &[&str] = &[
    "encia", "ancia", "ista", "eira", "aria", "eria", "iria", "osa", "ada", "ida",
];
const SUFFIXES_E: &[&str] = &["dade", "asse", "esse", "isse"];
const SUFFIXES_L: &[&str] = &["avel", "ivel"];
const SUFFIXES_M: &[&str] = &["ariam", "eriam", "iriam", "aram", "eram", "iram"];
const SUFFIXES_R: &[&str] = &["ar", "er", "ir"];

/// Grupo de sufixos cujo último byte é `last` (mais longo primeiro).
fn suffixes_ending_with(last: u8) -> &'static [&'static str] {
    match last {
        b'o' => SUFFIXES_O,
        b's' => SUFFIXES_S,
        b'a' => SUFFIXES_A,
        b'e' => SUFFIXES_E,
        b'l' => SUFFIXES_L,
        b'm' => SUFFIXES_M,
        b'r' => SUFFIXES_R,
        _ => &[],
    }
}

/// Radical conservador de `token` (minúsculo, sem acento). Empréstimo quando nada é cortado.
#[must_use]
pub fn stem(token: &str) -> Cow<'_, str> {
    if token.len() < MIN_STEM {
        return Cow::Borrowed(token);
    }
    // 1. Plural de "-ão" → "-ões": normaliza para o singular antes do corte derivacional.
    if let Some(base) = token.strip_suffix("oes") {
        let normalized = format!("{base}ao");
        return Cow::Owned(cut(&normalized).into_owned());
    }
    // 2. Plural geral: `s` final, preservando o radical mínimo.
    let singular = token
        .strip_suffix('s')
        .filter(|base| base.len() >= MIN_STEM)
        .unwrap_or(token);
    // 3. Corte derivacional.
    cut(singular)
}

/// Corta o primeiro sufixo conhecido que preserve [`MIN_STEM`].
fn cut(token: &str) -> Cow<'_, str> {
    let Some(&last) = token.as_bytes().last() else {
        return Cow::Borrowed(token);
    };
    for suffix in suffixes_ending_with(last) {
        if let Some(radical) = token.strip_suffix(suffix)
            && radical.len() >= MIN_STEM
        {
            return Cow::Owned(radical.to_string());
        }
    }
    Cow::Borrowed(token)
}

/// Aplica [`stem`] a termos já tokenizados, preservando o empréstimo quando o radical não muda.
#[must_use]
pub fn stem_terms(terms: Vec<Cow<'_, str>>) -> Vec<Cow<'_, str>> {
    terms
        .into_iter()
        .map(|term| match term {
            Cow::Borrowed(text) => stem(text),
            Cow::Owned(text) => Cow::Owned(stem(&text).into_owned()),
        })
        .collect()
}

/// Termos de **conteúdo** da consulta com radical (D206): a contraparte de `content_terms` usada
/// pelo canal lexical (BM25/dedup). O snippet segue com `content_terms` (termos crus).
#[must_use]
pub fn content_terms_stemmed(input: &str) -> Vec<Cow<'_, str>> {
    stem_terms(content_terms(input))
}
