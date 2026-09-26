//! Parser de blocos do subconjunto TOON (D74/D75).
//!
//! A gramática está em `TOON.md`. Documentos são **mapas**; entradas são `chave: valor`;
//! valores são escalares, coleções *flow* (`[a, b]`, `{a: 1}` — ver [`super::flow`]) ou blocos
//! aninhados por indentação de 2 espaços. Comentários `#` (fora de aspas) são ignorados.
//! `null`/`~` não fazem parte do subconjunto: campos opcionais ausentes são **omitidos** (D05).

#![allow(
    clippy::arithmetic_side_effects,
    reason = "índices de linha e indentação com limites conhecidos"
)]

use indexmap::IndexMap;

use super::flow::{insert, parse_flow, split_key};
use super::lex::{Line, split_lines};
use super::{fail, normalize_newlines};
use crate::Result;
use crate::schema::Value;

/// Profundidade máxima de aninhamento (proteção contra input hostil).
const MAX_DEPTH: usize = 32;

/// Parseia um documento TOON cujo topo é um mapa.
pub fn parse(src: &str) -> Result<Value> {
    let normalized = normalize_newlines(src);
    let lines = split_lines(&normalized)?;
    let mut index = 0_usize;
    let Some(first) = lines.first() else {
        return Ok(Value::Map(IndexMap::new()));
    };
    if first.indent != 0 {
        return Err(fail("documento deve começar sem indentação"));
    }
    let (value, next) = parse_node(&lines, &mut index, 0, 0)?;
    if next != lines.len() {
        return Err(fail(format!("conteúdo inesperado na linha {}", next + 1)));
    }
    Ok(value)
}

fn is_dash(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

fn parse_node(
    lines: &[Line<'_>],
    index: &mut usize,
    indent: usize,
    depth: usize,
) -> Result<(Value, usize)> {
    if depth > MAX_DEPTH {
        return Err(fail("aninhamento TOON profundo demais"));
    }
    let Some(line) = lines.get(*index) else {
        return Ok((Value::Map(IndexMap::new()), *index));
    };
    if line.indent != indent {
        return Err(fail("indentação inconsistente"));
    }
    if is_dash(&line.text) {
        let items = parse_list(lines, index, indent, depth)?;
        Ok((Value::List(items), *index))
    } else {
        let map = parse_map(lines, index, indent, depth)?;
        Ok((Value::Map(map), *index))
    }
}

fn parse_map(
    lines: &[Line<'_>],
    index: &mut usize,
    indent: usize,
    depth: usize,
) -> Result<IndexMap<String, Value>> {
    let mut map = IndexMap::new();
    while let Some(line) = lines.get(*index) {
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            return Err(fail(format!("indentação inesperada: {:?}", line.text)));
        }
        if is_dash(&line.text) {
            break;
        }
        let (key, rest) = split_key(&line.text)?;
        *index += 1;
        let value = if rest.is_empty() {
            if next_is_deeper(lines, *index, indent) {
                let child_indent = lines.get(*index).map_or(indent + 2, |child| child.indent);
                let (value, next) = parse_node(lines, index, child_indent, depth + 1)?;
                *index = next;
                value
            } else {
                Value::Map(IndexMap::new())
            }
        } else {
            parse_flow(&rest)?
        };
        insert(&mut map, key, value)?;
    }
    Ok(map)
}

fn parse_list(
    lines: &[Line<'_>],
    index: &mut usize,
    indent: usize,
    depth: usize,
) -> Result<Vec<Value>> {
    let mut items = Vec::new();
    while let Some(line) = lines.get(*index) {
        if line.indent != indent || !is_dash(&line.text) {
            break;
        }
        let content = line.text.strip_prefix("- ").unwrap_or("").to_string();
        *index += 1;
        let value = if content.is_empty() {
            if next_is_deeper(lines, *index, indent) {
                let child_indent = lines.get(*index).map_or(indent + 2, |child| child.indent);
                let (value, next) = parse_node(lines, index, child_indent, depth + 1)?;
                *index = next;
                value
            } else {
                Value::Map(IndexMap::new())
            }
        } else if looks_like_pair(&content) {
            parse_map_item(lines, index, indent, &content, depth)?
        } else {
            parse_flow(&content)?
        };
        items.push(value);
    }
    Ok(items)
}

fn parse_map_item(
    lines: &[Line<'_>],
    index: &mut usize,
    indent: usize,
    content: &str,
    depth: usize,
) -> Result<Value> {
    let mut map = IndexMap::new();
    let (key, rest) = split_key(content)?;
    if rest.is_empty() {
        return Err(fail("item de mapa precisa de valor inline"));
    }
    insert(&mut map, key, parse_flow(&rest)?)?;
    if next_is_deeper(lines, *index, indent) {
        let child_indent = lines.get(*index).map_or(indent + 2, |child| child.indent);
        for (key, value) in parse_map(lines, index, child_indent, depth + 1)? {
            insert(&mut map, key, value)?;
        }
    }
    Ok(Value::Map(map))
}

fn next_is_deeper(lines: &[Line<'_>], index: usize, indent: usize) -> bool {
    lines.get(index).is_some_and(|line| line.indent > indent)
}

/// Um item de lista é mapa quando há `chave: ` (ou `chave:`) no nível zero.
fn looks_like_pair(text: &str) -> bool {
    let Ok((key, rest)) = split_key(text) else {
        return false;
    };
    let key_ok = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    let separated = rest.is_empty() || text.contains(": ");
    key_ok && separated
}
