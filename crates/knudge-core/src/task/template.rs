//! Templates de plano (D105): `.knudge/templates.toml` no subset TOML próprio (D97).
//!
//! Built-ins (`feature`/`bug`/`refactor`) vêm do binário; o arquivo do projeto **sobrepõe** por
//! nome. O template é **configuração**, não nota — nada em `notas/` muda.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config::{ConfigValue, Table, toml};
use crate::ports::Fs;
use crate::{Error, Result};

/// Nome do catálogo dentro de `.knudge/`.
pub const TEMPLATE_FILE: &str = "templates.toml";

/// Template de plano.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTemplate {
    /// Nome (a chave no catálogo).
    pub name: String,
    /// Seções esperadas, em ordem canônica.
    pub sections: Vec<String>,
    /// Seções obrigatórias (subconjunto de `sections`).
    pub required: Vec<String>,
    /// Mínimo de passos (`steps`).
    pub min_steps: u32,
    /// Mínimo de critérios de aceite (`acceptance`).
    pub min_acceptance: u32,
}

/// Catálogo de templates (builtin + projeto).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TemplateCatalog {
    templates: BTreeMap<String, PlanTemplate>,
}

impl TemplateCatalog {
    /// Built-ins embutidos.
    #[must_use]
    pub fn builtin() -> Self {
        let mut catalog = Self::default();
        catalog.add(PlanTemplate {
            name: "feature".to_string(),
            sections: names(&["context", "approach", "steps", "acceptance"]),
            required: names(&["context", "approach", "steps", "acceptance"]),
            min_steps: 2,
            min_acceptance: 1,
        });
        catalog.add(PlanTemplate {
            name: "bug".to_string(),
            sections: names(&["context", "reproduction", "fix", "acceptance"]),
            required: names(&["context", "reproduction", "fix", "acceptance"]),
            min_steps: 1,
            min_acceptance: 1,
        });
        catalog.add(PlanTemplate {
            name: "refactor".to_string(),
            sections: names(&["context", "approach", "steps", "acceptance"]),
            required: names(&["context", "steps", "acceptance"]),
            min_steps: 2,
            min_acceptance: 1,
        });
        catalog
    }

    /// Carrega built-ins e, se existir, sobrepõe com `.knudge/templates.toml`.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Io`/`Config` em falha de leitura ou parse.
    pub fn load(fs: &dyn Fs, root: &Path) -> Result<Self> {
        let mut catalog = Self::builtin();
        let path = Self::path(root);
        if !fs.exists(&path) {
            return Ok(catalog);
        }
        let bytes = fs.read(&path)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Error::config(format!("catálogo não é UTF-8: {}", path.display())))?;
        for template in Self::parse(text)?.templates.into_values() {
            catalog.add(template);
        }
        Ok(catalog)
    }

    /// Interpreta o TOML do catálogo.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para sintaxe inválida ou entrada malformada.
    pub fn parse(text: &str) -> Result<Self> {
        let table = toml::parse(text)?;
        let Some(ConfigValue::Table(templates)) = table.get("templates") else {
            return Err(Error::config("templates.toml sem `[templates.<nome>]`"));
        };
        let mut out = Self::default();
        for (name, value) in templates {
            let ConfigValue::Table(entry) = value else {
                return Err(Error::config(format!("template `{name}` não é tabela")));
            };
            out.add(parse_entry(name, entry)?);
        }
        Ok(out)
    }

    /// Caminho do catálogo dentro de `.knudge/`.
    #[must_use]
    pub fn path(root: &Path) -> PathBuf {
        root.join(TEMPLATE_FILE)
    }

    /// Template pelo nome.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&PlanTemplate> {
        self.templates.get(name)
    }

    /// Nomes, ordenados.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.templates.keys().map(String::as_str).collect()
    }

    fn add(&mut self, template: PlanTemplate) {
        let _ignored = self.templates.insert(template.name.clone(), template);
    }
}

fn parse_entry(name: &str, entry: &Table) -> Result<PlanTemplate> {
    let sections = string_list(entry, "sections", name)?;
    let required = string_list(entry, "required", name)?;
    for item in &required {
        if !sections.contains(item) {
            return Err(Error::config(format!(
                "`{name}.required` cita seção ausente: {item}"
            )));
        }
    }
    Ok(PlanTemplate {
        name: name.to_string(),
        sections,
        required,
        min_steps: uint(entry, name, "min_steps", 1)?,
        min_acceptance: uint(entry, name, "min_acceptance", 1)?,
    })
}

fn string_list(entry: &Table, key: &str, name: &str) -> Result<Vec<String>> {
    match entry.get(key) {
        None => Ok(Vec::new()),
        Some(ConfigValue::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str().map(str::to_string).ok_or_else(|| {
                    Error::config(format!("`{name}.{key}` deve ser lista de strings"))
                })
            })
            .collect(),
        Some(_) => Err(Error::config(format!(
            "`{name}.{key}` deve ser lista de strings"
        ))),
    }
}

fn uint(entry: &Table, name: &str, key: &str, default: u32) -> Result<u32> {
    match entry.get(key) {
        None => Ok(default),
        Some(value) => {
            let int = value
                .as_int()
                .ok_or_else(|| Error::config(format!("`{name}.{key}` deve ser inteiro")))?;
            u32::try_from(int)
                .map_err(|_| Error::config(format!("`{name}.{key}` fora do intervalo")))
        }
    }
}

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}
