//! Stress de concorrência determinístico sobre os adaptadores reais (E13-T03).
//!
//! Verifica que: (a) o lock advisory impede *lost updates* no mesmo alvo; (b) escritas
//! concorrentes de notas distintas não se perdem; (c) um leitor do índice derivado nunca vê
//! estado parcial durante um rebuild (double-buffer).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use knudge_core::adapters::{StdFs, SystemClock, ThreadRng};
use knudge_core::ports::Fs;
use knudge_core::schema::NoteType;
use knudge_core::store::{LockPolicy, Staging, Store, acquire};
use knudge_core::write::Draft;
use knudge_core::{Error, Result};

type TestResult = Result<()>;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kd-stress-{tag}-{}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&dir);
    let _ignored = std::fs::create_dir_all(&dir);
    dir
}

fn read_u64(fs: &dyn Fs, path: &Path) -> Result<u64> {
    let bytes = fs.read(path)?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(text.trim().parse::<u64>().unwrap_or(0))
}

#[test]
fn lock_prevents_lost_updates_on_real_fs() -> TestResult {
    const WORKERS: u64 = 8;
    const ITERATIONS: u64 = 25;

    let dir = temp_dir("lock");
    let fs = StdFs::new();
    let counter = dir.join("counter");
    let lock = dir.join(".locks").join("counter.lock");
    fs.write_atomic(&counter, b"0")?;

    let policy = LockPolicy {
        stale_ms: 30_000,
        retries: 1_000_000,
        jitter_ms: 0,
    };

    thread::scope(|scope| -> TestResult {
        let mut handles = Vec::new();
        for _ in 0..WORKERS {
            let fs = &fs;
            let counter = counter.clone();
            let lock = lock.clone();
            handles.push(scope.spawn(move || -> TestResult {
                let clock = SystemClock::new();
                let mut rng = ThreadRng::new();
                for _ in 0..ITERATIONS {
                    let _guard = acquire(fs, &clock, &mut rng, &lock, policy)?;
                    let current = read_u64(fs, &counter)?;
                    fs.write_atomic(&counter, current.saturating_add(1).to_string().as_bytes())?;
                }
                Ok(())
            }));
        }
        for handle in handles {
            handle
                .join()
                .map_err(|_| Error::internal("worker do stress panicou"))??;
        }
        Ok(())
    })?;

    assert_eq!(read_u64(&fs, &counter)?, WORKERS * ITERATIONS);
    Ok(())
}

#[test]
fn concurrent_distinct_writes_all_land() -> TestResult {
    const WRITERS: u64 = 12;
    const PER_WRITER: u64 = 8;

    let dir = temp_dir("writes");
    let fs = StdFs::new();
    let root = dir.join(".knudge");
    let store = Store::new(&fs, root);
    store.ensure_dirs()?;

    thread::scope(|scope| -> TestResult {
        let mut handles = Vec::new();
        for writer in 0..WRITERS {
            let store = &store;
            handles.push(scope.spawn(move || -> TestResult {
                for index in 0..PER_WRITER {
                    let statement = format!("nota {writer}-{index}");
                    let note = Draft::new(NoteType::Fact, statement).to_note(0)?;
                    store.write(&note)?;
                }
                Ok(())
            }));
        }
        for handle in handles {
            handle
                .join()
                .map_err(|_| Error::internal("writer do stress panicou"))??;
        }
        Ok(())
    })?;

    let ids = store.list_ids()?;
    assert_eq!(
        ids.len(),
        usize::try_from(WRITERS * PER_WRITER).unwrap_or(usize::MAX)
    );
    Ok(())
}

#[test]
fn reader_never_sees_partial_index() -> TestResult {
    let dir = temp_dir("rebuild");
    let fs = StdFs::new();
    let index_dir = dir.join(".idx");
    fs.create_dir_all(&index_dir)?;
    let file = index_dir.join("retrieval.jsonl");
    fs.write_atomic(&file, b"v1\n")?;

    let stop = Arc::new(AtomicBool::new(false));
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));

    thread::scope(|scope| -> TestResult {
        let reader = {
            let fs = &fs;
            let file = file.clone();
            let stop = Arc::clone(&stop);
            let seen = Arc::clone(&seen);
            scope.spawn(move || -> TestResult {
                while !stop.load(Ordering::Relaxed) {
                    if let Ok(bytes) = fs.read(&file) {
                        let text = String::from_utf8_lossy(&bytes).into_owned();
                        assert!(
                            text == "v1\n" || text == "v2\n",
                            "leitor viu estado parcial: {text:?}"
                        );
                        if let Ok(mut guard) = seen.lock() {
                            guard.push(text);
                        }
                    }
                }
                Ok(())
            })
        };

        for _ in 0..50 {
            let staging = Staging::begin(&fs, &index_dir)?;
            staging.write("retrieval.jsonl", b"v2\n")?;
            staging.commit()?;
        }
        stop.store(true, Ordering::Relaxed);
        reader
            .join()
            .map_err(|_| Error::internal("reader do stress panicou"))??;
        Ok(())
    })?;

    let observed = seen.lock().map(|guard| guard.clone()).unwrap_or_default();
    assert!(!observed.is_empty(), "o leitor deveria ter lido o índice");
    assert!(observed.iter().all(|text| text == "v1\n" || text == "v2\n"));
    Ok(())
}
