//! `kd ai` — consulta **read-only** de uma fonte de memória de agente (D218).
//!
//! O knudge **indexa** (derivado); o agente **possui** a fonte. Esta área é fina de propósito: lê
//! um store JSONL de notas ([`Source::Jsonl`], o codec `{frontmatter, body}` de `Note::to_jsonl`),
//! deriva índice+grafo e responde com o mesmo motor do `ask` — sem reimplementar retrieval e sem
//! aprender o schema interno do agente (o contrato é o **documento**).

use std::path::PathBuf;

use knudge_core::Result;
use knudge_core::corpus::{Corpus, Source};
use knudge_core::retrieval::{DEFAULT_LIMIT, RecallQuery, Universe, recall};
use knudge_core::schema::{NoteType, Status};
use serde_json::json;

use crate::cli::AiArgs;
use crate::output::Output;
use crate::session::Session;

/// Resultado de um recall sobre a fonte: texto (pipe) + JSON + avisos de canal.
type Found = (String, serde_json::Value, Vec<String>);

/// Sentinela de busca vazia (D152), como no `ask`: distingue "sem resultado" de erro.
const NO_RESULTS: &str = "[no_results]";

/// Executa `kd ai`: lista a fonte (sem query) ou faz recall sobre ela (com query).
///
/// # Errors
/// Propaga erros de leitura da fonte e de derivação do índice/grafo; `strict` promove avisos a erro.
pub fn run(session: &Session, args: &AiArgs) -> Result<Output> {
    let path = resolve_source(session, args);
    let (corpus, mut warnings) = Source::jsonl(&path).load(&session.store())?;
    let query = args.query.join(" ");
    let (text, data) = if query.is_empty() {
        (list(&corpus), list_json(&corpus))
    } else {
        let (text, data, extra) = search(session, args, &corpus, &query)?;
        warnings.extend(extra);
        (text, data)
    };
    let data = json!({
        "area": args.area.as_str(),
        "source": path.display().to_string(),
        "query": query,
        "data": data,
    });
    Ok(Output::new(text, data).with_warnings(warnings))
}

/// Caminho da fonte: o `--source` explícito ou o convencional da área.
fn resolve_source(session: &Session, args: &AiArgs) -> PathBuf {
    match &args.source {
        Some(path) => PathBuf::from(path),
        None => session.project_root().join(args.area.default_path()),
    }
}

/// Lista as notas da fonte (`id|tipo|statement`), na ordem determinística do corpus.
fn list(corpus: &Corpus) -> String {
    corpus
        .notes
        .iter()
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
        .join("\n")
}

fn list_json(corpus: &Corpus) -> serde_json::Value {
    json!({
        "notes": corpus.notes.iter().map(|note| json!({
            "id": note.frontmatter.id().unwrap_or_default(),
            "type": note.frontmatter.note_type().map(NoteType::as_str).unwrap_or_default(),
            "statement": note.frontmatter.statement().unwrap_or_default(),
        })).collect::<Vec<_>>(),
    })
}

/// Recall sobre a fonte: mesma cascata do `ask`, com o universo **inteiro** (a fonte é a memória
/// do agente — não há separação conhecimento/trabalho) e os estados visíveis por default (D43).
fn search(session: &Session, args: &AiArgs, corpus: &Corpus, text: &str) -> Result<Found> {
    let limit = args.limit.unwrap_or_else(|| {
        session
            .config()
            .get_int("recall.default_limit")
            .and_then(|raw| usize::try_from(raw).ok())
            .unwrap_or(DEFAULT_LIMIT)
    });
    let mut query = RecallQuery::new(text.to_string());
    query.limit = limit;
    query.filter.statuses = Status::VISIBLE.to_vec();
    query.universe = Universe::All;
    query.now_ms = Some(session.now_ms());
    query.strict = session.config().strict();
    let out = recall(&corpus.index, &corpus.graph, &query)?;
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
        "hits": out.hits.iter().map(|hit| json!({
            "id": hit.id,
            "statement": hit.statement,
            "score": hit.score,
        })).collect::<Vec<_>>(),
    });
    Ok((text_out, data, out.warnings))
}
