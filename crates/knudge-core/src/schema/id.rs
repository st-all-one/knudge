//! IDs endereçados por conteúdo (D01–D03, D95).
//!
//! Formato fixo `<prefixo>_<base36(8)>`. O prefixo acompanha o `type` e é **histórico**:
//! reclassificar o tipo **não** reescreve o `id` (D02).

use crate::schema::body::normalize;
use crate::schema::hash;
use crate::schema::types::NoteType;

/// Separador de campo (US, `0x1F`) entre `type` e `statement` na chave do hash.
pub const ID_SEPARATOR: char = '\u{1f}';

/// Prefixos históricos aceitos em `id` mas que **não** são mais `type` (D149): notas antigas
/// `container_*` permanecem endereçáveis (o prefixo é histórico — D02/D95).
pub const HISTORICAL_PREFIXES: [&str; 1] = ["container"];

/// Gera o `id` de uma nota a partir de `type` + `statement` normalizado (D01).
#[must_use]
pub fn note_id(note_type: NoteType, statement: &str) -> String {
    let normalized = normalize(statement);
    let mut separator = [0_u8; 4];
    let separator = ID_SEPARATOR.encode_utf8(&mut separator);
    let hash = hash::short_hash_parts(&[
        note_type.as_str().as_bytes(),
        separator.as_bytes(),
        normalized.as_bytes(),
    ]);
    let prefix = note_type.prefix();
    let digits = hash::base36_8(hash);
    let mut out =
        String::with_capacity(prefix.len().saturating_add(1).saturating_add(digits.len()));
    out.push_str(prefix);
    out.push('_');
    out.push_str(&digits);
    out
}

/// Valida o formato `<prefixo>_<base36(8)>` e se o prefixo é de um [`NoteType`] conhecido.
#[must_use]
pub fn is_valid_note_id(id: &str) -> bool {
    let Some((prefix, suffix)) = id.split_once('_') else {
        return false;
    };
    suffix.len() == 8
        && suffix
            .bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_lowercase())
        && (NoteType::ALL
            .iter()
            .any(|note_type| note_type.prefix() == prefix)
            || prefix == NoteType::Epic.prefix()
            || HISTORICAL_PREFIXES.contains(&prefix))
}

/// Diretório canônico (`notas/<dir>/`) de um `id` — derivado do prefixo (D150).
///
/// `container_*` (prefixo histórico, D149) vive em `epic/`. Ids sem prefixo conhecido caem no
/// próprio prefixo (tolerante).
#[must_use]
pub fn type_dir(id: &str) -> &str {
    let prefix = id.split_once('_').map_or(id, |(prefix, _)| prefix);
    if prefix == "container" {
        NoteType::Epic.as_str()
    } else {
        prefix
    }
}
