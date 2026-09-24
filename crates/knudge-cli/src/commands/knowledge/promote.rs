//! `kd knowledge promote` — regras governadas no `AGENTS.md` (D157).
//!
//! O agente/humano decide: `recommend`/`list` são read-only; `approve`/`edit`/`remove` escrevem
//! o bloco governado e **nunca** tocam a nota de origem. O teto (`rules.max_promoted`) é
//! admission control: sem espaço, o comando recusa e nomeia quem sai.

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::git::block::read_text;
use knudge_core::knowledge::{Candidate, RulesPolicy, apply_block, parse_entries, recommend};
use serde_json::json;

use crate::cli::{PromoteCommand, PromoteEditArgs, PromoteRecommendArgs, PromoteTargetArgs};
use crate::output::Output;
use crate::session::Session;

/// Sentinela de resultado vazio (D152).
const NO_RESULTS: &str = "[no_results]";

/// Filtro de corpus da promoção (evita `bool` em parâmetro).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeFilter {
    /// Só conhecimento (sem `scope`).
    Knowledge,
    /// Projeto inteiro.
    Universe,
}

/// Executa `kd knowledge promote <subcomando>`.
///
/// # Errors
/// Propaga erros do domínio e `invalid_input`/`not_found`/`conflict` de promoção.
pub fn run(session: &Session, command: &PromoteCommand) -> Result<Output> {
    let policy = RulesPolicy::from_config(session.config());
    match command {
        PromoteCommand::Recommend(args) => recommend_cmd(session, &policy, args),
        PromoteCommand::Approve(args) => approve(session, &policy, args, None),
        PromoteCommand::Edit(args) => edit(session, &policy, args),
        PromoteCommand::Remove(args) => remove(session, &policy, args),
        PromoteCommand::List => list(session, &policy),
    }
}

fn ensure_enabled(policy: &RulesPolicy) -> Result<()> {
    if policy.enabled {
        Ok(())
    } else {
        Err(Error::invalid_input(
            "promoção desligada: defina `rules.enabled = true` na config",
        ))
    }
}

/// Carrega as candidatas (aplica o filtro de trabalho, salvo `--universe`).
fn candidates(
    session: &Session,
    policy: &RulesPolicy,
    scope: ScopeFilter,
) -> Result<Vec<Candidate>> {
    let store = session.store();
    let mut notes = Vec::new();
    for id in store.list_ids()? {
        if let Some(note) = store.read_optional(&id)? {
            notes.push(note);
        }
    }
    if scope == ScopeFilter::Knowledge {
        notes.retain(|note| note.frontmatter.scope().ok().flatten().is_none());
    }
    let graph = session.graph()?;
    recommend(&notes, &graph, policy, session.now_ms())
}

