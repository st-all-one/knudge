//! Testes do lock advisory e do reclaim de stale (E03-T03/T08).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::Error;
use crate::Result;
use crate::ports::Fs;
use crate::ports::fakes::{FixedClock, MemFs, SeqRng};
use crate::store::{LockPolicy, acquire};
use crate::time::Timestamp;

fn policy() -> LockPolicy {
    LockPolicy {
        stale_ms: 30_000,
        retries: 2,
        jitter_ms: 0,
    }
}

#[test]
fn lock_excludes_and_releases_on_drop() -> Result<()> {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(1_000));
    let mut rng = SeqRng::new(1);
    let path = PathBuf::from("/p/.knudge/.locks/fact_a.lock");

    let guard = acquire(&fs, &clock, &mut rng, &path, policy())?;
    assert!(fs.exists(&path));

    let blocked = acquire(&fs, &clock, &mut rng, &path, policy());
    assert!(matches!(blocked, Err(Error::Conflict(_))));

    drop(guard);
    assert!(!fs.exists(&path), "o Drop deve liberar o lock");

    let _again = acquire(&fs, &clock, &mut rng, &path, policy())?;
    Ok(())
}

#[test]
fn stale_lock_is_reclaimed() -> Result<()> {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(100_000));
    let mut rng = SeqRng::new(7);
    let path = PathBuf::from("/p/.knudge/.locks/fact_a.lock");
    fs.create_exclusive(&path, b"{\"at\":1}")?;

    let guard = acquire(&fs, &clock, &mut rng, &path, policy())?;
    assert!(fs.exists(&path), "o lock deve ter sido reclamado");
    drop(guard);
    Ok(())
}

#[test]
fn fresh_unreadable_lock_is_not_reclaimed() -> Result<()> {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(1_000));
    let mut rng = SeqRng::new(3);
    let path = PathBuf::from("/p/.knudge/.locks/fact_a.lock");
    // Janela entre `create_exclusive` e a escrita do `at`: arquivo existe e vazio.
    fs.create_exclusive(&path, b"")?;

    let blocked = acquire(&fs, &clock, &mut rng, &path, policy());
    assert!(
        matches!(blocked, Err(Error::Conflict(_))),
        "um lock recém-criado não pode ser roubado (E13-T03)"
    );
    Ok(())
}

#[test]
fn concurrent_lock_prevents_lost_updates() -> Result<()> {
    let fs = Arc::new(MemFs::new());
    let counter = "/p/.knudge/counter";
    fs.insert(counter, b"0");
    let lock_path = PathBuf::from("/p/.knudge/.locks/counter.lock");
    let policy = LockPolicy {
        stale_ms: 30_000,
        retries: 1_000_000,
        jitter_ms: 0,
    };

    let threads: Vec<_> = (1_u64..5)
        .map(|seed| {
            let fs = Arc::clone(&fs);
            let path = lock_path.clone();
            std::thread::spawn(move || -> Result<()> {
                let clock = FixedClock::new(Timestamp::from_millis(1));
                let mut rng = SeqRng::new(seed);
                let _guard = acquire(&*fs, &clock, &mut rng, &path, policy)?;
                let current = read_counter(fs.read(Path::new(counter))?);
                let next = current.saturating_add(1);
                fs.write_atomic(Path::new(counter), next.to_string().as_bytes())
            })
        })
        .collect();

    for thread in threads {
        thread
            .join()
            .map_err(|_| Error::internal("thread do teste panicou"))??;
    }
    assert_eq!(read_counter(fs.read(Path::new(counter))?), 4);
    Ok(())
}

fn read_counter(bytes: Vec<u8>) -> u32 {
    String::from_utf8(bytes)
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(0)
}
