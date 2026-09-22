//! Escalares e coleções *flow* do subconjunto TOON (`[a, b]`, `{a: 1}`).
//!
//! `null`/`~` não existem no subconjunto: um valor textual assim vira string (D05).

use indexmap::IndexMap;

use super::fail;
use crate::Result;
use crate::schema::Value;

/// Insere um par rejeitando chave duplicada.
pub(crate) fn insert(map: &mut IndexMap<String, Value>, key: String, value: Value) -> Result<()> {
    if map.contains_key(&key) {
        return Err(fail(format!("chave duplicada: {key:?}")));
    }
    map.insert(key, value);
    Ok(())
}

/// Separa `chave: valor` no primeiro `:` de nível zero (fora de aspas/coleções).
pub(crate) fn split_key(text: &str) -> Result<(String, String)> {
    let mut in_quotes = false;
    let mut escaped = false;
    let mut depth = 0_usize;
    for (idx, ch) in text.char_indices() {
        if in_quotes {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quotes = false;
            }
            continue;
        }
        match ch {
            '"' => in_quotes = true,
            '[' | '{' => depth = depth.saturating_add(1),
            ']' | '}' => depth = depth.saturating_sub(1),
            ':' if depth == 0 => {
                let Some(prefix) = text.get(..idx) else {
                    return Err(fail("chave inválida"));
                };
                let Some(suffix) = text.get(idx.saturating_add(1)..) else {
                    return Err(fail("chave inválida"));
                };
                let key = prefix.trim().to_string();
                if key.is_empty() {
                    return Err(fail("chave vazia"));
                }
                return Ok((key, suffix.trim().to_string()));
            }
            _ => {}
        }
    }
    Err(fail(format!("esperava `:` em {text:?}")))
}

/// Parseia um valor *flow*: escalar, `[a, b]` ou `{a: 1}`.
pub(crate) fn parse_flow(text: &str) -> Result<Value> {
    let text = text.trim();
    if let Some(inner) = text.strip_prefix('[') {
        let inner = inner
            .strip_suffix(']')
            .ok_or_else(|| fail("lista sem `]` de fechamento"))?;
        if inner.trim().is_empty() {
            return Ok(Value::List(Vec::new()));
        }
        let mut items = Vec::new();
        for part in split_top_level(inner, ',')? {
            items.push(parse_flow(&part)?);
        }
        return Ok(Value::List(items));
    }
    if let Some(inner) = text.strip_prefix('{') {
        let inner = inner
            .strip_suffix('}')
            .ok_or_else(|| fail("mapa sem `}` de fechamento"))?;
        let mut map = IndexMap::new();
        if inner.trim().is_empty() {
            return Ok(Value::Map(map));
        }
        for part in split_top_level(inner, ',')? {
            let (key, rest) = split_key(&part)?;
            if rest.is_empty() {
                return Err(fail("par de mapa flow precisa de valor"));
            }
            insert(&mut map, key, parse_flow(&rest)?)?;
        }
        return Ok(Value::Map(map));
    }
    parse_scalar(text)
}

fn split_top_level(text: &str, separator: char) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;
    let mut depth = 0_usize;
    for ch in text.chars() {
        if in_quotes {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quotes = false;
            }
            continue;
        }
        match ch {
            '"' => {
                in_quotes = true;
                current.push(ch);
            }
            '[' | '{' => {
                depth = depth.saturating_add(1);
                current.push(ch);
            }
            ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            c if c == separator && depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if in_quotes {
        return Err(fail("string sem aspas de fechamento"));
    }
    let last = current.trim();
    if !last.is_empty() || !parts.is_empty() {
        parts.push(last.to_string());
    }
    Ok(parts)
}

fn parse_scalar(text: &str) -> Result<Value> {
    if text.is_empty() {
        return Ok(Value::Str(String::new()));
    }
    if let Some(inner) = text.strip_prefix('"') {
        let inner = inner
            .strip_suffix('"')
            .ok_or_else(|| fail("string sem aspas de fechamento"))?;
        return Ok(Value::Str(unescape(inner)?));
    }
    match text {
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }
    if let Ok(int) = text.parse::<i64>() {
        return Ok(Value::Int(int));
    }
    if looks_numeric(text)
        && let Ok(float) = text.parse::<f64>()
        && float.is_finite()
    {
        return Ok(Value::Float(float));
    }
    Ok(Value::Str(text.to_string()))
}

fn looks_numeric(text: &str) -> bool {
    text.chars().any(|c| c == '.' || c == 'e' || c == 'E')
}

fn unescape(inner: &str) -> Result<String> {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let Some(next) = chars.next() else {
            return Err(fail("escape incompleto"));
        };
        match next {
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            '0' => out.push('\0'),
            'u' => out.push(unescape_unicode(&mut chars)?),
            other => return Err(fail(format!("escape desconhecido: \\{other}"))),
        }
    }
    Ok(out)
}

fn unescape_unicode(chars: &mut std::str::Chars<'_>) -> Result<char> {
    if chars.next() != Some('{') {
        return Err(fail("esperava `{` após \\u"));
    }
    let mut hex = String::new();
    loop {
        match chars.next() {
            Some('}') => break,
            Some(c) => hex.push(c),
            None => return Err(fail("escape unicode sem `}`")),
        }
    }
    let code = u32::from_str_radix(&hex, 16).map_err(|_| fail("escape unicode inválido"))?;
    char::from_u32(code).ok_or_else(|| fail("escalar unicode inválido"))
}
