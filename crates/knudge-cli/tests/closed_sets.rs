//! Conjuntos fechados (D212): valor inválido lista as possibilidades e sugere a mais provável;
//! flag ausente não valida nada.

mod common;

use common::{TestResult, expect_code, init, run_in, stderr, temp_project};

#[test]
fn invalid_enum_lists_options_and_suggests() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let out = run_in(&dir, &["ask", "cache", "--status", "activee"])?;
    assert_eq!(out.status.code(), Some(8), "{}", stderr(&out)?);
    let message = stderr(&out)?;
    assert!(message.contains("use: active"), "{message}");
    assert!(message.contains("você quis dizer \"active\""), "{message}");
    Ok(())
}

#[test]
fn invalid_edge_lists_the_twelve_kinds() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let out = run_in(&dir, &["write", "--link", "a:extnds:b"])?;
    assert_eq!(out.status.code(), Some(8), "{}", stderr(&out)?);
    let message = stderr(&out)?;
    assert!(message.contains("results_in"), "{message}");
    assert!(message.contains("você quis dizer \"extends\""), "{message}");
    Ok(())
}

#[test]
fn invalid_log_level_is_rejected() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    expect_code(&dir, &["--log-level", "bogus", "ask", "cache"], 2)?;
    Ok(())
}

#[test]
fn invalid_config_key_suggests_the_closest() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let out = run_in(&dir, &["config", "get", "--key", "recall.default_limitx"])?;
    assert_eq!(out.status.code(), Some(3), "{}", stderr(&out)?);
    assert!(stderr(&out)?.contains("recall.default_limit"));
    Ok(())
}

#[test]
fn absent_enum_flag_is_not_validated() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    // Sem a flag, não há erro nem lista (item 3).
    let out = run_in(&dir, &["ask", "cache"])?;
    assert!(out.status.success(), "{}", stderr(&out)?);
    Ok(())
}
