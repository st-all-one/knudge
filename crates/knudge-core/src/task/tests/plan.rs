//! Plano preenchível por LLM (D105).

use crate::Result;
use crate::ports::fakes::MemFs;
use crate::schema::{NoteType, Scope};
use crate::task::PlanTemplate;
use crate::task::plan::{PlanSpec, prompt, submit_plan};
use crate::task::template::TemplateCatalog;
use crate::task::{TaskSpec, submit};
use crate::toon;

use super::context;

fn feature(catalog: &TemplateCatalog) -> Option<&PlanTemplate> {
    catalog.get("feature")
}

#[test]
fn prompt_is_read_only_toon() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let plan = submit(&ctx, &TaskSpec::new(Scope::Epic, "programa"))?;
    let catalog = TemplateCatalog::builtin();
    let Some(template) = feature(&catalog) else {
        return Ok(());
    };
    let text = prompt(&ctx, &plan.id, template)?.to_toon();
    assert!(text.contains("template: feature"), "{text}");
    assert!(text.contains("min_steps: 2"), "{text}");
    Ok(())
}

#[test]
fn prompt_rejects_non_container() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let issue = submit(&ctx, &TaskSpec::new(Scope::Issue, "issue"))?;
    let catalog = TemplateCatalog::builtin();
    let Some(template) = feature(&catalog) else {
        return Ok(());
    };
    assert!(prompt(&ctx, &issue.id, template).is_err());
    Ok(())
}

#[test]
fn submit_plan_creates_children_with_kind() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let plan = submit(&ctx, &TaskSpec::new(Scope::Epic, "programa"))?;
    let spec = parse_plan(
        "template: feature\nsections:\n  context: ctx\n  approach: app\n  steps:\n    - title: Corrigir off-by-one\n      kind: error\n    - title: Implementar retry\n  acceptance:\n    - teste passa\n",
    )?;
    let catalog = TemplateCatalog::builtin();
    let Some(template) = feature(&catalog) else {
        return Ok(());
    };
    let out = submit_plan(&ctx, template, &plan.id, &spec)?;
    assert_eq!(out.len(), 2);
    let Some(first_id) = out.first().map(|outcome| outcome.id.clone()) else {
        return Ok(());
    };
    let first = ctx.store().read(&first_id)?;
    assert_eq!(first.frontmatter.note_type()?, NoteType::Error);
    assert_eq!(first.frontmatter.scope()?, Some(Scope::Task));
    Ok(())
}

#[test]
fn invalid_plan_writes_nothing() -> Result<()> {
    let fs = MemFs::new();
    let ctx = context(&fs)?;
    let plan = submit(&ctx, &TaskSpec::new(Scope::Epic, "programa"))?;
    let before = ctx.store().list_ids()?.len();
    // `acceptance` obrigatório ausente.
    let spec = parse_plan(
        "template: feature\nsections:\n  context: ctx\n  approach: app\n  steps:\n    - title: A\n    - title: B\n",
    )?;
    let catalog = TemplateCatalog::builtin();
    let Some(template) = feature(&catalog) else {
        return Ok(());
    };
    assert!(submit_plan(&ctx, template, &plan.id, &spec).is_err());
    assert_eq!(ctx.store().list_ids()?.len(), before);
    Ok(())
}

fn parse_plan(text: &str) -> Result<PlanSpec> {
    PlanSpec::parse(&toon::parse(text)?)
}
