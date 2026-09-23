//! Dono derivado de eventos `claim`/`release` (D114).

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::{NoteType, Scope};
use crate::store::events::Event;
use crate::task::{TaskSpec, claim, ownership, submit};
use crate::write::Draft;

use super::context;

#[test]
fn ownership_follows_last_claim_not_released() {
    let events = vec![
        Event::new("claim", 1)
            .with_note_id("task_a")
            .with_actor("agente-a"),
        Event::new("claim", 2)
            .with_note_id("task_a")
            .with_actor("agente-b"),
        Event::new("release", 3).with_note_id("task_a"),
        Event::new("claim", 4)
            .with_note_id("task_a")
            .with_actor("agente-c"),
        Event::new("claim", 5)
            .with_note_id("task_b")
            .with_actor("outro"),
    ];
    assert_eq!(ownership(&events, "task_a").as_deref(), Some("agente-c"));
    assert_eq!(ownership(&events, "task_b").as_deref(), Some("outro"));
    assert_eq!(ownership(&events, "task_c"), None);
}

#[test]
fn close_clears_ownership() {
    let events = vec![
        Event::new("claim", 1)
            .with_note_id("task_a")
            .with_actor("agente-a"),
        Event::new("close", 2).with_note_id("task_a"),
    ];
    assert_eq!(ownership(&events, "task_a"), None);
}

#[test]
fn claim_records_event_and_release_clears_it() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let spec = TaskSpec::new(Scope::Task, "fazer");
    let id = submit(&ctx, &spec)?.id;

    claim(&ctx, &id, Some("agente-a"))?;
    let (events, _warnings) = ctx.events().read_all()?;
    assert_eq!(ownership(&events, &id).as_deref(), Some("agente-a"));

    claim(&ctx, &id, None)?;
    let (events, _warnings) = ctx.events().read_all()?;
    assert_eq!(ownership(&events, &id), None);
    Ok(())
}

#[test]
fn claim_rejects_note_without_scope() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let note = Draft::new(NoteType::Fact, "solto").to_note(super::NOW)?;
    let id = note.id()?.to_string();
    ctx.store().write(&note)?;
    assert!(claim(&ctx, &id, Some("agente-a")).is_err());
    Ok(())
}
