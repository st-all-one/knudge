//! Testes da purga de inativos com histórico (E10-T03).

use crate::Result;
use crate::lifecycle::Retirement;
use crate::lifecycle::retire::{Retention, due_for_purge, purge_due, retirements};
use crate::lifecycle::shelf_life::DAY_MS;
use crate::ports::fakes::MemFs;
use crate::schema::{NoteType, Value};
use crate::store::Event;
use crate::write::forget;

use super::{NOW, note_created, seeded};

fn days(n: i64) -> i64 {
    DAY_MS.saturating_mul(n)
}

#[test]
fn retirements_keep_latest_per_note() {
    let events = vec![
        Event::new("forget", 100)
            .with_note_id("a")
            .with_data("reason", Value::Str("anchor_decay".to_string())),
        Event::new("forget", 200).with_note_id("a"),
        Event::new("supersede", 150).with_note_id("b"),
        Event::new("write", 300).with_note_id("c"),
    ];
    let derived = retirements(&events);
    assert_eq!(derived.len(), 2);
    assert_eq!(derived.first().map(|r| r.id.as_str()), Some("a"));
    assert_eq!(derived.first().map(|r| r.retired_at), Some(200));
    assert_eq!(derived.first().map(|r| r.reason.as_str()), Some("forget"));
    assert_eq!(derived.get(1).map(|r| r.id.as_str()), Some("b"));
    assert_eq!(derived.get(1).map(|r| r.reason.as_str()), Some("supersede"));
}

#[test]
fn due_for_purge_respects_window() {
    let retention = Retention { retired_days: 30 };
    let retired = vec![Retirement {
        id: "a".to_string(),
        retired_at: 0,
        reason: "forget".to_string(),
    }];
    assert!(due_for_purge(&retired, days(29), retention).is_empty());
    assert_eq!(due_for_purge(&retired, days(30), retention), ["a"]);
}

#[test]
fn purge_removes_content_only_after_window() -> Result<()> {
    let fs = MemFs::new();
    let note = note_created(NoteType::Fact, "efêmera", "", NOW)?;
    let id = note.id()?.to_string();
    let ctx = seeded(&fs, &[note], NOW)?;
    forget(&ctx, &id, Some("shelf_life"))?;
    assert!(ctx.store().exists(&id));

    let retention = Retention { retired_days: 30 };
    let too_soon = purge_due(
        ctx.store(),
        ctx.events(),
        NOW.saturating_add(days(10)),
        retention,
    )?;
    assert!(too_soon.is_empty());
    assert!(ctx.store().exists(&id));

    let purged = purge_due(
        ctx.store(),
        ctx.events(),
        NOW.saturating_add(days(31)),
        retention,
    )?;
    assert_eq!(purged.first().map(String::as_str), Some(id.as_str()));
    assert!(!ctx.store().exists(&id));
    Ok(())
}
