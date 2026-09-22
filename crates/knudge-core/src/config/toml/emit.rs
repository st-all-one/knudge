//! Emissor TOML com ordem de inserção e quoting estáveis (D63).

use crate::config::value::{ConfigValue, Table, bare_or_quoted};

/// Serializa uma tabela como TOML determinístico, terminando com `\n`.
#[must_use]
pub fn emit(table: &Table) -> String {
    let mut out = String::new();
    let mut path = Vec::new();
    emit_table(table, &mut path, &mut out);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn emit_table(table: &Table, path: &mut Vec<String>, out: &mut String) {
    for (key, value) in table {
        if matches!(value, ConfigValue::Table(_)) {
            continue;
        }
        out.push_str(&bare_or_quoted(key));
        out.push_str(" = ");
        out.push_str(&value.render());
        out.push('\n');
    }
    for (key, value) in table {
        let ConfigValue::Table(nested) = value else {
            continue;
        };
        if nested.is_empty() {
            continue;
        }
        path.push(key.clone());
        if !out.is_empty() {
            out.push('\n');
        }
        out.push('[');
        let header: Vec<String> = path.iter().map(|segment| bare_or_quoted(segment)).collect();
        out.push_str(&header.join("."));
        out.push_str("]\n");
        emit_table(nested, path, out);
        path.pop();
    }
}
