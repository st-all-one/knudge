//! Escopo `knowledge`: promoção de conhecimento a regras governadas (D157).

pub mod promote;

#[cfg(test)]
mod tests;

pub use promote::{
    Candidate, RULES_BEGIN, RULES_END, RULES_HEADING, RULES_VERSION, RulesPolicy, apply_block,
    parse_block, parse_entries, recommend, render_block,
};
