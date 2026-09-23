//! Plano preenchível por LLM (D105): `prompt` derivado e `submit_plan` atômico.
//!
//! O prompt é **derivado** (TOON, read-only); o plano submetido é validado por inteiro antes de
//! qualquer escrita (atomicidade lógica). Nada em `notas/` muda de formato.

use std::collections::BTreeSet;

use indexmap::IndexMap;

use crate::schema::{EdgeKind, NoteType, Scope, Value};
use crate::toon;
use crate::write::{WriteContext, link};
use crate::{Error, Result};

use super::template::PlanTemplate;
use super::{SubmitOutcome, TaskSpec, submit};

/// Prompt de plano (TOON derivado).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanPrompt {
    /// Nome do template.
    pub template: String,
    /// Id do container-semente.
    pub seed: String,
    /// Seções esperadas.
    pub sections: Vec<String>,
    /// Seções obrigatórias.
    pub required: Vec<String>,
    /// Mínimo de passos.
    pub min_steps: u32,
    /// Mínimo de critérios de aceite.
    pub min_acceptance: u32,
}

impl PlanPrompt {
    /// Serializa em TOON canônico.
    #[must_use]
    pub fn to_toon(&self) -> String {
        let mut map = IndexMap::new();
        map.insert("template".to_string(), Value::Str(self.template.clone()));
        map.insert("seed".to_string(), Value::Str(self.seed.clone()));
        map.insert("sections".to_string(), string_list(&self.sections));
        map.insert("required".to_string(), string_list(&self.required));
        map.insert(
            "min_steps".to_string(),
            Value::Int(i64::from(self.min_steps)),
        );
        map.insert(
            "min_acceptance".to_string(),
            Value::Int(i64::from(self.min_acceptance)),
        );
        toon::emit(&Value::Map(map))
    }
}

fn string_list(items: &[String]) -> Value {
    Value::List(items.iter().map(|item| Value::Str(item.clone())).collect())
}

/// Passo do plano submetido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStep {
    /// Afirmação do filho.
    pub title: String,
    /// Espécie (`type`), quando difere do default (D113).
    pub kind: Option<NoteType>,
    /// Dependências (`depends_on`).
    pub depends_on: Vec<String>,
    /// Validators.
    pub checks: Vec<String>,
    /// Âncoras.
    pub anchors: Vec<String>,
}

/// Plano submetido (TOON).
#[derive(Debug, Clone, PartialEq)]
pub struct PlanSpec {
    /// Nome do template.
    pub template: String,
    /// Nome opcional do plano.
    pub name: Option<String>,
    /// Seções preenchidas (ordem preservada).
    pub sections: IndexMap<String, Value>,
}

