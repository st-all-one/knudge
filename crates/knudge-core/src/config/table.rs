//! Operações sobre tabelas de configuração: caminhos pontilhados, merge e poda.

use crate::config::value::{ConfigValue, Table};

/// Separa uma chave pontilhada em segmentos, rejeitando segmentos vazios.
///
/// # Errors
/// Retorna mensagem de erro se a chave tiver segmento vazio (`a..b`, `.a`, `a.`).
pub fn split_key(key: &str) -> Result<Vec<&str>, String> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.iter().any(|p| p.is_empty()) {
        return Err(format!("chave inválida: `{key}`"));
    }
    Ok(parts)
}

/// Lê um valor por caminho pontilhado.
#[must_use]
pub fn get_path<'a>(table: &'a Table, parts: &[&str]) -> Option<&'a ConfigValue> {
    let (first, rest) = parts.split_first()?;
    let value = table.get(*first)?;
    if rest.is_empty() {
        Some(value)
    } else {
        get_path(value.as_table()?, rest)
    }
}

/// Insere um valor por caminho pontilhado, criando tabelas intermediárias.
///
/// # Errors
/// Retorna erro se um ancestral existente não for tabela.
pub fn set_path(table: &mut Table, parts: &[&str], value: ConfigValue) -> Result<(), String> {
    let (first, rest) = parts
        .split_first()
        .ok_or_else(|| "caminho vazio".to_string())?;
    if rest.is_empty() {
        table.insert((*first).to_string(), value);
        return Ok(());
    }
    let entry = table
        .entry((*first).to_string())
        .or_insert_with(|| ConfigValue::Table(Table::new()));
    let nested = entry
        .as_table_mut()
        .ok_or_else(|| format!("`{first}` não é uma tabela"))?;
    set_path(nested, rest, value)
}

/// Remove um valor por caminho pontilhado e poda ancestrais que ficarem vazios (D64).
///
/// Devolve `true` se algo foi removido.
pub fn remove_path(table: &mut Table, parts: &[&str]) -> bool {
    let Some((first, rest)) = parts.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return table.shift_remove(*first).is_some();
    }
    let removed = table
        .get_mut(*first)
        .and_then(ConfigValue::as_table_mut)
        .is_some_and(|nested| remove_path(nested, rest));
    if removed
        && table
            .get(*first)
            .and_then(ConfigValue::as_table)
            .is_some_and(Table::is_empty)
    {
        table.shift_remove(*first);
    }
    removed
}

/// Achata a tabela em pares `(chave.pontilhada, referência)` em ordem de inserção.
#[must_use]
pub fn flatten(table: &Table) -> Vec<(String, &ConfigValue)> {
    let mut out = Vec::new();
    for (key, value) in table {
        match value {
            ConfigValue::Table(nested) => {
                for (child, leaf) in flatten(nested) {
                    out.push((format!("{key}.{child}"), leaf));
                }
            }
            other => out.push((key.clone(), other)),
        }
    }
    out
}

/// Merge profundo: valores de `over` vencem; tabelas são combinadas (D61).
pub fn merge_into(base: &mut Table, over: &Table) {
    for (key, value) in over {
        match (base.get_mut(key), value) {
            (Some(ConfigValue::Table(existing)), ConfigValue::Table(incoming)) => {
                merge_into(existing, incoming);
            }
            _ => {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}

/// Remove uma subárvore pelo primeiro segmento (ex.: `secrets`).
pub fn remove_prefix(table: &mut Table, prefix: &str) {
    table.shift_remove(prefix);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Table {
        Table::new()
    }

    #[test]
    fn set_get_and_remove_prune() {
        let mut t = table();
        let _ignored = set_path(&mut t, &["a", "b", "c"], ConfigValue::Int(1));
        assert_eq!(get_path(&t, &["a", "b", "c"]), Some(&ConfigValue::Int(1)));
        assert!(remove_path(&mut t, &["a", "b", "c"]));
        assert!(t.is_empty(), "ancestrais vazios devem ser podados");
    }

    #[test]
    fn merge_prefers_overlay() {
        let mut base = table();
        let mut over = table();
        let _ignored = set_path(&mut base, &["x", "a"], ConfigValue::Int(1));
        let _ignored = set_path(&mut base, &["x", "b"], ConfigValue::Int(2));
        let _ignored = set_path(&mut over, &["x", "b"], ConfigValue::Int(9));
        let _ignored = set_path(&mut over, &["y"], ConfigValue::Bool(true));
        merge_into(&mut base, &over);
        assert_eq!(get_path(&base, &["x", "a"]), Some(&ConfigValue::Int(1)));
        assert_eq!(get_path(&base, &["x", "b"]), Some(&ConfigValue::Int(9)));
        assert_eq!(get_path(&base, &["y"]), Some(&ConfigValue::Bool(true)));
    }

    #[test]
    fn rejects_empty_segments() {
        assert!(split_key("a..b").is_err());
        assert_eq!(split_key("a.b").ok(), Some(vec!["a", "b"]));
    }

    #[test]
    fn flatten_uses_dotted_keys() {
        let mut t = table();
        let _ignored = set_path(&mut t, &["a", "b"], ConfigValue::Int(1));
        let flat = flatten(&t);
        assert_eq!(flat.len(), 1);
        assert_eq!(flat.first().map(|(k, _)| k.as_str()), Some("a.b"));
    }
}
