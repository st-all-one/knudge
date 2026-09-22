//! Emissor do subconjunto TOON (D74/D75).
//!
//! Saída **canônica**: mapas na ordem de inserção (D04), escalares sem aspas quando seguros,
//! listas de escalares em *flow* (`[a, b]`), listas de mapas em bloco (`- chave: valor`),
//! `LF` no fim de cada linha. Campos vazios são **omitidos** (D05) e nunca emitidos como `null`.

use indexmap::IndexMap;

use crate::schema::Value;

/// Serializa um documento TOON. O topo deve ser [`Value::Map`]; outro tipo cai em *flow*.
#[must_use]
pub fn emit(value: &Value) -> String {
    let mut out = String::new();
    match value {
        Value::Map(map) => emit_map(&mut out, map, 0),
        other => {
            out.push_str(&emit_flow(other));
            out.push('\n');
        }
    }
    out
}

fn emit_map(out: &mut String, map: &IndexMap<String, Value>, indent: usize) {
    for (key, value) in map {
        match value {
            Value::Map(inner) if !inner.is_empty() => {
                push_header(out, indent, key);
                emit_map(out, inner, indent.saturating_add(2));
            }
            Value::Map(_) | Value::List(_) if is_empty(value) => {}
            Value::List(items) if items.iter().any(is_block_item) => {
                push_header(out, indent, key);
                emit_list_block(out, items, indent.saturating_add(2));
            }
            _ => {
                let flow = emit_flow(value);
                push_indent(out, indent);
                out.push_str(key);
                out.push_str(": ");
                out.push_str(&flow);
                out.push('\n');
            }
        }
    }
}

fn is_empty(value: &Value) -> bool {
    match value {
        Value::Map(map) => map.is_empty(),
        Value::List(items) => items.is_empty(),
        _ => false,
    }
}

fn is_block_item(value: &Value) -> bool {
    matches!(value, Value::Map(_) | Value::List(_))
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn push_header(out: &mut String, indent: usize, key: &str) {
    push_indent(out, indent);
    out.push_str(key);
    out.push_str(":\n");
}

fn emit_list_block(out: &mut String, items: &[Value], indent: usize) {
    for item in items {
        match item {
            Value::Map(map) if !map.is_empty() => {
                for (position, (key, value)) in map.iter().enumerate() {
                    let prefix = if position == 0 { "- " } else { "  " };
                    push_indent(out, indent);
                    out.push_str(prefix);
                    out.push_str(key);
                    out.push_str(": ");
                    out.push_str(&emit_flow(value));
                    out.push('\n');
                }
            }
            other => {
                let flow = emit_flow(other);
                push_indent(out, indent);
                out.push_str("- ");
                out.push_str(&flow);
                out.push('\n');
            }
        }
    }
}

fn emit_flow(value: &Value) -> String {
    match value {
        Value::Str(text) => {
            if is_plain(text) {
                text.clone()
            } else {
                quote(text)
            }
        }
        Value::Int(number) => number.to_string(),
        Value::Float(number) => format!("{number}"),
        Value::Bool(flag) => flag.to_string(),
        Value::List(items) => {
            let parts: Vec<String> = items.iter().map(emit_flow).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Map(map) => {
            let parts: Vec<String> = map
                .iter()
                .map(|(key, value)| format!("{key}: {}", emit_flow(value)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}

fn looks_numeric(text: &str) -> bool {
    text.chars().any(|c| c == '.' || c == 'e' || c == 'E')
}

fn is_plain(text: &str) -> bool {
    if text.is_empty() || text == "-" {
        return false;
    }
    if text.trim() != text || text.starts_with("- ") {
        return false;
    }
    if text == "true" || text == "false" {
        return false;
    }
    if text.chars().any(char::is_control) {
        return false;
    }
    if text
        .chars()
        .any(|c| matches!(c, '"' | '\\' | '#' | ',' | '[' | ']' | '{' | '}'))
    {
        return false;
    }
    if text.parse::<i64>().is_ok() {
        return false;
    }
    if looks_numeric(text) && text.parse::<f64>().is_ok() {
        return false;
    }
    true
}

fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(2));
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let hex = format!("{:x}", u32::from(c));
                out.push_str("\\u{");
                out.push_str(&hex);
                out.push('}');
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
