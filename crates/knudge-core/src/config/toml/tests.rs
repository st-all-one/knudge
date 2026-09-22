//! Testes do codec TOML (parse/emit e round-trip).

use super::{emit, parse};
use crate::Result;
use crate::config::schema::default_table;
use crate::config::table::get_path;
use crate::config::value::ConfigValue;

#[test]
fn defaults_round_trip_byte_stable() -> Result<()> {
    let table = default_table();
    let first = emit(&table);
    let parsed = parse(&first)?;
    let second = emit(&parsed);
    assert_eq!(first, second, "emit(parse(emit)) deve ser estável");
    assert_eq!(parsed, table, "o round-trip deve preservar valores");
    Ok(())
}

#[test]
fn parses_comments_keys_and_types() -> Result<()> {
    let text = "\
# comentário de topo
[embeddings]
enabled = true   # inline
provider = 'local'
dimensions = 384
model = \"a\\tb\"
mode = \"lazy\"
flush_ms = 2_000

[dedup]
create_below = 0.75
merge_below = 0.92
";
    let table = parse(text)?;
    assert_eq!(
        get_path(&table, &["embeddings", "provider"]),
        Some(&ConfigValue::String("local".into()))
    );
    assert_eq!(
        get_path(&table, &["embeddings", "model"]),
        Some(&ConfigValue::String("a\tb".into()))
    );
    assert_eq!(
        get_path(&table, &["embeddings", "flush_ms"]),
        Some(&ConfigValue::Int(2000))
    );
    assert_eq!(
        get_path(&table, &["dedup", "create_below"]),
        Some(&ConfigValue::Float(0.75))
    );
    Ok(())
}

#[test]
fn parses_multiline_arrays_and_literals() -> Result<()> {
    let text = "\
[embeddings]
provider = \"local\"
[lista]
values = [
  \"a\",
  'b',  # comentário
  \"c\",
]
";
    let table = parse(text)?;
    let expected = ConfigValue::Array(vec![
        ConfigValue::String("a".into()),
        ConfigValue::String("b".into()),
        ConfigValue::String("c".into()),
    ]);
    assert_eq!(get_path(&table, &["lista", "values"]), Some(&expected));
    Ok(())
}

#[test]
fn dotted_keys_and_unicode_escapes() -> Result<()> {
    let text = "ids.prefix_style = \"compact\"\nemoji = \"\\u0041\"\n";
    let table = parse(text)?;
    assert_eq!(
        get_path(&table, &["ids", "prefix_style"]),
        Some(&ConfigValue::String("compact".into()))
    );
    assert_eq!(
        get_path(&table, &["emoji"]),
        Some(&ConfigValue::String("A".into()))
    );
    Ok(())
}

#[test]
fn rejects_invalid_documents() {
    assert!(parse("a = 1\na = 2\n").is_err(), "chave duplicada");
    assert!(parse("a = \n").is_err(), "valor ausente");
    assert!(parse("[[x]]\n").is_err(), "array de tabelas");
    assert!(parse("a = \"\"\"x\"\"\"\n").is_err(), "multilinha");
    assert!(parse("a = 1 b = 2\n").is_err(), "duas chaves na linha");
    assert!(parse("[a\n").is_err(), "cabeçalho sem fechamento");
}
