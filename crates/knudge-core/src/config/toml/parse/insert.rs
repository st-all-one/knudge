//! Inserção de tabelas e folhas no documento, rejeitando conflitos.

use crate::config::value::{ConfigValue, Table};
use crate::{Error, Result};

/// Garante que a tabela do cabeçalho exista, sem sobrescrever valores.
pub(super) fn ensure_table(root: &mut Table, path: &[String]) -> Result<()> {
    let mut table = root;
    for segment in path {
        let entry = table
            .entry(segment.clone())
            .or_insert_with(|| ConfigValue::Table(Table::new()));
        table = entry
            .as_table_mut()
            .ok_or_else(|| Error::config(format!("`{segment}` não é uma tabela")))?;
    }
    Ok(())
}

/// Insere a folha, criando tabelas intermediárias e rejeitando duplicatas.
pub(super) fn insert_leaf(root: &mut Table, path: &[String], value: ConfigValue) -> Result<()> {
    let Some((last, parents)) = path.split_last() else {
        return Err(Error::config("chave vazia"));
    };
    let mut table = root;
    for segment in parents {
        let entry = table
            .entry(segment.clone())
            .or_insert_with(|| ConfigValue::Table(Table::new()));
        table = entry
            .as_table_mut()
            .ok_or_else(|| Error::config(format!("`{segment}` não é uma tabela")))?;
    }
    if table.contains_key(last) {
        return Err(Error::config(format!("chave duplicada: `{last}`")));
    }
    table.insert(last.clone(), value);
    Ok(())
}
