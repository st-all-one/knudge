//! Regressão da padronização de listas (D210): toda flag de seleção aceita **repetição**
//! (`--tag a --tag b`) e **lista com vírgula** (`--tag a,b`), de forma equivalente; o formato
//! por espaço é rejeitado, e os modos `--id`/`--around` não aceitam query textual.

mod common;

use std::path::Path;

use common::{TestResult, data, expect_code, init, run_in, stderr, temp_project, write_note};

/// Ids dos `hits` de uma busca `--json`, ordenados.
fn hits_ids(dir: &Path, args: &[&str]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut full = vec!["--json"];
    full.extend_from_slice(args);
    let out = run_in(dir, &full)?;
    assert!(out.status.success(), "{args:?}: {}", stderr(&out)?);
    let hits = data(&out)?
        .get("hits")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let mut ids: Vec<String> = hits
        .iter()
        .filter_map(|hit| hit.get("id").and_then(|value| value.as_str()))
        .map(str::to_string)
        .collect();
    ids.sort();
    Ok(ids)
}

/// Ids das `notes` de um `ask --id` (`--json`), ordenados.
fn notes_ids(dir: &Path, args: &[&str]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut full = vec!["--json"];
    full.extend_from_slice(args);
    let out = run_in(dir, &full)?;
    assert!(out.status.success(), "{args:?}: {}", stderr(&out)?);
    let notes = data(&out)?
        .get("notes")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let mut ids: Vec<String> = notes
        .iter()
        .filter_map(|note| note.get("id").and_then(|value| value.as_str()))
        .map(str::to_string)
        .collect();
    ids.sort();
    Ok(ids)
}

/// Corpus com duas notas de tags/espécies distintas.
fn seed(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    init(dir)?;
    let _alpha = write_note(dir, "Alpha cache", "fact", &["--tag", "a"])?;
    let _beta = write_note(dir, "Beta cache", "decision", &["--tag", "b"])?;
    Ok(())
}

#[test]
fn comma_and_repeat_are_equivalent_for_tag() -> TestResult {
    let dir = temp_project();
    seed(&dir)?;
    let comma = hits_ids(&dir, &["ask", "cache", "--tag", "a,b"])?;
    let repeat = hits_ids(&dir, &["ask", "cache", "--tag", "a", "--tag", "b"])?;
    assert_eq!(
        comma, repeat,
        "`--tag a,b` deve equivaler a `--tag a --tag b`"
    );
    assert_eq!(comma.len(), 2, "as duas tags devem casar as duas notas");
    Ok(())
}

#[test]
fn comma_and_repeat_are_equivalent_for_type_and_class() -> TestResult {
    let dir = temp_project();
    seed(&dir)?;
    let comma = hits_ids(&dir, &["ask", "cache", "--type", "fact,decision"])?;
    let repeat = hits_ids(
        &dir,
        &["ask", "cache", "--type", "fact", "--type", "decision"],
    )?;
    assert_eq!(
        comma, repeat,
        "`--type a,b` deve equivaler a `--type a --type b`"
    );
    let class_comma = hits_ids(&dir, &["ask", "cache", "--class", "tactical,observational"])?;
    let class_repeat = hits_ids(
        &dir,
        &[
            "ask",
            "cache",
            "--class",
            "tactical",
            "--class",
            "observational",
        ],
    )?;
    assert_eq!(
        class_comma, class_repeat,
        "`--class a,b` deve equivaler a `--class a --class b`"
    );
    Ok(())
}

#[test]
fn comma_and_repeat_are_equivalent_for_ids() -> TestResult {
    let dir = temp_project();
    seed(&dir)?;
    let all = hits_ids(&dir, &["ask", "cache"])?;
    let (Some(first), Some(second)) = (all.first(), all.get(1)) else {
        return Err("corpus sem dois hits".into());
    };
    let joined = format!("{first},{second}");
    let comma = notes_ids(&dir, &["ask", "--id", joined.as_str()])?;
    let repeat = notes_ids(
        &dir,
        &["ask", "--id", first.as_str(), "--id", second.as_str()],
    )?;
    assert_eq!(comma, repeat, "`--id a,b` deve equivaler a `--id a --id b`");
    assert_eq!(comma.len(), 2);
    Ok(())
}

#[test]
fn space_separated_list_is_rejected() -> TestResult {
    let dir = temp_project();
    seed(&dir)?;
    // `--id a b`: o `b` seria a query; agora é conflito declarado (não descarte silencioso).
    expect_code(&dir, &["ask", "--id", "x", "y"], 2)?;
    expect_code(&dir, &["ask", "--around", "x", "y"], 2)?;
    Ok(())
}

#[test]
fn params_with_id_and_query_is_rejected() -> TestResult {
    let dir = temp_project();
    seed(&dir)?;
    expect_code(
        &dir,
        &["ask", "--params", r#"{"id":["x","y"],"query":"q"}"#],
        2,
    )?;
    Ok(())
}

#[test]
fn positional_query_keeps_its_commas() -> TestResult {
    let dir = temp_project();
    seed(&dir)?;
    let out = run_in(&dir, &["--json", "ask", "a,b"])?;
    assert!(out.status.success(), "{}", stderr(&out)?);
    let query = data(&out)?
        .get("query")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .unwrap_or_default();
    assert_eq!(query, "a,b", "a query textual não é dividida por vírgula");
    Ok(())
}

/// Cria uma tarefa simples e devolve o id.
fn task_id(dir: &Path, summary: &str) -> Result<String, Box<dyn std::error::Error>> {
    let out = run_in(
        dir,
        &[
            "--json",
            "task",
            "new",
            "--summary",
            summary,
            "--scope",
            "task",
        ],
    )?;
    assert!(out.status.success(), "{}", stderr(&out)?);
    data(&out)?
        .get("id")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| "task new sem id".into())
}

#[test]
fn task_show_accepts_comma_and_repeat_but_not_space() -> TestResult {
    let dir = temp_project();
    init(&dir)?;
    let first = task_id(&dir, "T1")?;
    let second = task_id(&dir, "T2")?;

    let joined = format!("{first},{second}");
    let comma = run_in(&dir, &["--json", "task", "show", "--id", joined.as_str()])?;
    assert!(comma.status.success(), "{}", stderr(&comma)?);
    let repeat = run_in(
        &dir,
        &[
            "--json",
            "task",
            "show",
            "--id",
            first.as_str(),
            "--id",
            second.as_str(),
        ],
    )?;
    assert!(repeat.status.success(), "{}", stderr(&repeat)?);
    assert_eq!(
        String::from_utf8(comma.stdout)?,
        String::from_utf8(repeat.stdout)?,
        "`task show --id a,b` deve equivaler a `--id a --id b`"
    );
    // O formato por espaço deixa de existir (não consome o próximo token).
    expect_code(
        &dir,
        &["task", "show", "--id", first.as_str(), second.as_str()],
        2,
    )?;
    Ok(())
}
