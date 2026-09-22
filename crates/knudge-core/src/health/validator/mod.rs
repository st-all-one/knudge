//! Catálogo de validators e resolução de `checks` (D54, E09-T01).
//!
//! O catálogo é **configuração**, não nota: `.knudge/validators.toml` (subset TOML próprio —
//! D97/D99). A resolução de `checks` de uma task combina três fontes:
//!
//! ```text
//! checks(task) = explícitos(task) ∪ globais ∪ por_âncora(anchors(task))
//! ```
//!
//! A task declara só o delta; o sistema expande. O catálogo é **executável** (comando +
//! severidade + timeout), mas a execução real fica na borda (`HookRunner`, E12) — o núcleo
//! apenas resolve e descreve.

mod resolve;

pub use resolve::{CheckSource, ResolvedCheck, ResolvedChecks, resolve_checks};

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config::{ConfigValue, Table, toml};
use crate::ports::Fs;
use crate::retrieval::anchor::glob_match;
use crate::{Error, Result};

/// Nome do catálogo dentro de `.knudge/`.
pub const CATALOG_FILE: &str = "validators.toml";

/// Timeout default de um validator (ms).
pub const DEFAULT_TIMEOUT_MS: u64 = 120_000;

/// Severidade de um validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Falha de severidade alta: derruba o fechamento para `failure`.
    Error,
    /// Falha que degrada para `partial`.
    Warn,
    /// Informativo: não afeta o resultado.
    Info,
}

impl Severity {
    /// Rótulo canônico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
        }
    }

    /// Interpreta o rótulo.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para valor desconhecido.
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            other => Err(Error::config(format!("severidade desconhecida: {other:?}"))),
        }
    }
}

/// Validator do catálogo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validator {
    /// Nome lógico (a chave no catálogo).
    pub name: String,
    /// Comando exato a executar.
    pub cmd: String,
    /// Globs de arquivos aos quais se aplica (vazio = só por declaração).
    pub scope: Vec<String>,
    /// Severidade.
    pub severity: Severity,
    /// Timeout em ms.
    pub timeout_ms: u64,
}

impl Validator {
    /// `true` se o validator se aplica a alguma das âncoras dadas.
    #[must_use]
    pub fn applies_to(&self, anchors: &[String]) -> bool {
        self.scope
            .iter()
            .any(|pattern| anchors.iter().any(|anchor| glob_match(pattern, anchor)))
    }
}

/// Catálogo de validators.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidatorCatalog {
    validators: BTreeMap<String, Validator>,
    globals: Vec<String>,
}

impl ValidatorCatalog {
    /// Catálogo vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Interpreta o TOML do catálogo.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para sintaxe inválida, validator sem `cmd` ou global que
    /// não existe no catálogo.
    pub fn parse(text: &str) -> Result<Self> {
        let table = toml::parse(text)?;
        Self::from_table(&table)
    }

    /// Carrega o catálogo de `.knudge/validators.toml`, se existir.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`Config` em falha de leitura ou parse.
    pub fn load(fs: &dyn Fs, root: &Path) -> Result<Self> {
        let path = Self::path(root);
        if !fs.exists(&path) {
            return Ok(Self::new());
        }
        let bytes = fs.read(&path)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Error::config(format!("catálogo não é UTF-8: {}", path.display())))?;
        Self::parse(text)
    }

    /// Caminho do catálogo dentro de `.knudge/`.
    #[must_use]
    pub fn path(root: &Path) -> PathBuf {
        root.join(CATALOG_FILE)
    }

    /// Validator pelo nome.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Validator> {
        self.validators.get(name)
    }

    /// Nomes dos validators globais.
    #[must_use]
    pub fn globals(&self) -> &[String] {
        &self.globals
    }

    /// Todos os validators, em ordem de nome.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Validator)> {
        self.validators
            .iter()
            .map(|(name, validator)| (name.as_str(), validator))
    }

    /// Número de validators.
    #[must_use]
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// `true` se o catálogo está vazio.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    fn from_table(table: &Table) -> Result<Self> {
        let mut validators = BTreeMap::new();
        let mut globals = Vec::new();
        for (key, value) in table {
            if key == "globals" {
                globals = string_array(value, "globals")?;
                continue;
            }
            let Some(entry) = value.as_table() else {
                return Err(Error::config(format!(
                    "validator `{key}` deve ser uma tabela"
                )));
            };
            validators.insert(key.clone(), parse_validator(key, entry)?);
        }
        for name in &globals {
            if !validators.contains_key(name) {
                return Err(Error::config(format!(
                    "global `{name}` não existe no catálogo"
                )));
            }
        }
        Ok(Self {
            validators,
            globals,
        })
    }
}

fn parse_validator(name: &str, table: &Table) -> Result<Validator> {
    let cmd = table
        .get("cmd")
        .and_then(ConfigValue::as_str)
        .ok_or_else(|| Error::config(format!("validator `{name}` sem `cmd`")))?
        .to_string();
    let scope = match table.get("scope") {
        Some(value) => string_array(value, "scope")?,
        None => Vec::new(),
    };
    let severity =
        match table.get("severity") {
            Some(value) => Severity::parse(value.as_str().ok_or_else(|| {
                Error::config(format!("`severity` de `{name}` deve ser string"))
            })?)?,
            None => Severity::Error,
        };
    let timeout_ms = match table.get("timeout") {
        Some(value) => {
            let raw = value
                .as_int()
                .ok_or_else(|| Error::config(format!("`timeout` de `{name}` deve ser inteiro")))?;
            u64::try_from(raw)
                .map_err(|_| Error::config(format!("`timeout` de `{name}` deve ser positivo")))?
        }
        None => DEFAULT_TIMEOUT_MS,
    };
    Ok(Validator {
        name: name.to_string(),
        cmd,
        scope,
        severity,
        timeout_ms,
    })
}

fn string_array(value: &ConfigValue, key: &str) -> Result<Vec<String>> {
    let Some(items) = value.as_array() else {
        return Err(Error::config(format!(
            "`{key}` deve ser uma lista de strings"
        )));
    };
    let mut out = Vec::new();
    for item in items {
        let text = item
            .as_str()
            .ok_or_else(|| Error::config(format!("`{key}` deve conter apenas strings")))?;
        out.push(text.to_string());
    }
    Ok(out)
}
