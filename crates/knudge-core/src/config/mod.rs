//! Escopo `config`: configuração em dois níveis (D61–D64).
//!
//! O **global** é o template/default curado pelo usuário; o do **projeto** é o efetivo e tem
//! **precedência**. `config set/unset` valida contra o schema (D64); o writer mantém ordem de
//! inserção e quoting estáveis para gerar diff mínimo (D63). Segredos vivem **só no global**
//! (D91) e são removidos do projeto em [`Config::effective`].

pub mod schema;
pub mod table;
pub mod toml;
pub mod value;

#[cfg(test)]
mod tests;

pub use schema::{KEYS, KeySpec, Kind, default_table};
pub use value::{ConfigValue, Table};

use std::path::{Path, PathBuf};

use crate::ports::{Env, Fs};
use crate::{Error, Result};

use table::{flatten, get_path, merge_into, remove_path, remove_prefix, set_path};

/// Nome do arquivo de configuração.
pub const CONFIG_FILE: &str = "config.toml";

/// Configuração do knudge.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    table: Table,
}

impl Default for Config {
    fn default() -> Self {
        Self::defaults()
    }
}

impl Config {
    /// Configuração com os defaults embutidos (ordem canônica).
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            table: default_table(),
        }
    }

    /// Envolve uma tabela já construída.
    #[must_use]
    pub const fn from_table(table: Table) -> Self {
        Self { table }
    }

    /// Interpreta TOML (sem validar contra o schema).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a sintaxe for inválida.
    pub fn parse(text: &str) -> Result<Self> {
        Ok(Self {
            table: toml::parse(text)?,
        })
    }

    /// Serializa como TOML canônico.
    #[must_use]
    pub fn render(&self) -> String {
        toml::emit(&self.table)
    }

    /// Tabela subjacente.
    #[must_use]
    pub const fn table(&self) -> &Table {
        &self.table
    }

    /// Lê um valor bruto.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        let parts = table::split_key(key).ok()?;
        get_path(&self.table, &parts)
    }

    /// Lê um booleano.
    #[must_use]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(ConfigValue::as_bool)
    }

    /// Lê um inteiro.
    #[must_use]
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(ConfigValue::as_int)
    }

    /// Lê um float.
    #[must_use]
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(ConfigValue::as_float)
    }

    /// Lê uma string.
    #[must_use]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(ConfigValue::as_str)
    }

    /// Define um valor já tipado, validando contra o schema.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a chave for desconhecida ou o tipo não bater.
    pub fn set_value(&mut self, key: &str, value: ConfigValue) -> Result<()> {
        schema::validate_leaf(key, &value).map_err(Error::config)?;
        let parts = table::split_key(key).map_err(Error::config)?;
        set_path(&mut self.table, &parts, value).map_err(Error::config)
    }

    /// Define a partir de um texto de CLI, convertendo conforme o tipo da chave.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se o texto não converter ou a chave for inválida.
    pub fn set_str(&mut self, key: &str, raw: &str) -> Result<()> {
        let value = if schema::is_secret(key) {
            ConfigValue::String(raw.to_string())
        } else {
            let spec = schema::spec(key)
                .ok_or_else(|| Error::config(format!("chave desconhecida: `{key}`")))?;
            parse_raw(&spec.kind, raw)?
        };
        self.set_value(key, value)
    }

    /// Remove uma chave e poda ancestrais vazios (D64).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` se a chave for inválida.
    pub fn unset(&mut self, key: &str) -> Result<bool> {
        let parts = table::split_key(key).map_err(Error::config)?;
        Ok(remove_path(&mut self.table, &parts))
    }

    /// Lista as folhas como `(chave.pontilhada, valor renderizado)`.
    #[must_use]
    pub fn list(&self) -> Vec<(String, String)> {
        flatten(&self.table)
            .into_iter()
            .map(|(key, value)| (key, value.render()))
            .collect()
    }

    /// Valida todas as folhas contra o schema.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` no primeiro problema.
    pub fn validate(&self) -> Result<()> {
        schema::validate_table(&self.table).map_err(Error::config)
    }

    /// Remove a seção `secrets` (o projeto nunca carrega credenciais — D91).
    pub fn strip_secrets(&mut self) {
        remove_prefix(&mut self.table, schema::SECRETS_PREFIX);
    }

    /// Combina defaults (base) + global + projeto (precedência), sem segredos no projeto.
    #[must_use]
    pub fn effective(global: Option<&Self>, project: Option<&Self>) -> Self {
        let mut config = Self::defaults();
        if let Some(global) = global {
            merge_into(&mut config.table, &global.table);
        }
        if let Some(project) = project {
            let mut sanitized = project.clone();
            sanitized.strip_secrets();
            merge_into(&mut config.table, &sanitized.table);
        }
        config
    }

    /// Lê a configuração de um arquivo, se existir.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`Config` em falha de leitura ou sintaxe.
    pub fn load(fs: &dyn Fs, path: &Path) -> Result<Option<Self>> {
        if !fs.exists(path) {
            return Ok(None);
        }
        let bytes = fs.read(path)?;
        let text = String::from_utf8(bytes)
            .map_err(|_| Error::config(format!("config não é UTF-8: {}", path.display())))?;
        Self::parse(&text).map(Some)
    }

    /// Grava a configuração atomicamente, criando o diretório pai.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io` se a escrita falhar.
    pub fn save(&self, fs: &dyn Fs, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs.create_dir_all(parent)?;
        }
        fs.write_atomic(path, self.render().as_bytes())
    }
}

/// Converte texto de CLI no valor do tipo esperado.
fn parse_raw(kind: &Kind, raw: &str) -> Result<ConfigValue> {
    let trimmed = raw.trim();
    match kind {
        Kind::Bool => match trimmed {
            "true" => Ok(ConfigValue::Bool(true)),
            "false" => Ok(ConfigValue::Bool(false)),
            other => Err(Error::config(format!("espera true|false: `{other}`"))),
        },
        Kind::Int => trimmed
            .parse::<i64>()
            .map(ConfigValue::Int)
            .map_err(|_| Error::config(format!("inteiro inválido: `{trimmed}`"))),
        Kind::Float => trimmed
            .parse::<f64>()
            .map(ConfigValue::Float)
            .map_err(|_| Error::config(format!("float inválido: `{trimmed}`"))),
        Kind::Text | Kind::Enum(_) => Ok(ConfigValue::String(trimmed.to_string())),
    }
}

/// Resolve o caminho do config global conforme a plataforma (D61).
///
/// # Errors
/// Retorna `ErrorKind::Config` se a variável de ambiente necessária faltar.
pub fn global_config_path(env: &dyn Env) -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let appdata = env
            .var("APPDATA")
            .ok_or_else(|| Error::config("APPDATA não definido"))?;
        return Ok(PathBuf::from(appdata).join("knudge").join(CONFIG_FILE));
    }
    if cfg!(target_os = "macos") {
        let home = env
            .var("HOME")
            .ok_or_else(|| Error::config("HOME não definido"))?;
        return Ok(PathBuf::from(home)
            .join("Library/Application Support/knudge")
            .join(CONFIG_FILE));
    }
    let base = match env.var("XDG_CONFIG_HOME") {
        Some(xdg) if !xdg.is_empty() => PathBuf::from(xdg),
        _ => {
            let home = env
                .var("HOME")
                .ok_or_else(|| Error::config("HOME não definido"))?;
            PathBuf::from(home).join(".config")
        }
    };
    Ok(base.join("local").join("knudge").join(CONFIG_FILE))
}
