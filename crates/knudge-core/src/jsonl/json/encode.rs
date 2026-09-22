//! Emissão JSON compacta e determinística (chaves ordenadas).

use std::fmt::Write as _;

use crate::schema::Value;
use crate::{Error, Result};

/// Codifica um valor como JSON compacto e determinístico.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` se houver float não finito.
pub fn encode(value: &Value) -> Result<String> {
    let mut out = String::new();
    encode_into(value, &mut out)?;
    Ok(out)
}

fn encode_into(value: &Value, out: &mut String) -> Result<()> {
    match value {
        Value::Str(text) => encode_str(text, out),
        Value::Int(number) => {
            let _ignored = write!(out, "{number}");
        }
        Value::Float(number) => encode_float(*number, out)?,
        Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Value::List(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                encode_into(item, out)?;
            }
            out.push(']');
        }
        Value::Map(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                encode_str(key, out);
                out.push(':');
                if let Some(item) = map.get(key) {
                    encode_into(item, out)?;
                }
            }
            out.push('}');
        }
    }
    Ok(())
}

fn encode_float(number: f64, out: &mut String) -> Result<()> {
    if !number.is_finite() {
        return Err(Error::invalid_input("float não finito em JSON"));
    }
    let text = format!("{number}");
    let integral = !text.contains('.') && !text.contains('e') && !text.contains('E');
    out.push_str(&text);
    if integral {
        out.push_str(".0");
    }
    Ok(())
}

fn encode_str(text: &str, out: &mut String) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c < ' ' => {
                let code = u32::from(c);
                out.push_str("\\u00");
                out.push(hex_digit((code >> 4) & 0xf));
                out.push(hex_digit(code & 0xf));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

const fn hex_digit(value: u32) -> char {
    match char::from_digit(value, 16) {
        Some(ch) => ch,
        None => '0',
    }
}