fn recommend_cmd(
    session: &Session,
    policy: &RulesPolicy,
    args: &PromoteRecommendArgs,
) -> Result<Output> {
    ensure_enabled(policy)?;
    let scope = if args.universe {
        ScopeFilter::Universe
    } else {
        ScopeFilter::Knowledge
    };
    let mut found = candidates(session, policy, scope)?;
    if let Some(limit) = args.limit {
        found.truncate(limit);
    }
    let text = if found.is_empty() {
        NO_RESULTS.to_string()
    } else {
        found
            .iter()
            .map(|candidate| {
                format!(
                    "{}|{:.2}|{}",
                    candidate.id, candidate.confidence, candidate.statement
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let data = json!({
        "candidates": found.iter().map(|candidate| json!({
            "id": candidate.id,
            "statement": candidate.statement,
            "confidence": candidate.confidence,
            "reason": candidate.reason,
        })).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}

fn approve(
    session: &Session,
    policy: &RulesPolicy,
    args: &PromoteTargetArgs,
    edit: Option<&str>,
) -> Result<Output> {
    ensure_enabled(policy)?;
    let root = session.project_root().to_path_buf();
    let path = root.join("AGENTS.md");
    let text = read_text(session.fs_dyn(), &path)?;
    let existing = parse_entries(&text);
    let scope = if args.universe {
        ScopeFilter::Universe
    } else {
        ScopeFilter::Knowledge
    };
    let recommendations = candidates(session, policy, scope)?;
    let is_present = existing.iter().any(|(id, _)| id == &args.id);
    let candidate = recommendations
        .iter()
        .find(|candidate| candidate.id == args.id)
        .cloned();
    if !is_present && candidate.is_none() {
        return Err(Error::not_found(format!(
            "candidata ausente (rode `kd knowledge promote recommend`): {}",
            args.id
        )));
    }
    if !is_present && existing.len() >= policy.max_promoted {
        let evict = existing.last().map_or("—", |(id, _)| id.as_str());
        return Err(Error::conflict(format!(
            "bloco cheio ({} regras); remova antes de admitir (sugestão: {evict})",
            policy.max_promoted
        )));
    }
    let mut approved = Vec::with_capacity(existing.len().saturating_add(1));
    for (id, statement) in &existing {
        let statement = if id == &args.id && edit.is_some() {
            edit.unwrap_or_default().to_string()
        } else {
            statement.clone()
        };
        approved.push(Candidate {
            id: id.clone(),
            statement,
            confidence: 0.0,
            reason: String::new(),
        });
    }
    if !is_present && let Some(candidate) = candidate {
        approved.push(candidate);
    }
    let changed = apply_block(session.fs_dyn(), &root, &approved)?;
    Ok(promotion_output(
        if changed { "updated" } else { "unchanged" },
        &approved,
    ))
}

fn edit(session: &Session, policy: &RulesPolicy, args: &PromoteEditArgs) -> Result<Output> {
    let target = PromoteTargetArgs {
        id: args.id.clone(),
        universe: args.universe,
    };
    approve(session, policy, &target, Some(&args.summary))
}

fn remove(session: &Session, policy: &RulesPolicy, args: &PromoteTargetArgs) -> Result<Output> {
    ensure_enabled(policy)?;
    let root = session.project_root().to_path_buf();
    let path = root.join("AGENTS.md");
    let text = read_text(session.fs_dyn(), &path)?;
    let existing = parse_entries(&text);
    if !existing.iter().any(|(id, _)| id == &args.id) {
        return Err(Error::not_found(format!(
            "nota não está promovida: {}",
            args.id
        )));
    }
    let approved: Vec<Candidate> = existing
        .iter()
        .filter(|(id, _)| id != &args.id)
        .map(|(id, statement)| Candidate {
            id: id.clone(),
            statement: statement.clone(),
            confidence: 0.0,
            reason: String::new(),
        })
        .collect();
    let changed = apply_block(session.fs_dyn(), &root, &approved)?;
    Ok(promotion_output(
        if changed { "updated" } else { "unchanged" },
        &approved,
    ))
}

fn list(session: &Session, policy: &RulesPolicy) -> Result<Output> {
    let root = session.project_root().to_path_buf();
    let text = read_text(session.fs_dyn(), &root.join("AGENTS.md"))?;
    let entries = parse_entries(&text);
    let text_out = if entries.is_empty() {
        NO_RESULTS.to_string()
    } else {
        entries
            .iter()
            .map(|(id, statement)| format!("{id}|{statement}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let data = json!({
        "promoted": entries.iter().map(|(id, statement)| json!({
            "id": id, "statement": statement,
        })).collect::<Vec<_>>(),
        "count": entries.len(),
        "max": policy.max_promoted,
        "enabled": policy.enabled,
    });
    Ok(Output::new(text_out, data))
}

fn promotion_output(action: &str, approved: &[Candidate]) -> Output {
    let text = approved
        .iter()
        .map(|candidate| format!("{}|{}", candidate.id, candidate.statement))
        .collect::<Vec<_>>()
        .join("\n");
    let text = if text.is_empty() {
        NO_RESULTS.to_string()
    } else {
        text
    };
    let data = json!({
        "action": action,
        "promoted": approved.iter().map(|candidate| json!({
            "id": candidate.id, "statement": candidate.statement,
        })).collect::<Vec<_>>(),
    });
    Output::new(text, data)
}
