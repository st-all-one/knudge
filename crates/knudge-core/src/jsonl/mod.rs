//! Escopo `jsonl`: leitura/escrita de JSON Lines com tolerância (E03).
//!
//! Um arquivo `*.jsonl` é uma sequência de valores JSON, um por linha. O knudge usa JSONL para
//! o log de eventos (`eventos/`) e para estruturas derivadas (`.idx/`), sempre com **dedup
//! on-read** (D26) e tolerância a linha malformada (skip + warning).

pub mod json;

#[cfg(test)]
mod tests;

pub use json::{decode, encode};

/// Itera as linhas úteis de uma fonte: sem `\r` final e sem linhas em branco.
pub fn lines(source: &str) -> impl Iterator<Item = &str> {
    source
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter(|line| !line.trim().is_empty())
}
