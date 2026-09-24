//! Testes do log de eventos: dedup, tolerância, rotação e checkpoint (E03-T04/T09).

use crate::Result;
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use crate::store::{Event, EventLog};

#[test]
fn dedup_on_read_ignores_repeated_lines() -> Result<()> {
    let fs = MemFs::new();
    let log = EventLog::new(&fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    let event = Event::new("write", 5).with_note_id("fact_00000001");
    log.append(&event)?;
    log.append(&event)?;

    let (events, warnings) = log.read_all()?;
    assert_eq!(events.len(), 1, "linhas idênticas devem deduplicar");
    assert!(warnings.is_empty());
    Ok(())
}

#[test]
fn legacy_actor_field_is_read_without_error() -> Result<()> {
    let fs = MemFs::new();
    let log = EventLog::new(&fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    let line =
        br#"{"id":"ev_00000001","op":"write","note_id":"fact_00000001","at":5,"actor":"cli"}"#;
    fs.append(&log.active_path(), line)?;
    fs.append(&log.active_path(), b"\n")?;

    let (events, warnings) = log.read_all()?;
    assert_eq!(events.len(), 1);
    assert!(
        warnings.is_empty(),
        "actor legado não deve gerar aviso: {warnings:?}"
    );
    assert_eq!(events.first().map(|event| event.op.as_str()), Some("write"));
    Ok(())
}

#[test]
fn malformed_line_is_skipped_with_warning() -> Result<()> {
    let fs = MemFs::new();
    let log = EventLog::new(&fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    log.append(&Event::new("write", 7))?;
    fs.append(&log.active_path(), b"{nao json}\n")?;

    let (events, warnings) = log.read_all()?;
    assert_eq!(events.len(), 1);
    assert_eq!(warnings.len(), 1);
    Ok(())
}

#[test]
fn rotates_by_size_and_preserves_order() -> Result<()> {
    let fs = MemFs::new();
    let log = EventLog::new(&fs, "/p/.knudge", 1);
    for index in 0..3 {
        let event = Event::new("write", index).with_note_id(format!("fact_0000000{index}"));
        log.append(&event)?;
    }

    assert!(log.segments()?.len() >= 3);
    let (events, warnings) = log.read_all()?;
    assert!(warnings.is_empty());
    let ats: Vec<i64> = events.iter().map(|event| event.at).collect();
    assert_eq!(ats, vec![0, 1, 2]);
    Ok(())
}

#[test]
fn history_filters_by_note() -> Result<()> {
    let fs = MemFs::new();
    let log = EventLog::new(&fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    log.append(&Event::new("write", 1).with_note_id("fact_a"))?;
    log.append(&Event::new("update", 2).with_note_id("fact_a"))?;
    log.append(&Event::new("write", 3).with_note_id("fact_b"))?;

    let history = log.history("fact_a")?;
    assert_eq!(history.len(), 2);
    assert_eq!(
        history.first().map(|event| event.op.as_str()),
        Some("write")
    );
    Ok(())
}

#[test]
fn checkpoint_reads_only_new_events() -> Result<()> {
    let fs = MemFs::new();
    let log = EventLog::new(&fs, "/p/.knudge", EventLog::DEFAULT_MAX_BYTES);
    log.append(&Event::new("write", 1))?;

    let (all, _warnings) = log.read_all()?;
    let first_id = all.first().map(Event::id).transpose()?;
    let Some(first_id) = first_id else {
        return Ok(());
    };
    log.mark_checkpoint(&first_id)?;

    log.append(&Event::new("write", 2))?;
    let (since, warnings) = log.read_since_checkpoint()?;
    assert!(warnings.is_empty());
    assert_eq!(since.len(), 1);
    assert_eq!(since.first().map(|event| event.at), Some(2));
    Ok(())
}
