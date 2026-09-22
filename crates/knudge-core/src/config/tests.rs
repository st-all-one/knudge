//! Testes de configuração em dois níveis (E04-T01).

use std::path::Path;

use super::{Config, ConfigValue, global_config_path};
use crate::Result;
use crate::ports::fakes::{FakeEnv, MemFs};

#[test]
fn defaults_round_trip_and_validate() -> Result<()> {
    let config = Config::defaults();
    config.validate()?;
    let text = config.render();
    let reparsed = Config::parse(&text)?;
    assert_eq!(reparsed, config);
    Ok(())
}

#[test]
fn set_validates_and_unset_prunes() -> Result<()> {
    let mut config = Config::defaults();
    assert!(!config.strict());
    config.set_str("behavior.strict", "true")?;
    assert_eq!(config.get_bool("behavior.strict"), Some(true));
    assert!(config.strict());

    assert!(config.set_str("behavior.strict", "maybe").is_err());
    assert!(config.set_str("nope.key", "x").is_err());
    assert!(config.set_str("ids.prefix_style", "wat").is_err());

    config.set_str("ids.prefix_style", "compact")?;
    assert_eq!(config.get_str("ids.prefix_style"), Some("compact"));

    assert!(config.unset("ids.prefix_style")?);
    assert!(!config.unset("ids.prefix_style")?);
    // `ids` ficou vazio e foi podado: nenhuma folha começa com `ids.`.
    assert!(
        config
            .list()
            .iter()
            .all(|(key, _)| !key.starts_with("ids."))
    );
    Ok(())
}

#[test]
fn project_overrides_global_and_secrets_are_stripped() -> Result<()> {
    let mut global = Config::defaults();
    global.set_str("recall.default_limit", "25")?;
    global.set_str("secrets.token", "s3cr3t")?;

    let mut project = Config::defaults();
    project.set_str("recall.default_limit", "5")?;
    project.set_str("secrets.token", "vazado")?;

    let effective = Config::effective(Some(&global), Some(&project));
    assert_eq!(effective.get_int("recall.default_limit"), Some(5));
    // O projeto não carrega segredos: o valor efetivo é o do global.
    assert_eq!(
        effective.get_str("secrets.token"),
        Some("s3cr3t"),
        "segredo do projeto não pode vencer o global"
    );
    Ok(())
}

#[test]
fn effective_without_global_uses_defaults_then_project() -> Result<()> {
    let mut project = Config::defaults();
    project.set_str("mcp.hints_cap", "7")?;
    let effective = Config::effective(None, Some(&project));
    assert_eq!(effective.get_int("mcp.hints_cap"), Some(7));
    assert_eq!(effective.get_int("recall.default_limit"), Some(10));
    Ok(())
}

#[test]
fn load_and_save_use_fs_port() -> Result<()> {
    let fs = MemFs::new();
    let path = Path::new("/p/.knudge/config.toml");
    assert!(Config::load(&fs, path)?.is_none());

    let mut config = Config::defaults();
    config.set_str("behavior.strict", "true")?;
    config.save(&fs, path)?;
    let loaded = Config::load(&fs, path)?.unwrap_or_default();
    assert_eq!(loaded.get_bool("behavior.strict"), Some(true));
    Ok(())
}

#[test]
fn global_path_follows_xdg_then_home() -> Result<()> {
    let mut env = FakeEnv {
        cwd: "/work".into(),
        ..FakeEnv::default()
    };
    env.vars.insert("HOME".into(), "/home/u".into());
    env.vars.insert("XDG_CONFIG_HOME".into(), "/xdg".into());
    let with_xdg = global_config_path(&env)?;
    assert!(with_xdg.ends_with("local/knudge/config.toml"));
    assert!(with_xdg.starts_with("/xdg"));

    env.vars.remove("XDG_CONFIG_HOME");
    let without_xdg = global_config_path(&env)?;
    assert!(without_xdg.starts_with("/home/u/.config"));
    Ok(())
}

#[test]
fn parse_tolerates_unknown_keys_but_validate_rejects() -> Result<()> {
    let config = Config::parse("desconhecida = 1\n")?;
    assert_eq!(config.get_int("desconhecida"), Some(1));
    assert!(config.validate().is_err());
    Ok(())
}

#[test]
fn invalid_set_value_is_rejected() {
    let mut config = Config::defaults();
    assert!(
        config
            .set_value("mcp.hints_cap", ConfigValue::Bool(true))
            .is_err()
    );
    // O valor default permanece intacto.
    assert_eq!(config.get_int("mcp.hints_cap"), Some(3));
}
