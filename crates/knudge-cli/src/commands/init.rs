//! `kd init` — funda `.knudge/` e emite o prompt inicial (E12-T06, D57/D60/D165).

use std::fmt::Write as _;

use knudge_core::Result;
use knudge_core::git::{LAYOUT_DIRS, OnboardOptions, OnboardReport, onboard};
use serde_json::json;

use crate::cli::InitArgs;
use crate::commands::prime;
use crate::output::Output;
use crate::session::Session;

/// Estado de um item criado/alterado pelo `init` (evita parâmetro `bool`).
#[derive(Debug, Clone, Copy)]
enum State {
    /// Não existia antes.
    New,
    /// Existia e foi atualizado.
    Updated,
    /// Já estava em dia.
    Unchanged,
}

impl State {
    /// Rótulo canônico.
    const fn label(self) -> &'static str {
        match self {
            Self::New => "novo",
            Self::Updated => "atualizado",
            Self::Unchanged => "inalterado",
        }
    }

    /// `novo`/`já existia` para diretórios.
    const fn dir(self) -> &'static str {
        match self {
            Self::New => "novo",
            Self::Updated | Self::Unchanged => "já existia",
        }
    }
}

/// O que o `init` fez (snapshot antes + relatório depois).
struct Plan {
    /// Diretórios do layout, com estado inicial.
    layout: Vec<(&'static str, State)>,
    /// Config do projeto.
    config: State,
    /// `AGENTS.md`.
    agents: State,
    /// `.agents/skill/kd/SKILL.md`.
    skill: State,
}

/// Executa `kd init` (idempotente; `--force` sobrescreve a config).
///
/// # Errors
/// Propaga erros de resolução de projeto, I/O e config.
pub fn run(session: &Session, args: &InitArgs) -> Result<Output> {
    let layout = layout(session);
    let config_existed = session.fs_dyn().exists(&session.project().config_path());
    let root = session.project_root();
    let agents_new = !session.fs_dyn().exists(&root.join("AGENTS.md"));
    let skill_new = !session
        .fs_dyn()
        .exists(&root.join(".agents/skill/kd/SKILL.md"));

    let report = onboard(
        session.fs_dyn(),
        session.git(),
        session.env(),
        OnboardOptions { force: args.force },
    )?;
    tracing::info!(
        project = %report.name,
        dir = %report.knowledge_dir.display(),
        "projeto fundado"
    );

    let prompt = if args.no_prompt {
        String::new()
    } else {
        prime::run(prime::PrimeFormat::Compact).text
    };
    let config = if !report.config_written {
        State::Unchanged
    } else if config_existed {
        State::Updated
    } else {
        State::New
    };
    let agents = if !report.agents_changed {
        State::Unchanged
    } else if agents_new {
        State::New
    } else {
        State::Updated
    };
    let skill = if !report.skill_changed {
        State::Unchanged
    } else if skill_new {
        State::New
    } else {
        State::Updated
    };
    let plan = Plan {
        layout,
        config,
        agents,
        skill,
    };
    let text = render(&report, &plan, &prompt);
    Ok(Output::new(text, report_data(&report)))
}

/// Estado inicial dos diretórios do layout (antes do `onboard`).
fn layout(session: &Session) -> Vec<(&'static str, State)> {
    let knowledge_dir = session.knowledge_dir();
    LAYOUT_DIRS
        .iter()
        .map(|dir| {
            let state = if session.fs_dyn().exists(&knowledge_dir.join(dir)) {
                State::Unchanged
            } else {
                State::New
            };
            (*dir, state)
        })
        .collect()
}

/// Texto explícito: o que foi criado/alterado e os próximos passos (D165).
fn render(report: &OnboardReport, plan: &Plan, prompt: &str) -> String {
    let structure = plan
        .layout
        .iter()
        .map(|(dir, state)| format!("{dir}/ ({})", state.dir()))
        .collect::<Vec<_>>()
        .join(", ");
    let mut text = String::new();
    let _ignored = writeln!(
        text,
        "projeto {} fundado em {}",
        report.name,
        report.knowledge_dir.display()
    );
    let _ignored = writeln!(text, "estrutura: {structure}");
    let _ignored = writeln!(text, "config.toml ({})", plan.config.label());
    if report.in_repo {
        let exclude = if report.exclude_changed {
            State::Updated
        } else {
            State::Unchanged
        };
        let attributes = if report.attributes_changed {
            State::Updated
        } else {
            State::Unchanged
        };
        let _ignored = writeln!(text, ".git/info/exclude ({})", exclude.label());
        let _ignored = writeln!(text, ".gitattributes ({})", attributes.label());
    }
    let _ignored = writeln!(text, "AGENTS.md ({})", plan.agents.label());
    let _ignored = writeln!(text, ".agents/skill/kd/SKILL.md ({})", plan.skill.label());
    text.push_str("próximos: kd write \"<afirmação>\" --type fact --anchor <PATH>\n");
    text.push_str("          kd prime   # protocolo completo com --long\n");
    if !prompt.is_empty() {
        text.push('\n');
        text.push_str(prompt);
    }
    text
}

/// Envelope de dados do `init`.
fn report_data(report: &OnboardReport) -> serde_json::Value {
    json!({
        "project": report.name,
        "root": report.root,
        "knowledge_dir": report.knowledge_dir,
        "in_repo": report.in_repo,
        "config_written": report.config_written,
        "exclude_changed": report.exclude_changed,
        "attributes_changed": report.attributes_changed,
        "agents_changed": report.agents_changed,
        "skill_changed": report.skill_changed,
    })
}
