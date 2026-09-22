//! Testes dos fakes das portas.

use super::*;

#[test]
fn fixed_clock_and_seq_rng_are_deterministic() {
    let clock = FixedClock::new(Timestamp::from_millis(42));
    assert_eq!(clock.now().as_millis(), 42);
    clock.set(Timestamp::from_millis(43));
    assert_eq!(clock.now().as_millis(), 43);

    let mut a = SeqRng::new(7);
    let mut b = SeqRng::new(7);
    assert_eq!(a.next_u64(), b.next_u64());
}

#[test]
fn memfs_round_trips() -> Result<()> {
    let fs = MemFs::new();
    fs.write_atomic(Path::new("/n/a.md"), b"x")?;
    assert_eq!(fs.read(Path::new("/n/a.md"))?, b"x");
    assert!(fs.exists(Path::new("/n/a.md")));
    assert_eq!(fs.list_dir(Path::new("/n"))?.len(), 1);
    fs.remove_file(Path::new("/n/a.md"))?;
    assert!(!fs.exists(Path::new("/n/a.md")));
    Ok(())
}

#[test]
fn recording_logger_redacts_secrets() {
    let logger = RecordingLogger::new(Redactor::new(["s3cr3t".to_string()]));
    logger.log(&LogRecord {
        level: Level::Info,
        message: "valor s3cr3t aqui",
        fields: &[("token", "s3cr3t")],
    });
    let records = logger.records();
    assert_eq!(records.len(), 1);
    let Some(record) = records.first() else {
        return;
    };
    assert!(!record.message.contains("s3cr3t"));
    assert!(!record.fields.iter().any(|(_, v)| v.contains("s3cr3t")));
}
