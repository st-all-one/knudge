//! Refs versionadas de handoff (D227): default byte-idêntico, opt-in e verificação advisory.

mod common;

use common::{
    TestResult, expect_code, init, ok, ok_json, run_in, stderr, temp_project, write_note,
};

/// `(id_da_nota, context_id_com_refs, texto_com_refs)`.
type RefsContext = (String, String, String);

/// Contexto com refs e o id da nota citada.
fn with_refs_context(dir: &std::path::Path) -> Result<RefsContext, Box<dyn std::error::Error>> {
    let id = write_note(dir, "nota para handoff", "fact", &["corpo da nota"])?;
    let plain_text = ok(dir, &["rewind", "--scope", &id])?;
    let first = ok_json(dir, &["rewind", "--scope", &id])?;
    let plain_id = first
        .get("context_id")
        .and_then(|value| value.as_str())
        .ok_or("rewind sem context_id")?
        .to_string();
    assert!(
        !plain_text.contains('@'),
        "default não deve conter refs: {plain_text}"
    );
    let resumed = ok(dir, &["rewind", "--resume", &plain_id])?;
    assert_eq!(resumed, plain_text, "D88: resume deve ser byte-idêntico");

    let refs_text = ok(dir, &["rewind", "--scope", &id, "--with-refs"])?;
    let rich = ok_json(dir, &["rewind", "--scope", &id, "--with-refs"])?;
    let refs_id = rich
        .get("context_id")
        .and_then(|value| value.as_str())
        .ok_or("rewind --with-refs sem context_id")?
        .to_string();
    assert!(refs_text.contains('@'), "refs ausentes: {refs_text}");
    assert_ne!(refs_id, plain_id, "refs devem mudar o context_id derivado");
    Ok((id, refs_id, refs_text))
}

#[test]
fn resume_default_e_byte_identico_e_refs_sao_opt_in() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let _context = with_refs_context(&dir)?;
    Ok(())
}

#[test]
fn doctor_refs_e_silencioso_quando_nada_mudou() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let (_id, context_id, _text) = with_refs_context(&dir)?;
    let out = run_in(&dir, &["doctor", "--refs", &context_id])?;
    assert!(
        out.status.success(),
        "doctor --refs falhou: {}",
        stderr(&out)?
    );
    let message = stderr(&out)?;
    assert!(
        !message.contains("changed") && !message.contains("dangling"),
        "stderr devia estar limpo: {message}"
    );
    Ok(())
}

#[test]
fn doctor_refs_reporta_changed_apos_update_e_resume_nao_muda() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let (id, context_id, refs_text) = with_refs_context(&dir)?;
    ok(&dir, &["write", "--update", &id, "corpo alterado"])?;

    let out = run_in(&dir, &["doctor", "--refs", &context_id])?;
    assert!(
        out.status.success(),
        "doctor --refs falhou: {}",
        stderr(&out)?
    );
    let message = stderr(&out)?;
    assert!(
        message.contains("changed"),
        "mudança não reportada: {message}"
    );

    let resumed = run_in(&dir, &["rewind", "--resume", &context_id, "--check-refs"])?;
    assert!(
        resumed.status.success(),
        "check-refs falhou: {}",
        stderr(&resumed)?
    );
    let message = stderr(&resumed)?;
    assert!(
        message.contains("changed"),
        "mudança não reportada: {message}"
    );
    assert_eq!(String::from_utf8(resumed.stdout)?, refs_text);
    Ok(())
}

#[test]
fn doctor_refs_rejeita_context_id_invalido() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    expect_code(&dir, &["doctor", "--refs", "ctx_curto"], 2)?;
    expect_code(&dir, &["doctor", "--refs", "ctx_aaaaaaaa"], 3)?;
    Ok(())
}
