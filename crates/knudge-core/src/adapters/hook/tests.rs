//! Testes do executor de hooks (E12-T04).

use std::time::Duration;

use super::ProcessHookRunner;
use crate::ports::HookRunner;
use crate::{ErrorKind, Result};

fn runner(timeout_ms: u64) -> ProcessHookRunner {
    ProcessHookRunner::new(".", Duration::from_millis(timeout_ms))
}

#[test]
fn captures_stdout_and_stdin_roundtrip() -> Result<()> {
    let output = runner(5_000).run("cat", b"payload")?;
    assert_eq!(output.status, 0);
    assert_eq!(output.stdout, b"payload");
    Ok(())
}

#[test]
fn nonzero_status_is_reported() -> Result<()> {
    let output = runner(5_000).run("false", b"")?;
    assert_ne!(output.status, 0);
    Ok(())
}

#[test]
fn timeout_kills_slow_hook() {
    let error = runner(200).run("sleep 10", b"").err();
    assert_eq!(error.map(|error| error.kind()), Some(ErrorKind::Timeout));
}

#[test]
fn missing_program_is_io_error() {
    let error = runner(1_000).run("definitely-not-a-program-xyz", b"").err();
    assert_eq!(error.map(|error| error.kind()), Some(ErrorKind::Io));
}

#[test]
fn empty_hook_is_config_error() {
    let error = runner(1_000).run("   ", b"").err();
    assert_eq!(error.map(|error| error.kind()), Some(ErrorKind::Config));
}
