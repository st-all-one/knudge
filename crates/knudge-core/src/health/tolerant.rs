//! Leitura tolerante do corpus (D16–D18, E09-T05).
//!
//! Uma nota ruim **não derruba o comando**: chave desconhecida vira warning (D16), `type`
//! desconhecido rejeita a nota (D17) e nota malformada é **skipada com orientação de correção**
//! (D18). O `recall` segue funcionando sobre o que sobrou.

use crate::store::{Note, Store};
use crate::{Error, ErrorKind, Result};

/// Resultado da leitura de uma nota individual: `Ok((nota, warnings))` ou uma nota pulada.
pub type TolerantNote = std::result::Result<(Note, Vec<String>), SkippedNote>;

/// Erro interno de leitura: `(motivo, orientação)`.
type ReadFailure = (String, String);

/// Nota descartada na leitura tolerante, com o motivo e a orientação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedNote {
    /// Id derivado do nome do arquivo.
    pub id: String,
    /// Motivo legível.
    pub reason: String,
    /// Orientação de correção.
    pub guidance: String,
}

/// Resultado de uma leitura tolerante.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TolerantRead {
    /// Notas válidas, ordenadas por id.
    pub notes: Vec<Note>,
    /// Notas descartadas.
    pub skipped: Vec<SkippedNote>,
    /// Warnings acumulados (chave desconhecida, nota pulada).
    pub warnings: Vec<String>,
}

/// Lê todas as notas do store sem abortar por nota ruim.
///
/// # Errors
/// Retorna `ErrorKind::Io` se a **listagem** do diretório falhar (falha global, não por nota).
pub fn read_tolerant(store: &Store<'_>) -> Result<TolerantRead> {
    let mut result = TolerantRead::default();
    for id in store.list_ids()? {
        match read_one(store, &id) {
            Ok((note, warnings)) => {
                for warning in warnings {
                    result.warnings.push(format!("{id}: {warning}"));
                }
                result.notes.push(note);
            }
            Err((reason, guidance)) => {
                result
                    .warnings
                    .push(format!("nota pulada: {id} ({reason})"));
                result.skipped.push(SkippedNote {
                    id,
                    reason,
                    guidance,
                });
            }
        }
    }
    Ok(result)
}

/// Lê uma nota específica de forma tolerante.
///
/// # Errors
/// Retorna `ErrorKind::NotFound`/`Io` se a leitura falhar de forma global.
pub fn read_note_tolerant(store: &Store<'_>, id: &str) -> Result<TolerantNote> {
    if !store.exists(id) {
        return Err(Error::not_found(format!("nota ausente: {id}")));
    }
    match read_one(store, id) {
        Ok((note, warnings)) => Ok(Ok((note, warnings))),
        Err((reason, guidance)) => Ok(Err(SkippedNote {
            id: id.to_string(),
            reason,
            guidance,
        })),
    }
}

fn read_one(store: &Store<'_>, id: &str) -> std::result::Result<(Note, Vec<String>), ReadFailure> {
    let bytes = store
        .fs()
        .read(&store.resolve_path(id))
        .map_err(|error| (error.to_string(), guidance(&error)))?;
    Note::parse_with_warnings(&bytes).map_err(|error| (error.to_string(), guidance(&error)))
}

fn guidance(error: &Error) -> String {
    match error.kind() {
        ErrorKind::InvalidInput => "salve o arquivo como UTF-8 válido".to_string(),
        ErrorKind::Schema => {
            "valide o frontmatter TOON: `type` deve ser um dos 11 tipos; ajuste ou remova a nota"
                .to_string()
        }
        ErrorKind::NotFound => "restaure o arquivo ou remova a referência".to_string(),
        _ => "revise o frontmatter TOON e o corpo da nota".to_string(),
    }
}
