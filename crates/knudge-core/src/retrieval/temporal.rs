//! Consulta temporal `as_of` — conjunto ativo num instante (D155).
//!
//! Reconstrói o corpus **ativo em `T`** a partir do log de eventos (auditoria canônica,
//! D33): `forget`/`restore` (soft-delete, D52) e `link replaces` (supersessão, D46) mudam o
//! estado; eventos posteriores a `T` são ignorados. O conteúdo da nota é o **atual** — o
//! knudge não versiona edição in-place (`update`); só a supersessão cria uma nova nota. Essa é
//! a borda honesta documentada em `DIVERGENCES.md`.

use std::collections::BTreeSet;

use crate::schema::Value;
use crate::store::Event;

/// Estado reconstruído de uma nota num instante.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Existia e estava ativa.
    Active,
    /// Havia sido esquecida (`forget`) sem `restore`.
    Forgotten,
    /// Havia sido substituída (`link replaces` mirando nela).
    Superseded,
}

/// Reconstrói o estado de `id` em `as_of_ms` (eventos com `at > as_of_ms` são ignorados).
#[must_use]
pub fn state_at(events: &[Event], id: &str, as_of_ms: i64) -> State {
    let mut forgotten = false;
    let mut superseded = false;
    for event in events {
        if event.at > as_of_ms {
            continue;
        }
        match event.op.as_str() {
            "forget" if event.note_id.as_deref() == Some(id) => forgotten = true,
            "restore" if event.note_id.as_deref() == Some(id) => forgotten = false,
            "link" if supersedes(event, id) => superseded = true,
            _ => {}
        }
    }
    if superseded {
        State::Superseded
    } else if forgotten {
        State::Forgotten
    } else {
        State::Active
    }
}

/// `true` se o evento `link` aponta `replaces` para `target`.
fn supersedes(event: &Event, target: &str) -> bool {
    event.data.get("kind").and_then(Value::as_str) == Some("replaces")
        && event.data.get("to").and_then(Value::as_str) == Some(target)
}

/// Ids ativos em `as_of_ms`.
///
/// `docs` é o par `(id, created_ms)` de cada nota do índice atual. Uma nota entra no conjunto
/// se já existia (`created_ms <= as_of_ms`) e o estado reconstruído é [`State::Active`].
#[must_use]
pub fn active_ids(events: &[Event], docs: &[(String, i64)], as_of_ms: i64) -> BTreeSet<String> {
    docs.iter()
        .filter(|(_, created_ms)| *created_ms <= as_of_ms)
        .filter(|(id, _)| state_at(events, id, as_of_ms) == State::Active)
        .map(|(id, _)| id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(op: &str, id: &str, at: i64) -> Event {
        Event::new(op, at).with_note_id(id)
    }

    fn replace(from: &str, to: &str, at: i64) -> Event {
        Event::new("link", at)
            .with_note_id(from)
            .with_data("kind", Value::Str("replaces".to_string()))
            .with_data("to", Value::Str(to.to_string()))
    }

    #[test]
    fn new_note_is_inactive_before_creation() {
        let events = vec![ev("write", "a", 100)];
        assert_eq!(state_at(&events, "a", 50), State::Active);
        let docs = vec![("a".to_string(), 100)];
        assert!(active_ids(&events, &docs, 50).is_empty());
        assert_eq!(active_ids(&events, &docs, 150).len(), 1);
    }

    #[test]
    fn forget_then_restore_is_time_ordered() {
        let events = vec![
            ev("write", "a", 10),
            ev("forget", "a", 20),
            ev("restore", "a", 30),
        ];
        assert_eq!(state_at(&events, "a", 15), State::Active);
        assert_eq!(state_at(&events, "a", 25), State::Forgotten);
        assert_eq!(state_at(&events, "a", 35), State::Active);
    }

    #[test]
    fn supersession_only_hides_after_its_instant() {
        let events = vec![
            ev("write", "a", 10),
            ev("write", "b", 50),
            replace("b", "a", 50),
        ];
        assert_eq!(state_at(&events, "a", 40), State::Active);
        assert_eq!(state_at(&events, "a", 60), State::Superseded);
        let docs = vec![("a".to_string(), 10), ("b".to_string(), 50)];
        let active = active_ids(&events, &docs, 40);
        assert!(active.contains("a"));
        assert!(!active.contains("b"));
    }
}
