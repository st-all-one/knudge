//! Uso real: corpus legado (layout plano + `type: container` + `scope: plan`) e migração
//! via `doctor --fix` (D134/D149/D150).
//!
//! Reproduz o formato do `TMP/lotep-master-.knudge` (526 notas planas) numa escala pequena:
//! antes do fix a leitura precisa **tolerar** as notas inválidas; depois do fix o layout e o
//! schema ficam canônicos.

mod common;

use std::path::Path;

use common::{TestResult, data, init, run_in, stderr, stdout, temp_project};

const LEGACY_EPIC: &str = "\
---
id: container_00000001
type: container
statement: \"Épico legado do portal\"
created_at: 2026-01-01T00:00:00.000Z
confidence: 0.7
body_hash: 00000000
schema_version: 1
tags: [legado, portal]
source: legacy.jsonl
scope: plan
status: active
---
Corpo do épico legado.
";

const LEGACY_TASK: &str = "\
---
id: task_00000002
type: task
statement: \"Tarefa legada do portal\"
created_at: 2026-01-02T00:00:00.000Z
confidence: 0.7
body_hash: 00000000
schema_version: 1
scope: issue
status: active
---
Corpo da tarefa legada.
";

const LEGACY_FACT: &str = "\
---
id: fact_00000003
type: fact
statement: \"Fato legado do portal\"
created_at: 2026-01-03T00:00:00.000Z
confidence: 0.7
body_hash: 00000000
schema_version: 1
tags: [portal]
source: legacy.jsonl
---
Corpo do fato legado.
";

/// Grava as três notas no layout plano antigo (`notas/<id>.md`).
fn seed_legacy(dir: &Path) -> TestResult {
    let notas = dir.join(".knudge").join("notas");
    std::fs::create_dir_all(&notas)?;
    for (name, content) in [
        ("container_00000001.md", LEGACY_EPIC),
        ("task_00000002.md", LEGACY_TASK),
        ("fact_00000003.md", LEGACY_FACT),
    ] {
        std::fs::write(notas.join(name), content)?;
    }
    Ok(())
}

#[test]
fn legacy_flat_corpus_is_readable_before_fix() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    seed_legacy(&dir)?;

    // `ask` ignora a nota `container` (tipo desconhecido) sem derrubar a busca.
    let ask = run_in(&dir, &["ask", "legado"])?;
    assert!(ask.status.success(), "ask: {}", stderr(&ask)?);
    let text = stdout(&ask)?;
    assert!(
        text.contains("fact_00000003"),
        "fato legado ausente: {text}"
    );

    // `task list` lê a tarefa legada (e não a nota `container`).
    let list = run_in(&dir, &["task", "list", "--universe"])?;
    assert!(list.status.success(), "task list: {}", stderr(&list)?);
    assert!(stdout(&list)?.contains("task_00000002"));

    // `rewind` monta o handoff sem abortar.
    let rewind = run_in(&dir, &["rewind", "--budget", "500"])?;
    assert!(rewind.status.success(), "rewind: {}", stderr(&rewind)?);
    assert!(stdout(&rewind)?.contains("notes="));

    // `doctor` reporta as notas puladas (o fix ainda não rodou).
    let doctor = run_in(&dir, &["--json", "maintenance", "doctor"])?;
    assert!(doctor.status.success());
    let checks = data(&doctor)?;
    assert_eq!(check_ok(&checks, "schema"), Some(false));
    Ok(())
}

