//! Recap derivada da memória de agente (atividade recente).

use std::collections::BTreeSet;

use crate::cli::AiArgs;
use knudge_core::Result;
use knudge_core::corpus::Corpus;
use knudge_core::schema::NoteType;
use knudge_core::schema::provenance;
use knudge_core::store::Note;
use serde_json::json;

/// Agente "atual": o com maior `created_at` (empate → lexicograficamente menor).
pub(super) fn current_agent(corpus: &Corpus) -> Option<String> {
    corpus
        .notes
        .iter()
        .max_by(|a, b| {
            let ca = a.frontmatter.created_at().unwrap_or(0);
            let cb = b.frontmatter.created_at().unwrap_or(0);
            ca.cmp(&cb)
                .then_with(|| effective_agent_id(b).cmp(&effective_agent_id(a)))
        })
        .map(effective_agent_id)
}

fn effective_agent_id(note: &Note) -> String {
    provenance(&note.frontmatter)
        .ok()
        .and_then(|p| p.agent_id.clone().or(p.agent))
        .unwrap_or_else(|| "legacy".to_string())
}

/// Agentes disponíveis no corpus (ordenados).
pub(super) fn available_agents(corpus: &Corpus) -> Vec<String> {
    let mut set = BTreeSet::new();
    for note in &corpus.notes {
        set.insert(effective_agent_id(note));
    }
    set.into_iter().collect()
}

#[allow(
    clippy::too_many_lines,
    clippy::unnecessary_wraps,
    clippy::format_push_string,
    reason = "recap aggregation"
)]
/// Executa o recap; devolve (texto, json).
pub(super) fn run(args: &AiArgs, corpus: &Corpus) -> Result<(String, serde_json::Value)> {
    let agents = available_agents(corpus);
    // agente selecionado
    let selected = if args.all_agents {
        None
    } else if !args.agent_id.is_empty() {
        Some(args.agent_id.first().cloned().unwrap_or_default())
    } else {
        current_agent(corpus)
    };

    // filtrar notas
    let mut notes: Vec<&Note> = corpus
        .notes
        .iter()
        .filter(|n| {
            if args.all_agents || args.agent_id.is_empty() {
                selected
                    .as_ref()
                    .is_none_or(|s| effective_agent_id(n) == *s)
            } else {
                args.agent_id
                    .iter()
                    .any(|id| id.as_str() == effective_agent_id(n).as_str())
            }
        })
        .collect();
    // since/until
    notes.retain(|n| {
        let c = n.frontmatter.created_at().unwrap_or(0);
        args.since.is_none_or(|s| c >= s) && args.until.is_none_or(|u| c <= u)
    });
    notes.sort_by_key(|n| {
        (
            n.frontmatter.created_at().unwrap_or(0),
            effective_agent_id(n),
            n.frontmatter.id().unwrap_or_default().to_string(),
        )
    });
    let window = args.limit.unwrap_or(30);
    // últimas N
    let start = notes.len().saturating_sub(window);
    let window_notes = notes.get(start..).unwrap_or(&[]);

    // agregação
    let mut last_decision: Option<&Note> = None;
    let mut last_verification: Option<&Note> = None;
    let mut last_error: Option<&Note> = None;
    let mut files: BTreeSet<String> = BTreeSet::new();
    let mut commands: Vec<String> = Vec::new();
    let mut seen_cmds = BTreeSet::new();

    for n in window_notes {
        let stmt = n.frontmatter.statement().unwrap_or_default().to_string();
        let kind = n
            .frontmatter
            .note_type()
            .map(NoteType::as_str)
            .unwrap_or_default();
        if kind == "decision" {
            last_decision = Some(n);
        }
        if kind == "error" {
            last_error = Some(n);
        }
        // verificação heurística
        let lower = stmt.to_lowercase();
        let is_verify = lower.starts_with("verifica")
            || lower.starts_with("teste")
            || lower.starts_with("check")
            || lower.contains("verific")
            || n.frontmatter
                .string_list("tags")
                .ok()
                .is_none_or(|t| t.contains(&"verify"));
        if is_verify {
            last_verification = Some(n);
        }
        // anchors → arquivos
        if let Ok(list) = n.frontmatter.string_list("anchors") {
            for a in list {
                files.insert(a.to_string());
            }
        }
        // anchors do frontmatter
        // (sem API direta; provenance anchors já cobrem)
        // comandos: statement "comando: ..."
        if let Some(rest) = stmt.strip_prefix("comando: ")
            && seen_cmds.insert(rest.to_string())
        {
            commands.push(rest.to_string());
        }
        // também "omp usou" não é comando
    }
    // comandos mais recentes primeiro
    commands.reverse();

    let agent_label = selected.clone().unwrap_or_else(|| "todos".to_string());

    // texto
    let mut text = String::new();
    text.push_str(&format!(
        "[recap] agente={agent_label} ({} entradas, janela={window})\n",
        window_notes.len()
    ));
    text.push_str(&format!(
        "decisão: {}\n",
        last_decision.map_or_else(
            || "nenhuma".to_string(),
            |n| n.frontmatter.statement().unwrap_or_default().to_string()
        )
    ));
    text.push_str(&format!(
        "verificação: {}\n",
        last_verification.map_or_else(
            || "nenhuma".to_string(),
            |n| n.frontmatter.statement().unwrap_or_default().to_string()
        )
    ));
    text.push_str(&format!(
        "erro: {}\n",
        last_error.map_or_else(
            || "nenhum".to_string(),
            |n| n.frontmatter.statement().unwrap_or_default().to_string()
        )
    ));
    text.push_str(&format!(
        "arquivos: {}\n",
        if files.is_empty() {
            "nenhum".to_string()
        } else {
            files.iter().cloned().collect::<Vec<_>>().join(", ")
        }
    ));
    text.push_str(&format!("comandos: {}\n", commands.join(", ")));
    // agentes disponíveis
    text.push_str("outros agentes: ");
    let others: Vec<String> = agents
        .iter()
        .filter(|a| Some((*a).clone()) != selected)
        .cloned()
        .collect();
    if others.is_empty() {
        text.push_str("nenhum\n");
    } else {
        text.push_str(
            &others
                .iter()
                .map(|a| format!("kd ai session-log --agent-id {a} --recap"))
                .collect::<Vec<_>>()
                .join(" | "),
        );
        text.push('\n');
    }

    let json = json!({
        "agent_id": selected,
        "window": window_notes.len(),
        "last_decision": last_decision.map(note_json),
        "last_verification": last_verification.map(note_json),
        "last_error": last_error.map(note_json),
        "files_touched": files.iter().collect::<Vec<_>>(),
        "commands": commands,
        "agents_available": agents,
    });

    Ok((text, json))
}

fn note_json(note: &Note) -> serde_json::Value {
    json!({
        "id": note.frontmatter.id().unwrap_or_default(),
        "statement": note.frontmatter.statement().unwrap_or_default(),
        "created_at": note.frontmatter.created_at().unwrap_or(0),
    })
}
