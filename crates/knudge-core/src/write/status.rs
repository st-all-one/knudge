//! Transições de `status` permitidas (D52).
//!
//! O ciclo de vida é **soft**: `forget`/`restore` mudam o estado, nunca apagam a nota. Alguns
//! estados são protegidos — `superseded` só é atribuído por [`crate::write::update`] (nunca à
//! mão) e uma nota `forgotten` só volta via `restore` para `active`.

use crate::schema::Status;
use crate::{Error, Result};

/// Valida uma transição `from → to`.
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` para transições proibidas (estado terminal ou
/// `superseded` manual).
pub fn validate_transition(from: Status, to: Status) -> Result<()> {
    if from == to {
        return Ok(());
    }
    if to == Status::Superseded {
        return Err(Error::invalid_input(
            "status `superseded` só é atribuído por supersede",
        ));
    }
    match from {
        Status::Superseded if to == Status::Forgotten => Ok(()),
        Status::Superseded => Err(Error::invalid_input(
            "nota `superseded` só pode virar `forgotten`",
        )),
        Status::Forgotten if to == Status::Active => Ok(()),
        Status::Forgotten => Err(Error::invalid_input(
            "nota `forgotten` só pode ser restaurada para `active`",
        )),
        _ => Ok(()),
    }
}
