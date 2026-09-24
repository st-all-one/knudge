//! Testes do bloco gerenciado de `.gitattributes` (E04-T06 / D31).

use std::path::Path;

use crate::Result;
use crate::git::Persistence;
use crate::git::attributes;
use crate::ports::Fs;
use crate::ports::fakes::MemFs;

fn read(fs: &MemFs, path: &Path) -> String {
    String::from_utf8(fs.read(path).unwrap_or_default()).unwrap_or_default()
}

#[test]
fn writes_explicit_rules_for_all_file_types() -> Result<()> {
    let fs = MemFs::new();
    let root = Path::new("/repo");
    fs.create_dir_all(root)?;
    assert!(attributes::apply(&fs, root, Persistence::Versioned)?);
    let text = read(&fs, &root.join(attributes::FILE));

    assert!(text.contains("# knudge:start"));
    assert!(text.contains("# knudge:end"));
    assert!(text.contains("/.knudge/notas/** text eol=lf"));
    assert!(text.contains("/.knudge/config.toml text eol=lf"));
    assert!(text.contains("/.knudge/templates.toml text eol=lf"));
    assert!(text.contains("/.knudge/validators.toml text eol=lf"));
    assert!(text.contains("events*.jsonl text eol=lf merge=union"));
    assert!(text.contains("/.knudge/emb_cache.jsonl text eol=lf merge=union"));
    for pattern in [
        "/.knudge/.idx/**",
        "/.knudge/cache/**",
        "/.knudge/.locks/**",
    ] {
        assert!(text.contains(pattern), "falta regra para {pattern}");
    }
    Ok(())
}

#[test]
fn preserves_user_lines_and_is_idempotent() -> Result<()> {
    let fs = MemFs::new();
    let root = Path::new("/repo");
    fs.create_dir_all(root)?;
    let path = root.join(attributes::FILE);
    fs.write_atomic(&path, b"*.png binary\n")?;

    assert!(attributes::apply(&fs, root, Persistence::Versioned)?);
    let once = read(&fs, &path);
    assert!(once.contains("*.png binary"));
    assert!(once.contains("knudge:start"));

    assert!(!attributes::apply(&fs, root, Persistence::Versioned)?);
    assert_eq!(once, read(&fs, &path));
    Ok(())
}

#[test]
fn local_only_removes_block() -> Result<()> {
    let fs = MemFs::new();
    let root = Path::new("/repo");
    fs.create_dir_all(root)?;
    attributes::apply(&fs, root, Persistence::Versioned)?;

    assert!(attributes::apply(&fs, root, Persistence::LocalOnly)?);
    let text = read(&fs, &root.join(attributes::FILE));
    assert!(!text.contains("knudge:start"));
    assert!(!text.contains("merge=union"));
    Ok(())
}
