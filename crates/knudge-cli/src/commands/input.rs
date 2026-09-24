//! Leitura de conteúdo por posicional/stdin (D140/D147).

use std::io::{IsTerminal, Read};

use knudge_core::{Error, Result};

/// Conteúdo de um argumento posicional.
///
/// - `["-"]` → lê stdin;
/// - `[]` → stdin não-TTY: lê; TTY: erro (`invalid_input`);
/// - demais → junta com espaço.
///
/// # Errors
/// `ErrorKind::InvalidInput` quando falta conteúdo e o stdin é um TTY; propaga I/O do stdin.
pub fn content(parts: &[String]) -> Result<String> {
    if is_stdin(parts) {
        return read_stdin();
    }
    if parts.is_empty() {
        if std::io::stdin().is_terminal() {
            return Err(Error::invalid_input(
                "sem conteúdo: passe o texto, use `-` ou um pipe",
            ));
        }
        return read_stdin();
    }
    Ok(parts.join(" "))
}

/// `true` se o posicional é exatamente `-`.
#[must_use]
pub fn is_stdin(parts: &[String]) -> bool {
    parts.len() == 1 && parts.first().is_some_and(|part| part == "-")
}

/// Lê o stdin inteiro como UTF-8.
///
/// # Errors
/// Propaga erros de I/O do stdin.
pub fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|error| Error::io("stdin", error))?;
    Ok(buffer)
}
