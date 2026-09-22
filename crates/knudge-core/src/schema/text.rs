//! Contagem de `statement` em escalares Unicode (D08).

use crate::{Error, Result};

/// Limite canônico de `statement` (D08).
pub const STATEMENT_MAX: usize = 120;

/// Conta **escalares Unicode** — nem bytes, nem unidades UTF-16, nem grafemas.
#[must_use]
pub fn count_scalars(input: &str) -> usize {
    input.chars().count()
}

/// Valida o limite de [`STATEMENT_MAX`] escalares.
pub fn validate_statement(statement: &str) -> Result<()> {
    let count = count_scalars(statement);
    if count > STATEMENT_MAX {
        return Err(Error::schema(format!(
            "statement tem {count} escalares (máx. {STATEMENT_MAX})"
        )));
    }
    Ok(())
}
