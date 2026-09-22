//! IDs endereçados por conteúdo (D01–D03, D95).
//!
//! Formato fixo `<prefixo>_<base36(8)>`. O prefixo acompanha o `type` e é **histórico**:
//! reclassificar o tipo **não** reescreve o `id` (D02).

use crate::schema::body::normalize;
use crate::schema::hash;
use crate::schema::types::NoteType;

/// Separador de campo (US, `0x1F`) entre `type` e `statement` na chave do hash.
pub const ID_SEPARATOR: char = '\u{1f}';

/// Gera o `id` de uma nota a partir de `type` + `statement` normalizado (D01).
#[must_use]
pub fn note_id(note_type: NoteType, statement: &str) -> String {
    let mut data = note_type.as_str().to_string();
    data.push(ID_SEPARATOR);
    data.push_str(&normalize(statement));
    format!(
        "{}_{}",
        note_type.prefix(),
        hash::base36_8(hash::short_hash(data.as_bytes()))
    )
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
        && NoteType::ALL
            .iter()
            .any(|note_type| note_type.prefix() == prefix)
}
