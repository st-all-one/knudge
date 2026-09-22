//! Conversão de argumentos de CLI em tipos do domínio (E12-T01).

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::schema::{Classification, EdgeKind, NoteType, Status};
use knudge_core::time::Timestamp;

/// Converte uma lista de tipos fechados.
///
/// # Errors
/// Retorna `ErrorKind::Schema` para tipo desconhecido.
pub fn types(values: &[String]) -> Result<Vec<NoteType>> {
    values.iter().map(|value| value.parse()).collect()
}

/// Converte uma lista de classificações fechadas.
///
/// # Errors
/// Retorna `ErrorKind::Schema` para classificação desconhecida.
pub fn classifications(values: &[String]) -> Result<Vec<Classification>> {
    values.iter().map(|value| value.parse()).collect()
}

/// Converte uma lista de status fechados.
///
/// # Errors
/// Retorna `ErrorKind::Schema` para status desconhecido.
pub fn statuses(values: &[String]) -> Result<Vec<Status>> {
    values.iter().map(|value| value.parse()).collect()
}

/// Converte um timestamp RFC3339/epoch em ms.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` para timestamp malformado.
pub fn timestamp(value: &str) -> Result<i64> {
    Ok(value.parse::<Timestamp>()?.as_millis())
}

/// Converte um timestamp opcional.
///
/// # Errors
/// Propaga erro de formato do timestamp.
pub fn timestamp_opt(value: Option<&String>) -> Result<Option<i64>> {
    value.map(|text| timestamp(text)).transpose()
}

/// Interpreta `ARESTA:ID` em `(EdgeKind, id)`.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` se faltar `:` ou o id estiver vazio.
pub fn edge(spec: &str) -> Result<(EdgeKind, String)> {
    let (kind, to) = spec
        .split_once(':')
        .ok_or_else(|| Error::invalid_input(format!("aresta esperada como ARESTA:ID: {spec:?}")))?;
    if to.is_empty() {
        return Err(Error::invalid_input("id da aresta vazio"));
    }
    Ok((kind.parse()?, to.to_string()))
}

/// Interpreta `FROM:ARESTA:TO` em `(from, EdgeKind, to)`.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` se o formato estiver errado.
pub fn triple(spec: &str) -> Result<(String, EdgeKind, String)> {
    let mut parts = spec.splitn(3, ':');
    let from = parts
        .next()
        .filter(|part| !part.is_empty())
        .ok_or_else(|| Error::invalid_input("link esperado como FROM:ARESTA:TO"))?;
    let kind = parts
        .next()
        .filter(|part| !part.is_empty())
        .ok_or_else(|| Error::invalid_input("link esperado como FROM:ARESTA:TO"))?;
    let to = parts
        .next()
        .filter(|part| !part.is_empty())
        .ok_or_else(|| Error::invalid_input("link esperado como FROM:ARESTA:TO"))?;
    Ok((from.to_string(), kind.parse()?, to.to_string()))
}
