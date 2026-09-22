//! Codec TOML próprio (subset) para a configuração.

mod emit;
mod parse;

pub use emit::emit;
pub use parse::parse;

#[cfg(test)]
mod tests;
