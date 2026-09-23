//! `kd task plan` — prompt, submissão (`--step`/`--from`) e ciclo de vida (D105).

use std::io::Read;
use std::path::Path;

use knudge_core::Error;
use knudge_core::Result;
use knudge_core::schema::Scope;
use knudge_core::task::plan::{PlanSpec, prompt as plan_prompt, submit_plan};
use knudge_core::task::template::TemplateCatalog;
use knudge_core::task::{PlanTemplate, TaskAction, TaskSpec, apply, child, reorder, submit};
use knudge_core::toon;
use knudge_core::write::WriteContext;
use serde_json::json;

use crate::cli::TaskPlanArgs;
use crate::output::Output;
use crate::session::Session;

/// Executa `kd task plan`.
///
/// # Errors
/// Propaga erros de validação/escrita do plano e I/O.
pub(super) fn run(session: &Session, args: &TaskPlanArgs) -> Result<Output> {
    if args.prompt {
        return render_prompt(session, args);
    }
    let ctx = session.write_context()?;
    if let Some(from) = &args.from {
        return submit_from(session, &ctx, args, from);
    }
    if !args.steps.is_empty() {
        return submit_steps(&ctx, args);
    }
    lifecycle(&ctx, args)
}

fn render_prompt(session: &Session, args: &TaskPlanArgs) -> Result<Output> {
    let ctx = session.write_context()?;
    let catalog = TemplateCatalog::load(session.fs_dyn(), &session.knowledge_dir())?;
    let name = args.template.as_deref().unwrap_or("feature");
    let template = lookup(&catalog, name)?;
    let text = plan_prompt(&ctx, &args.id, template)?.to_toon();
    let data = json!({ "template": name, "seed": args.id, "prompt": text });
    Ok(Output::new(text, data))
}

fn submit_from(
    session: &Session,
    ctx: &WriteContext<'_>,
    args: &TaskPlanArgs,
    from: &str,
) -> Result<Output> {
    let catalog = TemplateCatalog::load(session.fs_dyn(), &session.knowledge_dir())?;
    let text = read_source(session, from)?;
    let spec = PlanSpec::parse(&toon::parse(&text)?)?;
    let template = lookup(&catalog, &spec.template)?;
    let outcomes = submit_plan(ctx, template, &args.id, &spec)?;
    let ids: Vec<String> = outcomes.iter().map(|outcome| outcome.id.clone()).collect();
    let data = json!({ "plan": args.id, "template": spec.template, "children": ids });
    Ok(Output::new(ids.join("\n"), data))
}

fn read_source(session: &Session, from: &str) -> Result<String> {
    if from == "-" {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| Error::io("stdin", error))?;
        return Ok(buffer);
    }
    let bytes = session.fs_dyn().read(Path::new(from))?;
    String::from_utf8(bytes).map_err(|_| Error::config(format!("plano não é UTF-8: {from}")))
}

fn submit_steps(ctx: &WriteContext<'_>, args: &TaskPlanArgs) -> Result<Output> {
    let child_scope = child_scope(ctx, &args.id)?;
    let mut ids = Vec::new();
    for (index, step) in args.steps.iter().enumerate() {
        let mut spec = TaskSpec::new(child_scope, step.clone());
        spec.parent = Some(args.id.clone());
        spec.blocks = Some(position(index)?);
        ids.push(submit(ctx, &spec)?.id);
    }
    let data = json!({ "plan": args.id, "children": ids });
    Ok(Output::new(ids.join("\n"), data))
}

fn child_scope(ctx: &WriteContext<'_>, id: &str) -> Result<Scope> {
    let parent = ctx.store().read(id)?;
    let scope = parent
        .frontmatter
        .scope()?
        .ok_or_else(|| Error::schema(format!("{id} não é container")))?;
    child(scope).ok_or_else(|| Error::schema(format!("{id} ({scope}) não aceita filhos")))
}

fn position(index: usize) -> Result<u32> {
    u32::try_from(index.saturating_add(1)).map_err(|_| Error::invalid_input("passos demais"))
}

fn lifecycle(ctx: &WriteContext<'_>, args: &TaskPlanArgs) -> Result<Output> {
    let action = if args.adopt {
        Some(TaskAction::Adopt)
    } else if args.release {
        Some(TaskAction::Release)
    } else if args.review {
        Some(TaskAction::Review)
    } else {
        None
    };
    if let Some(action) = action {
        let revision = apply(ctx, &args.id, action)?;
        let data = json!({ "id": args.id, "action": action.as_str(), "revision": revision });
        return Ok(Output::new(
            format!("{}|{}|r{revision}", action.as_str(), args.id),
            data,
        ));
    }
    if let Some(blocks) = args.reorder {
        let revision = reorder(ctx, &args.id, blocks)?;
        let data = json!({
            "id": args.id,
            "action": "reorder",
            "blocks": blocks,
            "revision": revision,
        });
        return Ok(Output::new(
            format!("reorder|{}|{blocks}|r{revision}", args.id),
            data,
        ));
    }
    if args.submit {
        return Err(Error::invalid_input(
            "`--submit` exige `--step` ou `--from`",
        ));
    }
    Err(Error::invalid_input(
        "use `--prompt`, `--step`, `--from`, `--adopt`, `--release`, `--review` ou `--reorder`",
    ))
}

fn lookup<'a>(catalog: &'a TemplateCatalog, name: &str) -> Result<&'a PlanTemplate> {
    catalog.get(name).ok_or_else(|| {
        Error::invalid_input(format!(
            "template desconhecido: {name}; disponíveis: {}",
            catalog.names().join(", ")
        ))
    })
}
