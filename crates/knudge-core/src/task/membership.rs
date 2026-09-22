//! Filiação de tarefa por marcador no corpo (D52/D93).
//!
//! O pai é uma **view derivada**: a relação dura vive no marcador do corpo
//! (`<!-- knudge:parent <id> blocks <n> -->`), legível por humanos/LLMs mesmo sem o grafo. A
//! aresta `results_in` do pai para o filho é a projeção derivada dessa relação.

/// Prefixo do marcador de pai.
pub const MARKER_PREFIX: &str = "<!-- knudge:parent";

/// Filiação extraída do corpo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    /// Id do pai.
    pub parent: String,
    /// Ordem 1-based dentro do pai (`blocks`), quando declarada.
    pub blocks: Option<u32>,
}

/// Renderiza a linha do marcador.
#[must_use]
pub fn render(parent: &str, blocks: Option<u32>) -> String {
    match blocks {
        Some(number) => format!("{MARKER_PREFIX} {parent} blocks {number} -->"),
        None => format!("{MARKER_PREFIX} {parent} -->"),
    }
}

/// Extrai o marcador do corpo, se houver.
#[must_use]
pub fn parse(body: &str) -> Option<Marker> {
    for line in body.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(MARKER_PREFIX) else {
            continue;
        };
        let rest = rest.strip_suffix("-->").unwrap_or(rest).trim();
        let mut tokens = rest.split_whitespace();
        let parent = tokens.next()?.to_string();
        let mut blocks = None;
        while let Some(token) = tokens.next() {
            if token == "blocks" {
                blocks = tokens.next().and_then(|value| value.parse::<u32>().ok());
            }
        }
        return Some(Marker { parent, blocks });
    }
    None
}

/// Remove a linha do marcador (sem tocar no resto).
#[must_use]
pub fn strip(body: &str) -> String {
    let mut kept: Vec<&str> = body
        .lines()
        .filter(|line| !line.trim_start().starts_with(MARKER_PREFIX))
        .collect();
    while kept.last().is_some_and(|line| line.trim().is_empty()) {
        let _ignored = kept.pop();
    }
    kept.join("\n")
}

/// Substitui (ou insere) o marcador no fim do corpo.
#[must_use]
pub fn set(body: &str, parent: &str, blocks: Option<u32>) -> String {
    let cleaned = strip(body);
    let marker = render(parent, blocks);
    if cleaned.is_empty() {
        marker
    } else {
        format!("{cleaned}\n{marker}")
    }
}
