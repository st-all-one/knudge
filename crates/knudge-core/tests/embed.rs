//! Teste de incorporação: a fachada `Knudge` aponta para um diretório de conhecimento
//! alternativo (inclusive aninhado), sem tocar o Git.

use std::path::PathBuf;

use knudge_core::adapters::StdFs;
use knudge_core::ports::Fs;
use knudge_core::{Knudge, Result};

type TestResult = Result<()>;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kd-embed-{tag}-{}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&dir);
    let _ignored = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn builder_opens_project_with_nested_layout() -> TestResult {
    let root = temp_dir("nested-layout");
    let knowledge = root.join(".a").join("b");
    let fs = StdFs::new();
    fs.create_dir_all(&knowledge)?;
    fs.write_atomic(
        &knowledge.join("config.toml"),
        b"[behavior]\nstrict = true\n",
    )?;

    let kd = Knudge::builder()
        .root(root.clone())
        .knowledge_dir(".a/b")
        .open()?;
    assert_eq!(kd.knowledge_dir(), knowledge);
    assert_eq!(kd.config().get_bool("behavior.strict"), Some(true));
    assert!(kd.index()?.docs.is_empty());

    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn builder_defaults_to_knudge_layout() -> TestResult {
    let root = temp_dir("default-layout");
    let knowledge = root.join(".knudge");
    let fs = StdFs::new();
    fs.create_dir_all(&knowledge)?;

    let kd = Knudge::builder().root(root.clone()).open()?;
    assert_eq!(kd.knowledge_dir(), knowledge);
    assert_eq!(kd.project().layout_str(), ".knudge");

    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}