impl PlanSpec {
    /// Interpreta o TOON submetido.
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` para documento sem `template`/`sections`.
    pub fn parse(value: &Value) -> Result<Self> {
        let Value::Map(map) = value else {
            return Err(Error::invalid_input("plano deve ser um mapa TOON"));
        };
        let template = map
            .get("template")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::invalid_input("plano sem `template`"))?
            .to_string();
        let name = map.get("name").and_then(Value::as_str).map(str::to_string);
        let sections = match map.get("sections") {
            Some(Value::Map(sections)) => sections.clone(),
            _ => return Err(Error::invalid_input("plano sem `sections`")),
        };
        Ok(Self {
            template,
            name,
            sections,
        })
    }
}

/// Deriva o prompt de plano para um container (read-only).
///
/// # Errors
/// Retorna `ErrorKind::Schema` se `id` não for container; propaga I/O.
pub fn prompt(ctx: &WriteContext<'_>, id: &str, template: &PlanTemplate) -> Result<PlanPrompt> {
    let note = ctx.store().read(id)?;
    if note.frontmatter.note_type()? != NoteType::Container {
        return Err(Error::schema(format!("{id} não é container")));
    }
    Ok(PlanPrompt {
        template: template.name.clone(),
        seed: id.to_string(),
        sections: template.sections.clone(),
        required: template.required.clone(),
        min_steps: template.min_steps,
        min_acceptance: template.min_acceptance,
    })
}

/// Valida o plano inteiro e, só então, cria os filhos (atomicidade lógica).
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput`/`Schema`/`Conflict` para plano inválido; propaga I/O.
pub fn submit_plan(
    ctx: &WriteContext<'_>,
    template: &PlanTemplate,
    id: &str,
    spec: &PlanSpec,
) -> Result<Vec<SubmitOutcome>> {
    let steps = parse_steps(&spec.sections)?;
    validate_sections(template, &spec.sections, &steps)?;
    let parent = ctx.store().read(id)?;
    let parent_scope = parent
        .frontmatter
        .scope()?
        .ok_or_else(|| Error::schema(format!("{id} não é container")))?;
    let child_scope = super::child(parent_scope)
        .ok_or_else(|| Error::schema(format!("{id} ({parent_scope}) não aceita filhos")))?;
    let specs = prepare(ctx, child_scope, id, &steps)?;
    let mut out = Vec::new();
    for (spec, step) in specs.iter().zip(&steps) {
        let outcome = submit(ctx, spec)?;
        for dependency in &step.depends_on {
            link(ctx, &outcome.id, EdgeKind::DependsOn, dependency)?;
        }
        out.push(outcome);
    }
    Ok(out)
}

fn parse_steps(sections: &IndexMap<String, Value>) -> Result<Vec<PlanStep>> {
    let Some(Value::List(items)) = sections.get("steps") else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for item in items {
        out.push(parse_step(item)?);
    }
    Ok(out)
}

fn parse_step(value: &Value) -> Result<PlanStep> {
    let Value::Map(map) = value else {
        return Err(Error::invalid_input("passo deve ser um mapa"));
    };
    let title = map
        .get("title")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::invalid_input("passo sem `title`"))?
        .to_string();
    let kind = match map.get("kind").and_then(Value::as_str) {
        Some(text) => Some(text.parse::<NoteType>()?),
        None => None,
    };
    Ok(PlanStep {
        title,
        kind,
        depends_on: str_list(map.get("depends_on")),
        checks: str_list(map.get("checks")),
        anchors: str_list(map.get("anchors")),
    })
}

fn str_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::List(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn validate_sections(
    template: &PlanTemplate,
    sections: &IndexMap<String, Value>,
    steps: &[PlanStep],
) -> Result<()> {
    for name in &template.required {
        match sections.get(name) {
            None => {
                return Err(Error::invalid_input(format!(
                    "seção obrigatória ausente: {name}"
                )));
            }
            Some(value) if is_empty(value) => {
                return Err(Error::invalid_input(format!("seção vazia: {name}")));
            }
            Some(_) => {}
        }
    }
    let min_steps = usize::try_from(template.min_steps).unwrap_or(usize::MAX);
    if steps.len() < min_steps {
        return Err(Error::invalid_input(format!(
            "plano exige ao menos {} passos (recebeu {})",
            template.min_steps,
            steps.len()
        )));
    }
    let acceptance = count_list(sections.get("acceptance"));
    let min_acceptance = usize::try_from(template.min_acceptance).unwrap_or(usize::MAX);
    if acceptance < min_acceptance {
        return Err(Error::invalid_input(format!(
            "plano exige ao menos {} critérios de aceite (recebeu {acceptance})",
            template.min_acceptance
        )));
    }
    Ok(())
}

fn is_empty(value: &Value) -> bool {
    match value {
        Value::Str(text) => text.trim().is_empty(),
        Value::List(items) => items.is_empty(),
        Value::Map(map) => map.is_empty(),
        _ => false,
    }
}

fn count_list(value: Option<&Value>) -> usize {
    match value {
        Some(Value::List(items)) => items.len(),
        _ => 0,
    }
}

fn prepare(
    ctx: &WriteContext<'_>,
    child_scope: Scope,
    parent: &str,
    steps: &[PlanStep],
) -> Result<Vec<TaskSpec>> {
    let mut specs = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for (index, step) in steps.iter().enumerate() {
        let mut spec = TaskSpec::new(child_scope, step.title.clone());
        spec.parent = Some(parent.to_string());
        let position = u32::try_from(index.saturating_add(1))
            .map_err(|_| Error::invalid_input("plano com passos demais"))?;
        spec.blocks = Some(position);
        spec.kind = step.kind;
        spec.checks.clone_from(&step.checks);
        spec.anchors.clone_from(&step.anchors);
        let note = spec.to_note(ctx.now_ms())?;
        let note_id = note.id()?.to_string();
        if ctx.store().exists(&note_id) || !seen.insert(note_id.clone()) {
            return Err(Error::conflict(format!("passo colide por id: {note_id}")));
        }
        specs.push(spec);
    }
    Ok(specs)
}
