//! `kd ai` — consulta **read-only** de uma fonte de memória de agente (D218/D220).
//!
//! O knudge **indexa** (derivado); o agente **possui** a fonte. A memória v2 é um diretório de
//! shards NDJSON — um `<agent_id>.jsonl` por escritor (agente principal e subagentes), sem ficheiro
//! compartilhado no caminho quente. Esta área lê a fonte, deriva índice+grafo e responde com o
//! mesmo motor do `ask`; o contrato é o **documento** (o schema do knudge), nunca o schema
//! interno do agente.
//!
//! Não há caminho legado: o default é `.knudge/ai/<area>/`, e um `--source` explícito aceita
//! ficheiro único **ou** diretório de shards (D220).

use std::collections::BTreeMap;

use knudge_core::Result;
use knudge_core::corpus::{Corpus, Source};
use knudge_core::retrieval::{Filter, Meta, RecallHit, RecallQuery, Universe, recall};
use knudge_core::schema::{AgentRole, NoteType, Status, provenance};
use knudge_core::store::Note;
use serde_json::json;

use crate::cli::{AiArgs, AiSort};
use crate::commands::parse;
use crate::output::Output;
use crate::session::Session;

mod params;
mod recap;

use params::resolve_params;

/// Resultado de um recall sobre a fonte: texto (pipe) + JSON + avisos de canal.
type Found = (String, serde_json::Value, Vec<String>);

/// Sentinela de busca vazia (D152), como no `ask`: distingue "sem resultado" de erro.
const NO_RESULTS: &str = "[no_results]";

/// Executa `kd ai`: lista a fonte (sem query) ou faz recall sobre ela (com query).
///
/// # Errors
/// Propaga erros de leitura da fonte e de derivação do índice/grafo; `strict` promove avisos a erro.
pub fn run(session: &Session, args: &AiArgs) -> Result<Output> {
    let args = resolve_params(args)?;
    let source = resolve_source(session, &args);
    let source_label = source
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let (corpus, mut warnings) = source.load(&session.store())?;
    let query = args.query.join(" ");
    if args.recap {
        let (text, recap_json) = recap::run(&args, &corpus)?;
        let data = json!({
            "area": args.area.as_str(),
            "source": source_label,
            "sort": args.sort.as_str(),
            "query": query,
            "data": { "recap": recap_json },
        });
        return Ok(Output::new(text, data).with_warnings(warnings));
    }
    let (text, data) = if query.is_empty() {
        (list(&args, &corpus)?, list_json(&args, &corpus)?)
    } else {
        let (text, data, extra) = search(session, &args, &corpus, &query)?;
        warnings.extend(extra);
        (text, data)
    };
    let data = json!({
        "area": args.area.as_str(),
        "source": source_label,
        "sort": args.sort.as_str(),
        "query": query,
        "data": data,
    });
    Ok(Output::new(text, data).with_warnings(warnings))
}
/// Recap derivada (reusado pelo `rewind --with-activity`).
pub fn run_recap(args: &AiArgs, corpus: &Corpus) -> Result<(String, serde_json::Value)> {
    recap::run(args, corpus)
}

/// Fonte: o `--source` explícito (ficheiro ou diretório) ou o diretório convencional da área.
pub fn resolve_source(session: &Session, args: &AiArgs) -> Source {
    match &args.source {
        Some(path) => Source::at(session.fs_dyn(), path),
        None => Source::jsonl_dir(session.project_root().join(args.area.default_path())),
    }
}

