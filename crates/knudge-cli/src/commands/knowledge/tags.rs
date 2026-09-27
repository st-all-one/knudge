//! `kd ask --tags` — vocabulário de tags (`tag|count`, `count` desc — ex-`kd ask --tags`, D107/D146).

use knudge_core::Result;
use knudge_core::retrieval::tag_counts;
use serde_json::json;

use crate::cli::TagsArgs;
use crate::output::Output;
use crate::session::Session;

/// `kd ask --tags` — vocabulário de tags do corpus.
///
/// # Errors
/// Propaga erros de leitura do índice.
pub fn run(session: &Session, args: &TagsArgs) -> Result<Output> {
    let index = session.index()?;
    let mut counts = tag_counts(&index);
    if let Some(limit) = args.limit {
        counts.truncate(limit);
    }
    let text = counts
        .iter()
        .map(|(tag, count)| format!("{tag}|{count}"))
        .collect::<Vec<_>>()
        .join("\n");
    let data = json!({
        "tags": counts.iter().map(|(tag, count)| json!({"tag": tag, "count": count})).collect::<Vec<_>>(),
    });
    Ok(Output::new(text, data))
}