#[test]
fn doctor_fix_migrates_legacy_corpus() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    seed_legacy(&dir)?;

    let fix = run_in(&dir, &["maintenance", "doctor", "--fix"])?;
    assert!(fix.status.success(), "doctor --fix: {}", stderr(&fix)?);

    // Layout migrado para `notas/<tipo>/<id>.md`.
    let notas = dir.join(".knudge").join("notas");
    assert!(notas.join("epic").join("container_00000001.md").exists());
    assert!(notas.join("task").join("task_00000002.md").exists());
    assert!(notas.join("fact").join("fact_00000003.md").exists());
    assert!(!notas.join("container_00000001.md").exists());

    // Schema normalizado: `type: container` e `scope: plan` saem; `confidence` sai.
    let epic = std::fs::read_to_string(notas.join("epic").join("container_00000001.md"))?;
    assert!(!epic.contains("type: container"), "type legado permaneceu");
    assert!(!epic.contains("scope: plan"), "scope legado permaneceu");
    assert!(epic.contains("scope: epic"), "scope não migrou para epic");
    assert!(!epic.contains("confidence"), "chave removida permaneceu");

    // Depois do fix, o corpus está saudável e a leitura segue funcionando.
    let doctor = run_in(&dir, &["--json", "maintenance", "doctor"])?;
    let checks = data(&doctor)?;
    assert_eq!(check_ok(&checks, "schema"), Some(true));
    assert_eq!(check_ok(&checks, "body_hash"), Some(true));

    let ask = run_in(&dir, &["ask", "legado"])?;
    assert!(ask.status.success(), "ask pós-fix: {}", stderr(&ask)?);
    assert!(stdout(&ask)?.contains("fact_00000003"));
    Ok(())
}

/// Smoke no corpus real do `TMP` quando ele existe (ignorado no CI sem o corpus).
#[test]
fn real_tmp_corpus_smoke() -> TestResult {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = manifest
        .join("..")
        .join("..")
        .join("TMP")
        .join("lotep-master-.knudge")
        .join(".knudge");
    if !source.join("notas").is_dir() {
        return Ok(());
    }

    let dir = temp_project();
    copy_tree(&source, &dir.join(".knudge"))?;

    // Antes do fix: leitura tolerante não derruba.
    let ask = run_in(&dir, &["--json", "ask", "deploy"])?;
    assert!(
        ask.status.success(),
        "ask no corpus real: {}",
        stderr(&ask)?
    );
    let _ignored = data(&ask)?;

    let fix = run_in(&dir, &["maintenance", "doctor", "--fix"])?;
    assert!(fix.status.success(), "doctor --fix real: {}", stderr(&fix)?);

    // Depois do fix: nenhuma nota legada restou e os verbos principais respondem.
    let notas = dir.join(".knudge").join("notas");
    assert!(!notas.join("container_00000001.md").exists());
    assert_eq!(count_flat_notes(&notas)?, 0, "notas planas restaram");

    for args in [
        vec!["ask", "home"],
        vec!["task", "list", "--universe"],
        vec!["rewind", "--budget", "500"],
        vec!["knowledge", "map", "--universe", "--axis", "type"],
        vec!["knowledge", "rank", "--universe", "--limit", "5"],
    ] {
        let out = run_in(&dir, &args)?;
        assert!(out.status.success(), "{args:?}: {}", stderr(&out)?);
    }

    let doctor = run_in(&dir, &["--json", "maintenance", "doctor"])?;
    let checks = data(&doctor)?;
    assert_eq!(check_ok(&checks, "schema"), Some(true));
    Ok(())
}

/// `ok` de um check do `doctor` (`None` se ausente).
fn check_ok(checks: &serde_json::Value, id: &str) -> Option<bool> {
    checks
        .get("checks")
        .and_then(|value| value.as_array())
        .and_then(|checks| {
            checks
                .iter()
                .find(|check| check.get("id").and_then(serde_json::Value::as_str) == Some(id))
        })
        .and_then(|check| check.get("ok"))
        .and_then(serde_json::Value::as_bool)
}

/// Copia recursivamente um diretório (substituto de `cp -r` sem dependências).
fn copy_tree(source: &Path, target: &Path) -> TestResult {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let to = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

/// Conta arquivos `.md` na raiz de `notas/` (layout plano legado).
fn count_flat_notes(notas: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count: usize = 0;
    for entry in std::fs::read_dir(notas)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(|e| e.to_str()) == Some("md")
        {
            count = count.saturating_add(1);
        }
    }
    Ok(count)
}