/// Lista as notas da fonte (`id|tipo|statement`), aplicando os filtros de identidade e a ordem.
fn list(args: &AiArgs, corpus: &Corpus) -> Result<String> {
    let limit = args.limit.unwrap_or(20);
    Ok(ordered_notes(args, corpus)?
        .iter()
        .take(limit)
        .map(|note| {
            let id = note.frontmatter.id().unwrap_or_default();
            let kind = note
                .frontmatter
                .note_type()
                .map(NoteType::as_str)
                .unwrap_or_default();
            let statement = note.frontmatter.statement().unwrap_or_default();
            format!("{id}|{kind}|{statement}")
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Notas filtradas na ordem pedida (`id` ou `(created_at, agent_id, id)`).
fn ordered_notes<'a>(args: &AiArgs, corpus: &'a Corpus) -> Result<Vec<&'a Note>> {
    let roles = parse::agent_roles(&args.agent_role)?;
    let mut notes: Vec<&Note> = corpus
        .notes
        .iter()
        .filter(|note| identity_matches(&args.agent_id, &roles, note))
        .filter(|note| {
            let created = note.frontmatter.created_at().unwrap_or(0);
            args.since.is_none_or(|s| created >= s) && args.until.is_none_or(|u| created <= u)
        })
        .collect();
    if args.sort == AiSort::Time {
        notes.sort_by_key(|note| note_key(note));
    }
    let limit = args.limit.unwrap_or(20);
    notes.truncate(limit);
    Ok(notes)
}

/// `true` se a nota passa nos filtros de `agent_id`/`agent_role` da lista.
fn identity_matches(agent_ids: &[String], roles: &[AgentRole], note: &Note) -> bool {
    let Ok(prov) = provenance(&note.frontmatter) else {
        return false;
    };
    (agent_ids.is_empty() || agent_ids.iter().any(|id| id == prov.agent_id_or_legacy()))
        && (roles.is_empty() || roles.contains(&prov.agent_role_or_main()))
}

/// Chave temporal determinística `(created_at, agent_id, id)` (D222).
fn note_key(note: &Note) -> (i64, String, String) {
    let prov = provenance(&note.frontmatter).unwrap_or_default();
    (
        note.frontmatter.created_at().unwrap_or(0),
        prov.agent_id_or_legacy().to_string(),
        note.frontmatter.id().unwrap_or_default().to_string(),
    )
}

fn list_json(args: &AiArgs, corpus: &Corpus) -> Result<serde_json::Value> {
    Ok(json!({
        "notes": ordered_notes(args, corpus)?
            .iter()
            .map(|note| note_json(note))
            .collect::<Vec<_>>(),
    }))
}

/// JSON de uma nota: identidade + `created_at` (D221/D222).
fn note_json(note: &Note) -> serde_json::Value {
    let prov = provenance(&note.frontmatter).unwrap_or_default();
    let agent = prov.agent.clone();
    let agent_id = prov.agent_id_or_legacy().to_string();
    let role = prov.agent_role_or_main().as_str();
    json!({
        "id": note.frontmatter.id().unwrap_or_default(),
        "type": note.frontmatter.note_type().map(NoteType::as_str).unwrap_or_default(),
        "statement": note.frontmatter.statement().unwrap_or_default(),
        "created_at": note.frontmatter.created_at().unwrap_or(0),
        "agent": agent,
        "agent_id": agent_id,
        "agent_role": role,
        "parent_agent_id": prov.parent_agent_id,
    })
}

/// Recall sobre a fonte: mesma cascata do `ask`, com o universo **inteiro** (a fonte é a memória
/// do agente — não há separação conhecimento/trabalho) e os estados visíveis por default (D43).
fn search(session: &Session, args: &AiArgs, corpus: &Corpus, text: &str) -> Result<Found> {
    let limit = args.limit.unwrap_or(20);
    let mut query = RecallQuery::new(text.to_string());
    query.limit = limit;
    query.filter = Filter {
        agent_ids: args.agent_id.clone(),
        agent_roles: parse::agent_roles(&args.agent_role)?,
        created_since: args.since,
        created_until: args.until,
        ..Filter::default()
    };
    query.filter.statuses = Status::VISIBLE.to_vec();
    query.universe = Universe::All;
    query.now_ms = Some(session.now_ms());
    query.strict = session.config().strict();
    let mut out = recall(&corpus.index, &corpus.graph, &query)?;
    let metas: BTreeMap<&str, &Meta> = corpus
        .index
        .docs
        .iter()
        .map(|doc| (doc.meta.id.as_str(), &doc.meta))
        .collect();
    if args.sort == AiSort::Time {
        out.hits.sort_by_key(|hit| hit_key(hit, &metas));
    }
    let text_out = if out.hits.is_empty() {
        NO_RESULTS.to_string()
    } else {
        out.hits
            .iter()
            .map(|hit| format!("{}|{}|{:.3}", hit.id, hit.statement, hit.score))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let data = json!({
        "hits": out.hits.iter()
            .map(|hit| hit_json(hit, metas.get(hit.id.as_str()).copied()))
            .collect::<Vec<_>>(),
    });
    Ok((text_out, data, out.warnings))
}

/// `agent_id` efetivo na leitura: instância, nome lógico ou `legacy` (D221).
fn effective_agent_id(meta: &Meta) -> &str {
    meta.agent_id
        .as_deref()
        .or(meta.agent.as_deref())
        .unwrap_or("legacy")
}

/// Chave temporal de um hit; metadados ausentes degradam para a identidade vazia.
fn hit_key(hit: &RecallHit, metas: &BTreeMap<&str, &Meta>) -> (i64, String, String) {
    match metas.get(hit.id.as_str()) {
        Some(meta) => (
            meta.created_ms,
            effective_agent_id(meta).to_string(),
            meta.id.clone(),
        ),
        None => (0, String::new(), hit.id.clone()),
    }
}

/// JSON de um hit com a identidade do agente e `created_at` (D221/D222).
fn hit_json(hit: &RecallHit, meta: Option<&Meta>) -> serde_json::Value {
    let mut value = json!({
        "id": hit.id,
        "statement": hit.statement,
        "score": hit.score,
    });
    let identity = match meta {
        Some(meta) => json!({
            "created_at": meta.created_ms,
            "agent": meta.agent.clone(),
            "agent_id": effective_agent_id(meta),
            "agent_role": meta.agent_role.unwrap_or_default().as_str(),
            "parent_agent_id": meta.parent_agent_id.clone(),
        }),
        None => json!({
            "created_at": 0,
            "agent": null,
            "agent_id": null,
            "agent_role": "main",
            "parent_agent_id": null,
        }),
    };
    if let (Some(object), Some(extra)) = (value.as_object_mut(), identity.as_object()) {
        for (key, item) in extra {
            let _ignored = object.insert(key.clone(), item.clone());
        }
    }
    value
}
