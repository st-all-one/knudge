//! Help embutido do `kd` (D167/D171): resumo global e renderização por verbo.

use clap::CommandFactory;

use super::Cli;

/// Resumo global exibido no fim do help (D167).
pub(super) const AFTER_HELP: &str = "\
CICLO: kd ask → kd write → kd task → kd sync
ÂNCORAS: --anchor PATH liga nota/tarefa ao código (repetível; vírgula; glob `src/**`)
CORPO: use quando o statement sozinho não permite agir (Por quê / Evidência / Consequência)
veja: kd help <verbo>   # exemplo, escopo e quando NÃO usar";

/// Help completo do `kd` (idêntico a `kd --help`).
#[must_use]
pub fn render_help() -> String {
    Cli::command().render_help().to_string()
}

/// Help longo de um verbo, para erros de uso (D167).
#[must_use]
pub fn subcommand_help(verb: &str) -> String {
    let mut root = Cli::command();
    if let Some(sub) = root.find_subcommand_mut(verb) {
        return sub.render_long_help().to_string();
    }
    root.render_help().to_string()
}
